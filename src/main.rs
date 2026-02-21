use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

mod cli;
mod rack;
mod services;

use cli::*;
use services::adapters::{adapter_doctor, adapter_smoke, sync_installed};
use services::authoring::{
    hook_create, hook_remove, mcp_create, mcp_remove, plugin_create, plugin_remove, plugin_update,
    skill_create, skill_remove, subagent_create, subagent_remove,
};
use services::policy::source_matches_allowed;
use services::rack_ops::{
    rack_doctor, rack_license_audit, rack_license_audit_readonly, rack_mark_unknown_external,
    rack_sign_marketplace, rack_sync_upstreams,
};
use services::release_check::build_release_check_report;
use services::storage::{
    audit, load_state, materialize_plugin, save_lockfile, save_state, upsert_installed,
};
use services::trust::{list_pubkeys, trust_init, verify_marketplace_signature};

const OFFICIAL_RACK_PUBKEY_HEX: &str =
    "5aefcc2a6716ef9fab24dc3865013e29a8d579e4dda33bf753a7cd7a8d14450a";

fn default_scope() -> InstallScope {
    InstallScope::User
}

#[derive(Serialize)]
struct JsonOut<T: Serialize> {
    ok: bool,
    data: T,
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct State {
    marketplaces: Vec<MarketRef>,
    installed: Vec<InstalledPlugin>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct MarketRef {
    name: String,
    source: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct InstalledPlugin {
    name: String,
    marketplace: String,
    #[serde(default)]
    marketplace_source: String,
    source: String,
    #[serde(default)]
    local_path: String,
    version: Option<String>,
    #[serde(default)]
    permissions: Vec<String>,
    #[serde(default = "default_scope")]
    scope: InstallScope,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct Lockfile {
    version: u32,
    plugins: Vec<InstalledPlugin>,
}

#[derive(Debug, Deserialize)]
struct PolicyFile {
    #[serde(default)]
    general: PolicyGeneral,
}

#[derive(Debug, Deserialize, Default)]
struct PolicyGeneral {
    #[serde(default)]
    require_signed_marketplace: bool,
    #[serde(default)]
    allowed_sources: Vec<String>,
    #[serde(default)]
    denied_plugins: Vec<String>,
    #[serde(default)]
    blocked_permissions: Vec<String>,
    #[serde(default)]
    block_unknown_licenses: bool,
    #[serde(default)]
    allow_unknown_license_plugins: Vec<String>,
    #[serde(default)]
    allow_external_reference_installs: bool,
    #[serde(default)]
    allow_external_reference_plugins: Vec<String>,
}

fn main() {
    let cli = Cli::parse();
    let json = cli.json;
    if let Err(e) = run(cli) {
        if json {
            let out = serde_json::json!({
                "ok": false,
                "error": {
                    "code": map_error_code(&e.to_string()),
                    "message": e.to_string(),
                    "hint": error_hint(&e.to_string()),
                    "retryable": false
                },
                "meta": {"version": "v1"}
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&out).unwrap_or_else(|_| "{\"ok\":false}".to_string())
            );
        } else {
            eprintln!("error: {}", e);
        }
        std::process::exit(1);
    }
}

fn map_error_code(msg: &str) -> &'static str {
    let m = msg.to_ascii_lowercase();
    if m.contains("policy") {
        "POLICY_DENY"
    } else if m.contains("signature") {
        "SIGNATURE_INVALID"
    } else if m.contains("not found") {
        "NOT_FOUND"
    } else if m.contains("permission") {
        "PERMISSION_DELTA_BLOCKED"
    } else {
        "INTERNAL_ERROR"
    }
}

fn error_hint(msg: &str) -> &'static str {
    let m = msg.to_ascii_lowercase();
    if m.contains("signature") {
        "run `pater trust init` and verify marketplace.sig"
    } else if m.contains("policy") {
        "review ~/.config/pater/policy.toml"
    } else if m.contains("not found") {
        "check plugin/marketplace name and run `pater search`"
    } else {
        "run `pater --json check` for diagnostics"
    }
}

fn handle_trust_commands(cli: &Cli, policy: &PolicyFile) -> anyhow::Result<bool> {
    let Commands::Trust { command } = &cli.command else {
        return Ok(false);
    };

    match command {
        TrustCommands::Init => {
            trust_init(OFFICIAL_RACK_PUBKEY_HEX)?;
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&JsonOut {
                        ok: true,
                        data: "initialized"
                    })?
                );
            } else {
                println!("trust initialized (official rack key installed)");
            }
        }
        TrustCommands::List => {
            let keys = list_pubkeys()?;
            print_out(cli.json, &keys, |k| k.to_string())?;
        }
        TrustCommands::Status => {
            let keys = list_pubkeys()?;
            let sig_ok = verify_marketplace_signature(DEFAULT_MARKETPLACE_SOURCE).unwrap_or(false);
            let status = TrustStatus {
                require_signed_marketplace: policy.general.require_signed_marketplace,
                trusted_key_count: keys.len(),
                default_marketplace: DEFAULT_MARKETPLACE_SOURCE.to_string(),
                default_marketplace_signature_ok: sig_ok,
            };
            print_one(cli.json, status, |s| {
                format!(
                    "signed_required={} keys={} default_sig_ok={}",
                    s.require_signed_marketplace,
                    s.trusted_key_count,
                    s.default_marketplace_signature_ok
                )
            })?;
        }
    }

    Ok(true)
}

fn handle_rack_commands(cli: &Cli) -> anyhow::Result<bool> {
    let Commands::Rack { command } = &cli.command else {
        return Ok(false);
    };

    match command {
        RackCommands::Doctor { rack_dir, sign_key } => {
            let report = rack_doctor(rack_dir, sign_key.as_deref());
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&JsonOut {
                        ok: report.overall == "ok",
                        data: report
                    })?
                );
            } else {
                println!("rack doctor: {}", report.overall);
                for c in report.checks {
                    println!("{}	{}", c.name, c.status);
                }
            }
        }
        RackCommands::Sync { rack_dir } => {
            let count = rack_sync_upstreams(rack_dir)?;
            print_one(cli.json, count, |c| format!("synced {} plugins", c))?;
        }
        RackCommands::MarkUnknownExternal { rack_dir } => {
            let count = rack_mark_unknown_external(rack_dir)?;
            print_one(cli.json, count, |c| format!("marked {} plugins", c))?;
        }
        RackCommands::LicenseAudit { rack_dir } => {
            let report = rack_license_audit(rack_dir)?;
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&JsonOut {
                        ok: report.unknown_count == 0,
                        data: report.clone()
                    })?
                );
            } else {
                println!(
                    "license audit: permissive={} copyleft={} unknown={} total= {}",
                    report.permissive, report.copyleft, report.unknown_count, report.total
                );
            }
            if report.unknown_count > 0 {
                std::process::exit(1);
            }
        }
        RackCommands::Sign { rack_dir, sign_key } => {
            rack_sign_marketplace(rack_dir, sign_key)?;
            print_one(cli.json, "signed", |_| "marketplace signed".to_string())?;
        }
        RackCommands::PrepareRelease { rack_dir, sign_key } => {
            let synced = rack_sync_upstreams(rack_dir)?;
            let marked = rack_mark_unknown_external(rack_dir)?;
            let report = rack_license_audit(rack_dir)?;
            rack_sign_marketplace(rack_dir, sign_key)?;
            let data = serde_json::json!({
                "synced": synced,
                "marked": marked,
                "license": report,
                "signed": true
            });
            print_one(cli.json, data, |_| "rack release prepared".to_string())?;
            if report.unknown_count > 0 {
                std::process::exit(1);
            }
        }
    }

    Ok(true)
}

fn handle_author_commands(cli: &Cli) -> anyhow::Result<bool> {
    let Commands::Author { command } = &cli.command else {
        return Ok(false);
    };

    match command {
        AuthorCommands::Plugin { command } => match command {
            PluginCommands::Create {
                name,
                rack_dir,
                description,
            } => {
                plugin_create(rack_dir, name, description)?;
                print_one(cli.json, "created", |_| format!("plugin {} created", name))?;
            }
            PluginCommands::Update {
                name,
                rack_dir,
                description,
                version,
            } => {
                plugin_update(rack_dir, name, description.clone(), version.clone())?;
                print_one(cli.json, "updated", |_| format!("plugin {} updated", name))?;
            }
            PluginCommands::Remove { name, rack_dir } => {
                plugin_remove(rack_dir, name)?;
                print_one(cli.json, "removed", |_| format!("plugin {} removed", name))?;
            }
        },
        AuthorCommands::Skill { command } => match command {
            SkillCommands::Create {
                plugin,
                name,
                rack_dir,
                description,
            } => {
                skill_create(rack_dir, plugin, name, description)?;
                print_one(cli.json, "created", |_| {
                    format!("skill {}/{} created", plugin, name)
                })?;
            }
            SkillCommands::Remove {
                plugin,
                name,
                rack_dir,
            } => {
                skill_remove(rack_dir, plugin, name)?;
                print_one(cli.json, "removed", |_| {
                    format!("skill {}/{} removed", plugin, name)
                })?;
            }
        },
        AuthorCommands::Subagent { command } => match command {
            SubagentCommands::Create {
                plugin,
                name,
                rack_dir,
                purpose,
            } => {
                subagent_create(rack_dir, plugin, name, purpose)?;
                print_one(cli.json, "created", |_| {
                    format!("subagent {}/{} created", plugin, name)
                })?;
            }
            SubagentCommands::Remove {
                plugin,
                name,
                rack_dir,
            } => {
                subagent_remove(rack_dir, plugin, name)?;
                print_one(cli.json, "removed", |_| {
                    format!("subagent {}/{} removed", plugin, name)
                })?;
            }
        },
        AuthorCommands::Hook { command } => match command {
            HookCommandsAdmin::List { .. } => {
                anyhow::bail!("use `pater hook list` for listing hooks");
            }
            HookCommandsAdmin::Create {
                plugin,
                rack_dir,
                agent,
                event,
                run,
            } => {
                hook_create(rack_dir, plugin, agent, event, run)?;
                print_one(cli.json, "created", |_| {
                    format!("hook created for {}", plugin)
                })?;
            }
            HookCommandsAdmin::Remove {
                plugin,
                rack_dir,
                agent,
                event,
            } => {
                hook_remove(rack_dir, plugin, agent, event)?;
                print_one(cli.json, "removed", |_| {
                    format!("hook removed for {}", plugin)
                })?;
            }
        },
        AuthorCommands::Mcp { command } => match command {
            McpCommands::Create {
                plugin,
                name,
                rack_dir,
                command,
            } => {
                mcp_create(rack_dir, plugin, name, command)?;
                print_one(cli.json, "created", |_| {
                    format!("mcp {} added to {}", name, plugin)
                })?;
            }
            McpCommands::Remove {
                plugin,
                name,
                rack_dir,
            } => {
                mcp_remove(rack_dir, plugin, name)?;
                print_one(cli.json, "removed", |_| {
                    format!("mcp {} removed from {}", name, plugin)
                })?;
            }
        },
    }

    Ok(true)
}

fn handle_runtime_commands(
    cli: &Cli,
    state: &mut State,
    policy: &PolicyFile,
    all_markets: &[MarketRef],
    default_market: &rack::Marketplace,
) -> anyhow::Result<()> {
    match &cli.command {
        Commands::Search { query } => {
            let items = discover_across(all_markets, query.as_deref(), policy)?;
            print_out(cli.json, &items, |p| {
                format!("{}\t{}\t{}", p.marketplace, p.name, p.description)
            })?;
        }
        Commands::Recommend { context } => {
            let items = discover_across(all_markets, context.as_deref(), policy)?;
            let recs = recommend_plugins(items, context.as_deref());
            print_out(cli.json, &recs, |r| {
                format!("{}\t{}\t{}", r.marketplace, r.plugin, r.reason)
            })?;
        }
        Commands::Plan { intent, agent } => {
            let items = discover_across(all_markets, Some(intent), policy)?;
            let recs = recommend_plugins(items, Some(intent));
            let report = PlanReport {
                intent: intent.clone(),
                agent: format!("{:?}", agent).to_lowercase(),
                recommendations: recs,
            };
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&JsonOut {
                        ok: true,
                        data: report
                    })?
                );
            } else {
                println!("plan for {}", report.agent);
                for r in report.recommendations {
                    println!("{}\t{}\t{}", r.marketplace, r.plugin, r.reason);
                }
            }
        }
        Commands::Show { plugin } => {
            let (name, market) = parse_target(plugin);
            let p = show_plugin(all_markets, &name, market.as_deref(), policy)?;
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&JsonOut { ok: true, data: p })?
                );
            } else {
                println!("marketplace: {}", p.marketplace);
                println!("name: {}", p.name);
                println!(
                    "version: {}",
                    p.version.unwrap_or_else(|| "n/a".to_string())
                );
                println!("description: {}", p.description);
                if !p.permissions.is_empty() {
                    println!("permissions: {}", p.permissions.join(", "));
                }
            }
        }
        Commands::Install { target, scope } => {
            let (name, market) = parse_target(target);
            let p = show_plugin(all_markets, &name, market.as_deref(), policy)?;
            enforce_policy_for_plugin(policy, &p)?;
            let source_path = rack::resolve_plugin_path(&p.marketplace_source, &p.source)?;
            let local_path = materialize_plugin(&p.name, &source_path)?;
            let entry = InstalledPlugin {
                name: p.name.clone(),
                marketplace: p.marketplace.clone(),
                marketplace_source: p.marketplace_source.clone(),
                source: p.source.clone(),
                local_path: local_path.to_string_lossy().to_string(),
                version: p.version.clone(),
                permissions: p.permissions.clone(),
                scope: scope.clone(),
            };
            upsert_installed(state, entry.clone());
            audit(
                "install",
                serde_json::json!({"plugin": entry.name, "marketplace": entry.marketplace}),
            );
            save_state(state)?;
            save_lockfile(state)?;
            sync_installed(state, AdapterTarget::All)?;

            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&JsonOut {
                        ok: true,
                        data: entry
                    })?
                );
            } else {
                println!("installed {}@{}", entry.name, entry.marketplace);
                println!("adapter sync complete (claude/codex/openclaw)");
            }
        }
        Commands::Apply {
            target,
            target_adapter,
            scope,
        } => {
            let (name, market) = parse_target(target);
            let p = show_plugin(all_markets, &name, market.as_deref(), policy)?;
            enforce_policy_for_plugin(policy, &p)?;
            let source_path = rack::resolve_plugin_path(&p.marketplace_source, &p.source)?;
            let local_path = materialize_plugin(&p.name, &source_path)?;
            let entry = InstalledPlugin {
                name: p.name.clone(),
                marketplace: p.marketplace.clone(),
                marketplace_source: p.marketplace_source.clone(),
                source: p.source.clone(),
                local_path: local_path.to_string_lossy().to_string(),
                version: p.version.clone(),
                permissions: p.permissions.clone(),
                scope: scope.clone(),
            };
            upsert_installed(state, entry.clone());
            save_state(state)?;
            save_lockfile(state)?;
            sync_installed(state, target_adapter.clone())?;
            let smoke = adapter_smoke(state, target_adapter.clone())?;
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&JsonOut {
                        ok: true,
                        data: serde_json::json!({"installed": entry, "smoke": smoke})
                    })?
                );
            } else {
                println!("applied {} and synced adapters", entry.name);
            }
        }
        Commands::Adapter { command } => match command {
            AdapterCommands::Sync { target } => {
                sync_installed(state, target.clone())?;
                audit(
                    "adapter_sync",
                    serde_json::json!({"target": format!("{:?}", target)}),
                );
                if cli.json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&JsonOut {
                            ok: true,
                            data: "synced"
                        })?
                    );
                } else {
                    println!("adapter sync completed");
                }
            }
            AdapterCommands::Smoke { target } => {
                let report = adapter_smoke(state, target.clone())?;
                if cli.json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&JsonOut {
                            ok: true,
                            data: report
                        })?
                    );
                } else {
                    for r in report {
                        println!("{}\t{}", r.adapter, r.status);
                    }
                }
            }
            AdapterCommands::Doctor => {
                let report = adapter_doctor(state)?;
                audit(
                    "adapter_doctor",
                    serde_json::json!({"overall": report.overall}),
                );
                if cli.json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&JsonOut {
                            ok: true,
                            data: report
                        })?
                    );
                } else {
                    println!("overall: {}", report.overall);
                    println!("path_has_local_bin: {}", report.path_has_local_bin);
                    for r in report.smoke {
                        println!("{}\t{}", r.adapter, r.status);
                    }
                    for c in report.configs {
                        println!("config:{}\t{}", c.name, c.status);
                    }
                    for w in report.wrappers {
                        println!("wrapper:{}\t{}", w.name, w.status);
                    }
                }
            }
        },
        Commands::Update {
            plugin,
            allow_permission_increase,
        } => {
            let report = update_plugins(
                state,
                all_markets,
                plugin.as_deref(),
                *allow_permission_increase,
                policy,
            )?;
            audit(
                "update",
                serde_json::json!({"plugin": plugin, "count": report.len()}),
            );
            save_state(state)?;
            save_lockfile(state)?;
            sync_installed(state, AdapterTarget::All)?;
            print_out(cli.json, &report, |r| format!("{}\t{}", r.name, r.status))?;
        }
        Commands::Remove { plugin } => {
            let before = state.installed.len();
            state.installed.retain(|p| p.name != *plugin);
            audit("remove", serde_json::json!({"plugin": plugin}));
            save_state(state)?;
            save_lockfile(state)?;
            sync_installed(state, AdapterTarget::All)?;
            let removed = before.saturating_sub(state.installed.len());
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&JsonOut {
                        ok: true,
                        data: removed
                    })?
                );
            } else {
                println!("removed {} entries", removed);
            }
        }
        Commands::List => {
            print_out(cli.json, &state.installed, |p| {
                format!("{}\t{}\t{:?}", p.name, p.marketplace, p.scope)
            })?;
        }
        Commands::Capabilities { agent } => {
            let smoke = adapter_smoke(state, agent.clone())?;
            let report = CapabilitiesReport {
                installed_count: state.installed.len(),
                installed_plugins: state.installed.iter().map(|p| p.name.clone()).collect(),
                adapter_smoke: smoke,
            };
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&JsonOut {
                        ok: true,
                        data: report
                    })?
                );
            } else {
                println!("installed: {}", report.installed_count);
                for p in report.installed_plugins {
                    println!("- {}", p);
                }
            }
        }
        Commands::Hook { command } => match command {
            HookCommands::List { agent } => {
                let hooks = rack::list_hooks(default_market, agent.as_deref());
                if cli.json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&JsonOut {
                            ok: true,
                            data: hooks
                        })?
                    );
                } else {
                    for h in hooks {
                        println!("{}\t{}\t{}", h.plugin_name, h.agent, h.event);
                    }
                }
            }
        },
        Commands::Validate => {
            rack::validate(default_market)?;
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&JsonOut {
                        ok: true,
                        data: "valid"
                    })?
                );
            } else {
                println!("marketplace valid");
            }
        }
        Commands::Remote { command } => match command {
            RemoteCommands::Add { source } => {
                let m = checked_load_marketplace(source, policy)?;
                let mr = MarketRef {
                    name: m.name,
                    source: source.clone(),
                };
                if !state.marketplaces.iter().any(|x| x.name == mr.name) {
                    state.marketplaces.push(mr.clone());
                    save_state(state)?;
                }
                print_one(cli.json, mr, |m| format!("added {}", m.name))?;
            }
            RemoteCommands::List => {
                print_out(cli.json, &state.marketplaces, |m| {
                    format!("{}\t{}", m.name, m.source)
                })?;
            }
            RemoteCommands::Update => {
                let mut checked = 0usize;
                for m in &state.marketplaces {
                    rack::refresh_marketplace(&m.source)?;
                    let _ = checked_load_marketplace(&m.source, policy)?;
                    checked += 1;
                }
                if cli.json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&JsonOut {
                            ok: true,
                            data: checked
                        })?
                    );
                } else {
                    println!("updated {} marketplaces", checked);
                }
            }
        },
        Commands::Ensure { intent, agent } => {
            let items = discover_across(all_markets, Some(intent), policy)?;
            let recs = recommend_plugins(items, Some(intent));
            let top = recs
                .first()
                .ok_or_else(|| anyhow::anyhow!("no plugin recommendation for intent"))?;
            let target = format!("{}@{}", top.plugin, top.marketplace);
            let (name, market) = parse_target(&target);
            let p = show_plugin(all_markets, &name, market.as_deref(), policy)?;
            enforce_policy_for_plugin(policy, &p)?;
            let source_path = rack::resolve_plugin_path(&p.marketplace_source, &p.source)?;
            let local_path = materialize_plugin(&p.name, &source_path)?;
            let entry = InstalledPlugin {
                name: p.name.clone(),
                marketplace: p.marketplace.clone(),
                marketplace_source: p.marketplace_source.clone(),
                source: p.source.clone(),
                local_path: local_path.to_string_lossy().to_string(),
                version: p.version.clone(),
                permissions: p.permissions.clone(),
                scope: InstallScope::User,
            };
            upsert_installed(state, entry.clone());
            save_state(state)?;
            save_lockfile(state)?;
            sync_installed(state, agent.clone())?;
            let smoke = adapter_smoke(state, agent.clone())?;
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&JsonOut {
                        ok: true,
                        data: serde_json::json!({"intent": intent, "selected": entry, "smoke": smoke})
                    })?
                );
            } else {
                println!("ensured capability for intent: {}", intent);
                println!("selected: {}", entry.name);
            }
        }
        Commands::Check => {
            let trust = TrustStatus {
                require_signed_marketplace: policy.general.require_signed_marketplace,
                trusted_key_count: list_pubkeys()?.len(),
                default_marketplace: DEFAULT_MARKETPLACE_SOURCE.to_string(),
                default_marketplace_signature_ok: verify_marketplace_signature(
                    DEFAULT_MARKETPLACE_SOURCE,
                )
                .unwrap_or(false),
            };
            let doctor = adapter_doctor(state)?;
            let rack_license_audit = run_rack_license_audit(&cli.marketplace);
            let report = build_release_check_report(trust, doctor, rack_license_audit);
            print_one(cli.json, report, |r| {
                format!("release-check: {}", r.overall)
            })?;
        }
        Commands::Policy { command } => match command {
            PolicyCommands::Eval { plugin, agent } => {
                let (name, market) = parse_target(plugin);
                let p = show_plugin(all_markets, &name, market.as_deref(), policy)?;
                let eval = policy_eval_for_plugin(policy, &p, agent.clone());
                if cli.json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&JsonOut {
                            ok: true,
                            data: eval
                        })?
                    );
                } else {
                    println!(
                        "{}\t{}",
                        eval.plugin,
                        if eval.allowed { "allowed" } else { "denied" }
                    );
                    println!("reason: {}", eval.reason);
                }
            }
        },
        Commands::Trust { .. } | Commands::Rack { .. } | Commands::Author { .. } => {
            unreachable!("handled before marketplace loading")
        }
    }

    Ok(())
}

fn run(cli: Cli) -> anyhow::Result<()> {
    let mut state = load_state()?;
    let policy = load_policy()?;

    ensure_default_marketplace(&mut state)?;

    if handle_trust_commands(&cli, &policy)? {
        return Ok(());
    }

    if handle_rack_commands(&cli)? {
        return Ok(());
    }

    if handle_author_commands(&cli)? {
        return Ok(());
    }

    for m in &state.marketplaces {
        let _ = rack::refresh_marketplace(&m.source);
    }
    let _ = rack::refresh_marketplace(&cli.marketplace);

    let default_market = checked_load_marketplace(&cli.marketplace, &policy)?;
    let mut all_markets = vec![MarketRef {
        name: default_market.name.clone(),
        source: cli.marketplace.clone(),
    }];
    all_markets.extend(state.marketplaces.clone());
    dedupe_markets(&mut all_markets);

    handle_runtime_commands(&cli, &mut state, &policy, &all_markets, &default_market)?;

    Ok(())
}

#[derive(Serialize, Clone)]
struct DiscoverItem {
    marketplace: String,
    marketplace_source: String,
    name: String,
    description: String,
    version: Option<String>,
    source: String,
    distribution: Option<String>,
    license_status: Option<String>,
    permissions: Vec<String>,
}

#[derive(Serialize, Clone)]
struct Recommendation {
    plugin: String,
    marketplace: String,
    score: i32,
    reason: String,
    permission_count: usize,
    distribution: Option<String>,
    license_status: Option<String>,
}

#[derive(Serialize)]
struct UpdateReport {
    name: String,
    status: String,
    old_version: Option<String>,
    new_version: Option<String>,
    added_permissions: Vec<String>,
}

#[derive(Serialize)]
pub struct SmokeReport {
    pub adapter: String,
    pub status: String,
    pub checked_plugins: usize,
    pub missing_plugins: Vec<String>,
}

#[derive(Serialize)]
pub struct CheckItem {
    pub name: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct DoctorReport {
    pub overall: String,
    pub path_has_local_bin: bool,
    pub smoke: Vec<SmokeReport>,
    pub configs: Vec<CheckItem>,
    pub wrappers: Vec<CheckItem>,
}

#[derive(Serialize)]
struct CapabilitiesReport {
    installed_count: usize,
    installed_plugins: Vec<String>,
    adapter_smoke: Vec<SmokeReport>,
}

#[derive(Serialize)]
struct PolicyEvalReport {
    plugin: String,
    agent: String,
    allowed: bool,
    reason: String,
}

#[derive(Serialize)]
struct PlanReport {
    intent: String,
    agent: String,
    recommendations: Vec<Recommendation>,
}

#[derive(Serialize)]
pub struct TrustStatus {
    pub require_signed_marketplace: bool,
    pub trusted_key_count: usize,
    pub default_marketplace: String,
    pub default_marketplace_signature_ok: bool,
}

#[derive(Serialize)]
pub struct ReleaseCheckReport {
    pub overall: String,
    pub trust: TrustStatus,
    pub doctor: DoctorReport,
    pub rack_license_audit: String,
    pub recommendations: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RackLicenseAuditSummary {
    pub permissive: usize,
    pub copyleft: usize,
    pub unknown_count: usize,
    pub total: usize,
}

#[derive(Serialize)]
pub struct RackDoctorReport {
    pub overall: String,
    pub checks: Vec<CheckItem>,
}

fn update_plugins(
    state: &mut State,
    markets: &[MarketRef],
    only: Option<&str>,
    allow_permission_increase: bool,
    policy: &PolicyFile,
) -> anyhow::Result<Vec<UpdateReport>> {
    let mut reports = Vec::new();
    for installed in &mut state.installed {
        if only.map(|o| o != installed.name).unwrap_or(false) {
            continue;
        }
        let latest = show_plugin(
            markets,
            &installed.name,
            Some(&installed.marketplace),
            policy,
        )?;
        enforce_policy_for_plugin(policy, &latest)?;
        let old_permissions: HashSet<_> = installed.permissions.iter().cloned().collect();
        let new_permissions: HashSet<_> = latest.permissions.iter().cloned().collect();
        let added_permissions: Vec<String> = new_permissions
            .difference(&old_permissions)
            .cloned()
            .collect();

        if !added_permissions.is_empty() && !allow_permission_increase {
            reports.push(UpdateReport {
                name: installed.name.clone(),
                status: "blocked_permission_increase".to_string(),
                old_version: installed.version.clone(),
                new_version: latest.version.clone(),
                added_permissions,
            });
            continue;
        }

        let changed =
            installed.version != latest.version || installed.permissions != latest.permissions;
        if changed {
            let report = UpdateReport {
                name: installed.name.clone(),
                status: "updated".to_string(),
                old_version: installed.version.clone(),
                new_version: latest.version.clone(),
                added_permissions,
            };
            installed.version = latest.version.clone();
            installed.permissions = latest.permissions.clone();
            installed.source = latest.source.clone();
            installed.marketplace_source = latest.marketplace_source.clone();
            if let Ok(src) = rack::resolve_plugin_path(&latest.marketplace_source, &latest.source) {
                if let Ok(p) = materialize_plugin(&installed.name, &src) {
                    installed.local_path = p.to_string_lossy().to_string();
                }
            }
            reports.push(report);
        } else {
            reports.push(UpdateReport {
                name: installed.name.clone(),
                status: "up_to_date".to_string(),
                old_version: installed.version.clone(),
                new_version: latest.version.clone(),
                added_permissions: vec![],
            });
        }
    }
    Ok(reports)
}

fn ensure_default_marketplace(state: &mut State) -> anyhow::Result<()> {
    if !state
        .marketplaces
        .iter()
        .any(|m| m.source == DEFAULT_MARKETPLACE_SOURCE)
    {
        state.marketplaces.push(MarketRef {
            name: "paternoster-rack".to_string(),
            source: DEFAULT_MARKETPLACE_SOURCE.to_string(),
        });
        save_state(state)?;
    }
    Ok(())
}

fn dedupe_markets(markets: &mut Vec<MarketRef>) {
    let mut seen = HashSet::new();
    markets.retain(|m| seen.insert(format!("{}::{}", m.name, m.source)));
}

fn discover_across(
    markets: &[MarketRef],
    query: Option<&str>,
    policy: &PolicyFile,
) -> anyhow::Result<Vec<DiscoverItem>> {
    let mut out = Vec::new();
    for m in markets {
        let loaded = checked_load_marketplace(&m.source, policy)?;
        for p in rack::discover(&loaded, query) {
            out.push(DiscoverItem {
                marketplace: loaded.name.clone(),
                marketplace_source: m.source.clone(),
                name: p.name.clone(),
                description: p.description.clone().unwrap_or_default(),
                version: p.version.clone(),
                source: p.source.clone(),
                distribution: p.distribution.clone(),
                license_status: p.license_status.clone(),
                permissions: p.permissions.clone(),
            });
        }
    }
    Ok(out)
}

fn recommend_plugins(items: Vec<DiscoverItem>, context: Option<&str>) -> Vec<Recommendation> {
    let ctx = context.unwrap_or("").to_ascii_lowercase();
    let mut out: Vec<Recommendation> = items
        .into_iter()
        .map(|p| {
            let mut score = 0;
            let mut reason = String::from("baseline relevance");
            if !ctx.is_empty() {
                if p.name.to_ascii_lowercase().contains(&ctx) {
                    score += 50;
                    reason = "name matches context".to_string();
                }
                if p.description.to_ascii_lowercase().contains(&ctx) {
                    score += 30;
                    reason = "description matches context".to_string();
                }
                for perm in &p.permissions {
                    if ctx.contains(&perm.to_ascii_lowercase()) {
                        score += 10;
                    }
                }
            }
            if p.distribution.as_deref() == Some("external-reference-only") {
                score -= 20;
            }
            Recommendation {
                plugin: p.name,
                marketplace: p.marketplace,
                score,
                reason,
                permission_count: p.permissions.len(),
                distribution: p.distribution,
                license_status: p.license_status,
            }
        })
        .collect();
    out.sort_by(|a, b| b.score.cmp(&a.score).then(a.plugin.cmp(&b.plugin)));
    out.truncate(10);
    out
}

fn show_plugin(
    markets: &[MarketRef],
    name: &str,
    marketplace: Option<&str>,
    policy: &PolicyFile,
) -> anyhow::Result<DiscoverItem> {
    for m in markets {
        let loaded = checked_load_marketplace(&m.source, policy)?;
        if let Some(filter) = marketplace {
            if loaded.name != filter {
                continue;
            }
        }
        if let Ok(p) = rack::show(&loaded, name) {
            return Ok(DiscoverItem {
                marketplace: loaded.name.clone(),
                marketplace_source: m.source.clone(),
                name: p.name.clone(),
                description: p.description.clone().unwrap_or_default(),
                version: p.version.clone(),
                source: p.source.clone(),
                distribution: p.distribution.clone(),
                license_status: p.license_status.clone(),
                permissions: p.permissions.clone(),
            });
        }
    }
    anyhow::bail!("plugin not found: {}", name)
}

fn load_policy() -> anyhow::Result<PolicyFile> {
    let home = std::env::var("HOME")?;
    let path = PathBuf::from(home).join(".config/pater/policy.toml");
    if !path.exists() {
        return Ok(PolicyFile {
            general: PolicyGeneral::default(),
        });
    }
    let raw = std::fs::read_to_string(path)?;
    Ok(toml::from_str(&raw)?)
}

fn checked_load_marketplace(
    source: &str,
    policy: &PolicyFile,
) -> anyhow::Result<rack::Marketplace> {
    if policy.general.require_signed_marketplace {
        let ok = verify_marketplace_signature(source)?;
        if !ok {
            anyhow::bail!("marketplace signature verification failed: {}", source);
        }
    }
    rack::load_marketplace(source)
}

fn policy_eval_for_plugin(
    policy: &PolicyFile,
    p: &DiscoverItem,
    agent: AdapterTarget,
) -> PolicyEvalReport {
    match enforce_policy_for_plugin(policy, p) {
        Ok(_) => PolicyEvalReport {
            plugin: p.name.clone(),
            agent: format!("{:?}", agent).to_lowercase(),
            allowed: true,
            reason: "allowed".to_string(),
        },
        Err(e) => PolicyEvalReport {
            plugin: p.name.clone(),
            agent: format!("{:?}", agent).to_lowercase(),
            allowed: false,
            reason: e.to_string(),
        },
    }
}

fn enforce_policy_for_plugin(policy: &PolicyFile, p: &DiscoverItem) -> anyhow::Result<()> {
    if policy.general.denied_plugins.iter().any(|x| x == &p.name) {
        anyhow::bail!("policy denied plugin: {}", p.name);
    }
    if !policy.general.allowed_sources.is_empty()
        && !policy
            .general
            .allowed_sources
            .iter()
            .any(|s| source_matches_allowed(&p.marketplace_source, s))
    {
        anyhow::bail!("policy denied source: {}", p.marketplace_source);
    }
    if p.permissions
        .iter()
        .any(|perm| policy.general.blocked_permissions.contains(perm))
    {
        anyhow::bail!("policy blocked permission in plugin: {}", p.name);
    }

    if p.distribution.as_deref() == Some("external-reference-only")
        && !policy.general.allow_external_reference_installs
        && !policy
            .general
            .allow_external_reference_plugins
            .iter()
            .any(|x| x == &p.name)
    {
        anyhow::bail!(
            "policy blocked external-reference-only plugin: {} (set allow_external_reference_installs=true or add plugin to allow_external_reference_plugins)",
            p.name
        );
    }

    if policy.general.block_unknown_licenses {
        let cls = classify_plugin_license(p)?;
        if cls == "unknown"
            && !policy
                .general
                .allow_unknown_license_plugins
                .iter()
                .any(|x| x == &p.name)
        {
            anyhow::bail!(
                "policy blocked unknown-license plugin: {} (add to allow_unknown_license_plugins to override)",
                p.name
            );
        }
    }

    Ok(())
}

fn normalize_license_token(raw: &str) -> String {
    raw.trim().to_ascii_uppercase().replace(' ', "-")
}

fn classify_plugin_license(p: &DiscoverItem) -> anyhow::Result<&'static str> {
    let src = rack::resolve_plugin_path(&p.marketplace_source, &p.source)?;
    let mut tokens = Vec::new();

    for name in ["LICENSE", "LICENSE.md", "COPYING"] {
        let lp = src.join(name);
        if lp.exists() {
            let text = std::fs::read_to_string(lp)
                .unwrap_or_default()
                .to_ascii_uppercase();
            tokens.push(text);
        }
    }

    let manifest = src.join(".claude-plugin").join("plugin.json");
    if manifest.exists() {
        let raw = std::fs::read_to_string(manifest).unwrap_or_default();
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
            if let Some(l) = v.get("license").and_then(|x| x.as_str()) {
                tokens.push(normalize_license_token(l));
            }
        }
    }

    let joined = tokens.join("\n");
    if joined.is_empty() {
        return Ok("unknown");
    }

    let permissive_needles = ["MIT", "APACHE", "BSD", "ISC", "UNLICENSE", "CC0"];
    if permissive_needles.iter().any(|n| joined.contains(n)) {
        return Ok("permissive");
    }

    let copyleft_needles = ["GPL", "AGPL", "LGPL", "MPL"];
    if copyleft_needles.iter().any(|n| joined.contains(n)) {
        return Ok("copyleft");
    }

    Ok("unknown")
}

fn run_rack_license_audit(marketplace_source: &str) -> String {
    let path = PathBuf::from(marketplace_source);
    if !path.exists() {
        return "not_applicable".to_string();
    }

    let rack_dir = if path.is_dir() {
        path
    } else if let Some(parent) = path.parent() {
        parent.to_path_buf()
    } else {
        PathBuf::from(".")
    };

    match rack_license_audit_readonly(&rack_dir) {
        Ok(r) if r.unknown_count == 0 => "ok".to_string(),
        Ok(_) => "failed".to_string(),
        Err(_) => "error".to_string(),
    }
}

fn parse_target(target: &str) -> (String, Option<String>) {
    let parts: Vec<&str> = target.split('@').collect();
    if parts.len() == 2 {
        (parts[0].to_string(), Some(parts[1].to_string()))
    } else {
        (target.to_string(), None)
    }
}

fn print_out<T: Serialize>(
    json: bool,
    data: &[T],
    row: impl Fn(&T) -> String,
) -> anyhow::Result<()> {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&JsonOut { ok: true, data })?
        );
    } else {
        for d in data {
            println!("{}", row(d));
        }
    }
    Ok(())
}

fn print_one<T: Serialize>(json: bool, data: T, row: impl Fn(&T) -> String) -> anyhow::Result<()> {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&JsonOut { ok: true, data })?
        );
    } else {
        println!("{}", row(&data));
    }
    Ok(())
}
