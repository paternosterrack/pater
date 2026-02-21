use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

mod rack;

const DEFAULT_MARKETPLACE_SOURCE: &str = "paternosterrack/rack";
const OFFICIAL_RACK_PUBKEY_HEX: &str =
    "5aefcc2a6716ef9fab24dc3865013e29a8d579e4dda33bf753a7cd7a8d14450a";

#[derive(Parser, Debug)]
#[command(name = "pater", version, about = "Paternoster Rack CLI")]
struct Cli {
    #[arg(long, global = true, help = "Output machine-readable JSON")]
    json: bool,
    #[arg(
        long,
        global = true,
        default_value = DEFAULT_MARKETPLACE_SOURCE,
        help = "Default marketplace source (dir, marketplace.json, url, or owner/repo)"
    )]
    marketplace: String,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Search {
        query: Option<String>,
    },
    Recommend {
        #[arg(long)]
        context: Option<String>,
    },
    Plan {
        #[arg(long)]
        intent: String,
        #[arg(long, value_enum, default_value_t = AdapterTarget::All)]
        agent: AdapterTarget,
    },
    Show {
        plugin: String,
    },
    Install {
        target: String,
        #[arg(long, value_enum, default_value_t = InstallScope::User)]
        scope: InstallScope,
    },
    Apply {
        target: String,
        #[arg(long, value_enum, default_value_t = AdapterTarget::All)]
        target_adapter: AdapterTarget,
        #[arg(long, value_enum, default_value_t = InstallScope::User)]
        scope: InstallScope,
    },
    Adapter {
        #[command(subcommand)]
        command: AdapterCommands,
    },
    Update {
        plugin: Option<String>,
        #[arg(long, default_value_t = false)]
        allow_permission_increase: bool,
    },
    Remove {
        plugin: String,
    },
    List,
    Capabilities {
        #[arg(long, value_enum, default_value_t = AdapterTarget::All)]
        agent: AdapterTarget,
    },
    Hook {
        #[command(subcommand)]
        command: HookCommands,
    },
    Validate,
    Remote {
        #[command(subcommand)]
        command: RemoteCommands,
    },
    Trust {
        #[command(subcommand)]
        command: TrustCommands,
    },
    Policy {
        #[command(subcommand)]
        command: PolicyCommands,
    },
    Rack {
        #[command(subcommand)]
        command: RackCommands,
    },
    Author {
        #[command(subcommand)]
        command: AuthorCommands,
    },
    Ensure {
        #[arg(long)]
        intent: String,
        #[arg(long, value_enum, default_value_t = AdapterTarget::All)]
        agent: AdapterTarget,
    },
    Check,
}

#[derive(Subcommand, Debug)]
enum RemoteCommands {
    Add { source: String },
    List,
    Update,
}

#[derive(Subcommand, Debug)]
enum HookCommands {
    List {
        #[arg(long)]
        agent: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum TrustCommands {
    Init,
    List,
    Status,
}

#[derive(Subcommand, Debug)]
enum PolicyCommands {
    Eval {
        plugin: String,
        #[arg(long, value_enum, default_value_t = AdapterTarget::All)]
        agent: AdapterTarget,
    },
}

#[derive(Subcommand, Debug)]
enum RackCommands {
    Doctor {
        #[arg(long, default_value = "../rack")]
        rack_dir: String,
        #[arg(long)]
        sign_key: Option<String>,
    },
    Sync {
        #[arg(long, default_value = "../rack")]
        rack_dir: String,
    },
    MarkUnknownExternal {
        #[arg(long, default_value = "../rack")]
        rack_dir: String,
    },
    LicenseAudit {
        #[arg(long, default_value = "../rack")]
        rack_dir: String,
    },
    Sign {
        #[arg(long, default_value = "../rack")]
        rack_dir: String,
        #[arg(long)]
        sign_key: String,
    },
    PrepareRelease {
        #[arg(long, default_value = "../rack")]
        rack_dir: String,
        #[arg(long)]
        sign_key: String,
    },
}

#[derive(Subcommand, Debug)]
enum AuthorCommands {
    Plugin {
        #[command(subcommand)]
        command: PluginCommands,
    },
    Skill {
        #[command(subcommand)]
        command: SkillCommands,
    },
    Subagent {
        #[command(subcommand)]
        command: SubagentCommands,
    },
    Hook {
        #[command(subcommand)]
        command: HookCommandsAdmin,
    },
    Mcp {
        #[command(subcommand)]
        command: McpCommands,
    },
}

#[derive(Subcommand, Debug)]
enum PluginCommands {
    Create {
        name: String,
        #[arg(long, default_value = "../rack")]
        rack_dir: String,
        #[arg(long, default_value = "New plugin")]
        description: String,
    },
    Update {
        name: String,
        #[arg(long, default_value = "../rack")]
        rack_dir: String,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        version: Option<String>,
    },
    Remove {
        name: String,
        #[arg(long, default_value = "../rack")]
        rack_dir: String,
    },
}

#[derive(Subcommand, Debug)]
enum SkillCommands {
    Create {
        plugin: String,
        name: String,
        #[arg(long, default_value = "../rack")]
        rack_dir: String,
        #[arg(long, default_value = "Skill description")]
        description: String,
    },
    Remove {
        plugin: String,
        name: String,
        #[arg(long, default_value = "../rack")]
        rack_dir: String,
    },
}

#[derive(Subcommand, Debug)]
enum SubagentCommands {
    Create {
        plugin: String,
        name: String,
        #[arg(long, default_value = "../rack")]
        rack_dir: String,
        #[arg(long, default_value = "Subagent purpose")]
        purpose: String,
    },
    Remove {
        plugin: String,
        name: String,
        #[arg(long, default_value = "../rack")]
        rack_dir: String,
    },
}

#[derive(Subcommand, Debug)]
enum HookCommandsAdmin {
    List {
        #[arg(long)]
        agent: Option<String>,
    },
    Create {
        plugin: String,
        #[arg(long, default_value = "../rack")]
        rack_dir: String,
        #[arg(long)]
        agent: String,
        #[arg(long)]
        event: String,
        #[arg(long)]
        run: String,
    },
    Remove {
        plugin: String,
        #[arg(long, default_value = "../rack")]
        rack_dir: String,
        #[arg(long)]
        agent: String,
        #[arg(long)]
        event: String,
    },
}

#[derive(Subcommand, Debug)]
enum McpCommands {
    Create {
        plugin: String,
        name: String,
        #[arg(long, default_value = "../rack")]
        rack_dir: String,
        #[arg(long)]
        command: String,
    },
    Remove {
        plugin: String,
        name: String,
        #[arg(long, default_value = "../rack")]
        rack_dir: String,
    },
}

#[derive(Subcommand, Debug)]
enum AdapterCommands {
    Sync {
        #[arg(long, value_enum, default_value_t = AdapterTarget::All)]
        target: AdapterTarget,
    },
    Smoke {
        #[arg(long, value_enum, default_value_t = AdapterTarget::All)]
        target: AdapterTarget,
    },
    Doctor,
}

#[derive(Clone, Debug, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
enum AdapterTarget {
    All,
    Claude,
    Codex,
    Openclaw,
}

#[derive(Clone, Debug, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
enum InstallScope {
    User,
    Project,
    Local,
}

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

fn run(cli: Cli) -> anyhow::Result<()> {
    let mut state = load_state()?;
    let policy = load_policy()?;

    ensure_default_marketplace(&mut state)?;

    if let Commands::Trust { command } = &cli.command {
        match command {
            TrustCommands::Init => {
                trust_init()?;
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
                let sig_ok =
                    verify_marketplace_signature(DEFAULT_MARKETPLACE_SOURCE).unwrap_or(false);
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
        return Ok(());
    }

    if let Commands::Rack { command } = &cli.command {
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
                        println!("{}\t{}", c.name, c.status);
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
                        "license audit: permissive={} copyleft={} unknown={} total={}",
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
        return Ok(());
    }

    if let Commands::Author { command } = &cli.command {
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

    match cli.command {
        Commands::Search { query } => {
            let items = discover_across(&all_markets, query.as_deref(), &policy)?;
            print_out(cli.json, &items, |p| {
                format!("{}\t{}\t{}", p.marketplace, p.name, p.description)
            })?;
        }
        Commands::Recommend { context } => {
            let items = discover_across(&all_markets, context.as_deref(), &policy)?;
            let recs = recommend_plugins(items, context.as_deref());
            print_out(cli.json, &recs, |r| {
                format!("{}\t{}\t{}", r.marketplace, r.plugin, r.reason)
            })?;
        }
        Commands::Plan { intent, agent } => {
            let items = discover_across(&all_markets, Some(&intent), &policy)?;
            let recs = recommend_plugins(items, Some(&intent));
            let report = PlanReport {
                intent,
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
            let (name, market) = parse_target(&plugin);
            let p = show_plugin(&all_markets, &name, market.as_deref(), &policy)?;
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
            let (name, market) = parse_target(&target);
            let p = show_plugin(&all_markets, &name, market.as_deref(), &policy)?;
            enforce_policy_for_plugin(&policy, &p)?;
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
                scope,
            };
            upsert_installed(&mut state, entry.clone());
            audit(
                "install",
                serde_json::json!({"plugin": entry.name, "marketplace": entry.marketplace}),
            );
            save_state(&state)?;
            save_lockfile(&state)?;
            sync_installed(&state, AdapterTarget::All)?;

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
            let (name, market) = parse_target(&target);
            let p = show_plugin(&all_markets, &name, market.as_deref(), &policy)?;
            enforce_policy_for_plugin(&policy, &p)?;
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
                scope,
            };
            upsert_installed(&mut state, entry.clone());
            save_state(&state)?;
            save_lockfile(&state)?;
            sync_installed(&state, target_adapter.clone())?;
            let smoke = adapter_smoke(&state, target_adapter.clone())?;
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
                sync_installed(&state, target.clone())?;
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
                let report = adapter_smoke(&state, target.clone())?;
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
                let report = adapter_doctor(&state)?;
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
                &mut state,
                &all_markets,
                plugin.as_deref(),
                allow_permission_increase,
                &policy,
            )?;
            audit(
                "update",
                serde_json::json!({"plugin": plugin, "count": report.len()}),
            );
            save_state(&state)?;
            save_lockfile(&state)?;
            sync_installed(&state, AdapterTarget::All)?;
            print_out(cli.json, &report, |r| format!("{}\t{}", r.name, r.status))?;
        }
        Commands::Remove { plugin } => {
            let before = state.installed.len();
            state.installed.retain(|p| p.name != plugin);
            audit("remove", serde_json::json!({"plugin": plugin}));
            save_state(&state)?;
            save_lockfile(&state)?;
            sync_installed(&state, AdapterTarget::All)?;
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
            let smoke = adapter_smoke(&state, agent)?;
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
                let hooks = rack::list_hooks(&default_market, agent.as_deref());
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
            rack::validate(&default_market)?;
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
                let m = checked_load_marketplace(&source, &policy)?;
                let mr = MarketRef {
                    name: m.name,
                    source,
                };
                if !state.marketplaces.iter().any(|x| x.name == mr.name) {
                    state.marketplaces.push(mr.clone());
                    save_state(&state)?;
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
                    let _ = checked_load_marketplace(&m.source, &policy)?;
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
            let items = discover_across(&all_markets, Some(&intent), &policy)?;
            let recs = recommend_plugins(items, Some(&intent));
            let top = recs
                .first()
                .ok_or_else(|| anyhow::anyhow!("no plugin recommendation for intent"))?;
            let target = format!("{}@{}", top.plugin, top.marketplace);
            let (name, market) = parse_target(&target);
            let p = show_plugin(&all_markets, &name, market.as_deref(), &policy)?;
            enforce_policy_for_plugin(&policy, &p)?;
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
            upsert_installed(&mut state, entry.clone());
            save_state(&state)?;
            save_lockfile(&state)?;
            sync_installed(&state, agent.clone())?;
            let smoke = adapter_smoke(&state, agent)?;
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
            let doctor = adapter_doctor(&state)?;
            let rack_license_audit = run_rack_license_audit();
            let overall = if trust.default_marketplace_signature_ok
                && doctor.overall == "ok"
                && rack_license_audit == "ok"
            {
                "ok"
            } else {
                "needs_attention"
            }
            .to_string();

            let mut recommendations = Vec::new();
            if !trust.default_marketplace_signature_ok {
                recommendations.push("Run `pater trust init` and ensure marketplace.sig is published for default marketplace.".to_string());
            }
            if doctor.overall != "ok" {
                recommendations.push("Run `pater adapter sync --target all` and `pater adapter doctor` until all adapter checks are ok.".to_string());
            }
            if rack_license_audit != "ok" {
                recommendations.push("Run `pater rack license-audit --rack-dir ../rack` and resolve unknown/proprietary plugins before release.".to_string());
            }
            let report = ReleaseCheckReport {
                overall,
                trust,
                doctor,
                rack_license_audit,
                recommendations,
            };
            print_one(cli.json, report, |r| {
                format!("release-check: {}", r.overall)
            })?;
        }
        Commands::Policy { command } => match command {
            PolicyCommands::Eval { plugin, agent } => {
                let (name, market) = parse_target(&plugin);
                let p = show_plugin(&all_markets, &name, market.as_deref(), &policy)?;
                let eval = policy_eval_for_plugin(&policy, &p, agent.clone());
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
        Commands::Trust { .. } => unreachable!("handled before marketplace loading"),
        Commands::Rack { .. } => unreachable!("handled before marketplace loading"),
        Commands::Author { .. } => unreachable!("handled before marketplace loading"),
    }

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
struct SmokeReport {
    adapter: String,
    status: String,
    checked_plugins: usize,
    missing_plugins: Vec<String>,
}

#[derive(Serialize)]
struct CheckItem {
    name: String,
    status: String,
}

#[derive(Serialize)]
struct DoctorReport {
    overall: String,
    path_has_local_bin: bool,
    smoke: Vec<SmokeReport>,
    configs: Vec<CheckItem>,
    wrappers: Vec<CheckItem>,
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
struct TrustStatus {
    require_signed_marketplace: bool,
    trusted_key_count: usize,
    default_marketplace: String,
    default_marketplace_signature_ok: bool,
}

#[derive(Serialize)]
struct ReleaseCheckReport {
    overall: String,
    trust: TrustStatus,
    doctor: DoctorReport,
    rack_license_audit: String,
    recommendations: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct RackLicenseAuditSummary {
    permissive: usize,
    copyleft: usize,
    unknown_count: usize,
    total: usize,
}

#[derive(Serialize)]
struct RackDoctorReport {
    overall: String,
    checks: Vec<CheckItem>,
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

fn trusted_pubkeys_path() -> anyhow::Result<PathBuf> {
    let home = std::env::var("HOME")?;
    Ok(PathBuf::from(home).join(".config/pater/trust/pubkeys.txt"))
}

fn list_pubkeys() -> anyhow::Result<Vec<String>> {
    let path = trusted_pubkeys_path()?;
    if !path.exists() {
        return Ok(vec![]);
    }
    Ok(std::fs::read_to_string(path)?
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect())
}

fn trust_init() -> anyhow::Result<()> {
    let path = trusted_pubkeys_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut existing = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    if !existing
        .lines()
        .any(|l| l.trim() == OFFICIAL_RACK_PUBKEY_HEX)
    {
        if !existing.is_empty() && !existing.ends_with('\n') {
            existing.push('\n');
        }
        existing.push_str(OFFICIAL_RACK_PUBKEY_HEX);
        existing.push('\n');
        std::fs::write(path, existing)?;
    }
    Ok(())
}

fn load_trusted_pubkeys() -> anyhow::Result<Vec<ed25519_dalek::VerifyingKey>> {
    let path = trusted_pubkeys_path()?;
    if !path.exists() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    for line in std::fs::read_to_string(path)?.lines() {
        let l = line.trim();
        if l.is_empty() || l.starts_with('#') {
            continue;
        }
        let bytes = hex::decode(l)?;
        if bytes.len() != 32 {
            continue;
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        if let Ok(k) = ed25519_dalek::VerifyingKey::from_bytes(&arr) {
            out.push(k);
        }
    }
    Ok(out)
}

fn verify_marketplace_signature(source: &str) -> anyhow::Result<bool> {
    let raw = rack::load_marketplace_raw(source)?;
    let sigs_raw = rack::load_marketplace_signature(source)?;
    let keys = load_trusted_pubkeys()?;
    if keys.is_empty() {
        return Ok(false);
    }

    let mut signatures = Vec::new();
    for line in sigs_raw.lines() {
        let s = line.trim();
        if s.is_empty() || s.starts_with('#') {
            continue;
        }
        let sig_bytes = match hex::decode(s) {
            Ok(b) => b,
            Err(_) => continue,
        };
        let sig_arr: [u8; 64] = match sig_bytes.as_slice().try_into() {
            Ok(a) => a,
            Err(_) => continue,
        };
        signatures.push(ed25519_dalek::Signature::from_bytes(&sig_arr));
    }

    for sig in signatures {
        for k in &keys {
            if k.verify_strict(raw.as_bytes(), &sig).is_ok() {
                return Ok(true);
            }
        }
    }
    Ok(false)
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
            .any(|s| p.marketplace_source.starts_with(s))
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

fn rack_doctor(rack_dir: &str, sign_key: Option<&str>) -> RackDoctorReport {
    let root = PathBuf::from(rack_dir);
    let checks = vec![
        CheckItem {
            name: "rack_dir_exists".to_string(),
            status: if root.exists() { "ok" } else { "missing" }.to_string(),
        },
        CheckItem {
            name: "marketplace_json".to_string(),
            status: if root.join(".pater/marketplace.json").exists() {
                "ok"
            } else {
                "missing"
            }
            .to_string(),
        },
        CheckItem {
            name: "upstream_official_snapshot".to_string(),
            status: if root
                .join("_upstreams/claude-plugins-official/.claude-plugin/marketplace.json")
                .exists()
            {
                "ok"
            } else {
                "missing"
            }
            .to_string(),
        },
        CheckItem {
            name: "upstream_claude_code_snapshot".to_string(),
            status: if root
                .join("_upstreams/claude-code/.claude-plugin/marketplace.json")
                .exists()
            {
                "ok"
            } else {
                "missing"
            }
            .to_string(),
        },
        CheckItem {
            name: "upstream_skills_snapshot".to_string(),
            status: if root
                .join("_upstreams/skills/.claude-plugin/marketplace.json")
                .exists()
            {
                "ok"
            } else {
                "missing"
            }
            .to_string(),
        },
        CheckItem {
            name: "sign_key".to_string(),
            status: match sign_key {
                Some(k) if PathBuf::from(k).exists() => "ok".to_string(),
                Some(_) => "missing".to_string(),
                None => "not_provided".to_string(),
            },
        },
        CheckItem {
            name: "openssl_available".to_string(),
            status: if std::process::Command::new("openssl")
                .arg("version")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
            {
                "ok"
            } else {
                "missing"
            }
            .to_string(),
        },
    ];

    let overall = if checks
        .iter()
        .all(|c| c.status == "ok" || c.status == "not_provided")
    {
        "ok"
    } else {
        "needs_attention"
    }
    .to_string();

    RackDoctorReport { overall, checks }
}

fn rack_marketplace_path(rack_dir: &str) -> PathBuf {
    PathBuf::from(rack_dir).join(".pater/marketplace.json")
}

fn load_marketplace_value(rack_dir: &str) -> anyhow::Result<serde_json::Value> {
    let p = rack_marketplace_path(rack_dir);
    Ok(serde_json::from_str(&std::fs::read_to_string(p)?)?)
}

fn save_marketplace_value(rack_dir: &str, v: &serde_json::Value) -> anyhow::Result<()> {
    let p = rack_marketplace_path(rack_dir);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(p, serde_json::to_string_pretty(v)?)?;
    Ok(())
}

fn get_plugin_mut<'a>(
    m: &'a mut serde_json::Value,
    name: &str,
) -> anyhow::Result<&'a mut serde_json::Value> {
    let arr = m
        .get_mut("plugins")
        .and_then(|x| x.as_array_mut())
        .ok_or_else(|| anyhow::anyhow!("missing plugins array"))?;
    arr.iter_mut()
        .find(|p| p.get("name").and_then(|x| x.as_str()) == Some(name))
        .ok_or_else(|| anyhow::anyhow!("plugin not found: {}", name))
}

fn ensure_array_field<'a>(
    obj: &'a mut serde_json::Value,
    field: &str,
) -> anyhow::Result<&'a mut Vec<serde_json::Value>> {
    if obj.get(field).is_none() {
        obj[field] = serde_json::Value::Array(vec![]);
    }
    obj.get_mut(field)
        .and_then(|x| x.as_array_mut())
        .ok_or_else(|| anyhow::anyhow!("invalid array field {}", field))
}

fn plugin_create(rack_dir: &str, name: &str, description: &str) -> anyhow::Result<()> {
    let mut m = load_marketplace_value(rack_dir)?;
    if m.get("plugins").is_none() {
        m["plugins"] = serde_json::Value::Array(vec![]);
    }
    let arr = m.get_mut("plugins").and_then(|x| x.as_array_mut()).unwrap();
    if arr
        .iter()
        .any(|p| p.get("name").and_then(|x| x.as_str()) == Some(name))
    {
        anyhow::bail!("plugin exists: {}", name);
    }
    arr.push(serde_json::json!({"name": name, "source": format!("./plugins/{}", name), "description": description, "version": "0.1.0", "skills": [], "hooks": [], "subagents": []}));
    save_marketplace_value(rack_dir, &m)?;

    let base = PathBuf::from(rack_dir).join("plugins").join(name);
    std::fs::create_dir_all(base.join(".claude-plugin"))?;
    std::fs::create_dir_all(base.join("skills"))?;
    std::fs::write(
        base.join(".claude-plugin/plugin.json"),
        serde_json::to_string_pretty(
            &serde_json::json!({"name": name, "description": description, "version": "0.1.0"}),
        )?,
    )?;
    Ok(())
}

fn plugin_update(
    rack_dir: &str,
    name: &str,
    description: Option<String>,
    version: Option<String>,
) -> anyhow::Result<()> {
    let mut m = load_marketplace_value(rack_dir)?;
    let p = get_plugin_mut(&mut m, name)?;
    if let Some(d) = description {
        p["description"] = serde_json::Value::String(d);
    }
    if let Some(v) = version {
        p["version"] = serde_json::Value::String(v.clone());
        let manifest = PathBuf::from(rack_dir)
            .join("plugins")
            .join(name)
            .join(".claude-plugin/plugin.json");
        if manifest.exists() {
            let mut mv: serde_json::Value =
                serde_json::from_str(&std::fs::read_to_string(&manifest)?)?;
            mv["version"] = serde_json::Value::String(v);
            std::fs::write(manifest, serde_json::to_string_pretty(&mv)?)?;
        }
    }
    save_marketplace_value(rack_dir, &m)
}

fn plugin_remove(rack_dir: &str, name: &str) -> anyhow::Result<()> {
    let mut m = load_marketplace_value(rack_dir)?;
    let arr = m
        .get_mut("plugins")
        .and_then(|x| x.as_array_mut())
        .ok_or_else(|| anyhow::anyhow!("missing plugins array"))?;
    arr.retain(|p| p.get("name").and_then(|x| x.as_str()) != Some(name));
    save_marketplace_value(rack_dir, &m)?;
    let dir = PathBuf::from(rack_dir).join("plugins").join(name);
    if dir.exists() {
        std::fs::remove_dir_all(dir)?;
    }
    Ok(())
}

fn skill_create(rack_dir: &str, plugin: &str, name: &str, description: &str) -> anyhow::Result<()> {
    let mut m = load_marketplace_value(rack_dir)?;
    let p = get_plugin_mut(&mut m, plugin)?;
    let skills = ensure_array_field(p, "skills")?;
    if !skills.iter().any(|s| s.as_str() == Some(name)) {
        skills.push(serde_json::Value::String(name.to_string()));
    }
    save_marketplace_value(rack_dir, &m)?;
    let d = PathBuf::from(rack_dir)
        .join("plugins")
        .join(plugin)
        .join("skills")
        .join(name);
    std::fs::create_dir_all(&d)?;
    std::fs::write(
        d.join("SKILL.md"),
        format!(
            "---
description: {}
---

# {}
",
            description, name
        ),
    )?;
    Ok(())
}

fn skill_remove(rack_dir: &str, plugin: &str, name: &str) -> anyhow::Result<()> {
    let mut m = load_marketplace_value(rack_dir)?;
    let p = get_plugin_mut(&mut m, plugin)?;
    let skills = ensure_array_field(p, "skills")?;
    skills.retain(|s| s.as_str() != Some(name));
    save_marketplace_value(rack_dir, &m)?;
    let d = PathBuf::from(rack_dir)
        .join("plugins")
        .join(plugin)
        .join("skills")
        .join(name);
    if d.exists() {
        std::fs::remove_dir_all(d)?;
    }
    Ok(())
}

fn subagent_create(rack_dir: &str, plugin: &str, name: &str, purpose: &str) -> anyhow::Result<()> {
    let mut m = load_marketplace_value(rack_dir)?;
    let p = get_plugin_mut(&mut m, plugin)?;
    let arr = ensure_array_field(p, "subagents")?;
    arr.retain(|x| x.get("name").and_then(|v| v.as_str()) != Some(name));
    arr.push(serde_json::json!({"name":name,"purpose":purpose}));
    save_marketplace_value(rack_dir, &m)
}

fn subagent_remove(rack_dir: &str, plugin: &str, name: &str) -> anyhow::Result<()> {
    let mut m = load_marketplace_value(rack_dir)?;
    let p = get_plugin_mut(&mut m, plugin)?;
    let arr = ensure_array_field(p, "subagents")?;
    arr.retain(|x| x.get("name").and_then(|v| v.as_str()) != Some(name));
    save_marketplace_value(rack_dir, &m)
}

fn hook_create(
    rack_dir: &str,
    plugin: &str,
    agent: &str,
    event: &str,
    run: &str,
) -> anyhow::Result<()> {
    let mut m = load_marketplace_value(rack_dir)?;
    let p = get_plugin_mut(&mut m, plugin)?;
    let arr = ensure_array_field(p, "hooks")?;
    arr.push(serde_json::json!({"agent":agent,"event":event,"run":run}));
    save_marketplace_value(rack_dir, &m)
}

fn hook_remove(rack_dir: &str, plugin: &str, agent: &str, event: &str) -> anyhow::Result<()> {
    let mut m = load_marketplace_value(rack_dir)?;
    let p = get_plugin_mut(&mut m, plugin)?;
    let arr = ensure_array_field(p, "hooks")?;
    arr.retain(|x| {
        !(x.get("agent").and_then(|v| v.as_str()) == Some(agent)
            && x.get("event").and_then(|v| v.as_str()) == Some(event))
    });
    save_marketplace_value(rack_dir, &m)
}

fn mcp_create(rack_dir: &str, plugin: &str, name: &str, command: &str) -> anyhow::Result<()> {
    let manifest = PathBuf::from(rack_dir)
        .join("plugins")
        .join(plugin)
        .join(".claude-plugin/plugin.json");
    if !manifest.exists() {
        anyhow::bail!("plugin manifest not found for {}", plugin);
    }
    let mut v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&manifest)?)?;
    if v.get("mcps").is_none() {
        v["mcps"] = serde_json::json!([]);
    }
    let arr = v
        .get_mut("mcps")
        .and_then(|x| x.as_array_mut())
        .ok_or_else(|| anyhow::anyhow!("invalid mcps"))?;
    arr.retain(|x| x.get("name").and_then(|s| s.as_str()) != Some(name));
    arr.push(serde_json::json!({"name":name,"command":command}));
    std::fs::write(manifest, serde_json::to_string_pretty(&v)?)?;
    Ok(())
}

fn mcp_remove(rack_dir: &str, plugin: &str, name: &str) -> anyhow::Result<()> {
    let manifest = PathBuf::from(rack_dir)
        .join("plugins")
        .join(plugin)
        .join(".claude-plugin/plugin.json");
    if !manifest.exists() {
        return Ok(());
    }
    let mut v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&manifest)?)?;
    if let Some(arr) = v.get_mut("mcps").and_then(|x| x.as_array_mut()) {
        arr.retain(|x| x.get("name").and_then(|s| s.as_str()) != Some(name));
    }
    std::fs::write(manifest, serde_json::to_string_pretty(&v)?)?;
    Ok(())
}

fn parse_upstream_plugins(path: &PathBuf) -> Vec<serde_json::Value> {
    if !path.exists() {
        return vec![];
    }
    let raw = match std::fs::read_to_string(path) {
        Ok(v) => v,
        Err(_) => return vec![],
    };
    let v: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(_) => return vec![],
    };
    v.get("plugins")
        .and_then(|x| x.as_array())
        .cloned()
        .unwrap_or_default()
}

fn rack_sync_upstreams(rack_dir: &str) -> anyhow::Result<usize> {
    let root = PathBuf::from(rack_dir);
    let upstreams = vec![
        root.join("_upstreams/claude-plugins-official/.claude-plugin/marketplace.json"),
        root.join("_upstreams/claude-code/.claude-plugin/marketplace.json"),
        root.join("_upstreams/skills/.claude-plugin/marketplace.json"),
    ];
    let mut seen = std::collections::HashSet::new();
    let mut plugins = Vec::<serde_json::Value>::new();
    for p in upstreams {
        for pl in parse_upstream_plugins(&p) {
            let name = pl
                .get("name")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            if name.is_empty() || seen.contains(&name) {
                continue;
            }
            seen.insert(name);
            plugins.push(pl);
        }
    }
    let out = serde_json::json!({
        "name": "paternoster-rack",
        "owner": {"name": "Paternoster Rack"},
        "metadata": {"sync_priority": ["claude-plugins-official", "claude-code", "skills"]},
        "plugins": plugins,
    });
    let out_path = root.join(".pater/marketplace.json");
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(out_path, serde_json::to_string_pretty(&out)?)?;
    Ok(seen.len())
}

fn classify_local_plugin_license(plugin_path: &std::path::Path) -> &'static str {
    for name in ["LICENSE", "LICENSE.md", "COPYING"] {
        let p = plugin_path.join(name);
        if p.exists() {
            let txt = std::fs::read_to_string(p)
                .unwrap_or_default()
                .to_ascii_uppercase();
            if ["MIT", "APACHE", "BSD", "ISC", "UNLICENSE", "CC0"]
                .iter()
                .any(|n| txt.contains(n))
            {
                return "permissive";
            }
            if ["GPL", "AGPL", "LGPL", "MPL"]
                .iter()
                .any(|n| txt.contains(n))
            {
                return "copyleft";
            }
        }
    }
    "unknown"
}

fn rack_license_audit(rack_dir: &str) -> anyhow::Result<RackLicenseAuditSummary> {
    let root = PathBuf::from(rack_dir);
    let mp = root.join(".pater/marketplace.json");
    let raw = std::fs::read_to_string(&mp)?;
    let v: serde_json::Value = serde_json::from_str(&raw)?;
    let plugins = v
        .get("plugins")
        .and_then(|x| x.as_array())
        .cloned()
        .unwrap_or_default();

    let mut permissive = 0usize;
    let mut copyleft = 0usize;
    let mut unknown = 0usize;
    let mut detailed = Vec::new();

    for p in plugins {
        let name = p.get("name").and_then(|x| x.as_str()).unwrap_or("");
        let source = p.get("source");
        let mut cls = "unknown";
        if let Some(s) = source.and_then(|x| x.as_str()) {
            if s.starts_with("./") {
                cls = classify_local_plugin_license(&root.join(s.trim_start_matches("./")));
            }
        }
        match cls {
            "permissive" => permissive += 1,
            "copyleft" => copyleft += 1,
            _ => unknown += 1,
        }
        detailed.push(serde_json::json!({"name": name, "classification": if cls=="unknown" {"proprietary/unknown"} else {cls}}));
    }

    let report = serde_json::json!({"plugins": detailed});
    std::fs::write(
        root.join(".pater/license-audit.json"),
        serde_json::to_string_pretty(&report)?,
    )?;

    Ok(RackLicenseAuditSummary {
        permissive,
        copyleft,
        unknown_count: unknown,
        total: permissive + copyleft + unknown,
    })
}

fn rack_mark_unknown_external(rack_dir: &str) -> anyhow::Result<usize> {
    let root = PathBuf::from(rack_dir);
    let mp_path = root.join(".pater/marketplace.json");
    let audit_path = root.join(".pater/license-audit.json");
    let mut m: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&mp_path)?)?;
    let a: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&audit_path)?)?;
    let mut unknown = std::collections::HashSet::new();
    if let Some(arr) = a.get("plugins").and_then(|x| x.as_array()) {
        for p in arr {
            if p.get("classification").and_then(|x| x.as_str()) == Some("proprietary/unknown") {
                if let Some(name) = p.get("name").and_then(|x| x.as_str()) {
                    unknown.insert(name.to_string());
                }
            }
        }
    }

    let mut marked = 0usize;
    if let Some(arr) = m.get_mut("plugins").and_then(|x| x.as_array_mut()) {
        for p in arr {
            if let Some(name) = p.get("name").and_then(|x| x.as_str()) {
                if unknown.contains(name) {
                    p["distribution"] =
                        serde_json::Value::String("external-reference-only".to_string());
                    p["license_status"] = serde_json::Value::String("unknown".to_string());
                    marked += 1;
                }
            }
        }
    }

    std::fs::write(mp_path, serde_json::to_string_pretty(&m)?)?;
    Ok(marked)
}

fn rack_sign_marketplace(rack_dir: &str, sign_key: &str) -> anyhow::Result<()> {
    let root = PathBuf::from(rack_dir);
    let status = std::process::Command::new("openssl")
        .args([
            "pkeyutl",
            "-sign",
            "-inkey",
            sign_key,
            "-rawin",
            "-in",
            root.join(".pater/marketplace.json")
                .to_string_lossy()
                .as_ref(),
            "-out",
            root.join(".pater/marketplace.sig.bin")
                .to_string_lossy()
                .as_ref(),
        ])
        .status()?;
    if !status.success() {
        anyhow::bail!("openssl signing failed")
    }
    let bytes = std::fs::read(root.join(".pater/marketplace.sig.bin"))?;
    std::fs::write(
        root.join(".pater/marketplace.sig"),
        format!("{}\n", hex::encode(bytes)),
    )?;
    let _ = std::fs::remove_file(root.join(".pater/marketplace.sig.bin"));
    Ok(())
}

fn run_rack_license_audit() -> String {
    match rack_license_audit("../rack") {
        Ok(r) if r.unknown_count == 0 => "ok".to_string(),
        Ok(_) => "failed".to_string(),
        Err(_) => "error".to_string(),
    }
}

fn audit(action: &str, data: serde_json::Value) {
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => return,
    };
    let path = PathBuf::from(home).join(".config/pater/audit.jsonl");
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let event = serde_json::json!({
        "ts": chrono_like_now(),
        "action": action,
        "data": data
    });
    let line = format!("{}\n", event);
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut f| std::io::Write::write_all(&mut f, line.as_bytes()));
}

fn chrono_like_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    ts.to_string()
}

fn upsert_installed(state: &mut State, entry: InstalledPlugin) {
    if let Some(existing) = state
        .installed
        .iter_mut()
        .find(|i| i.name == entry.name && i.marketplace == entry.marketplace)
    {
        *existing = entry;
    } else {
        state.installed.push(entry);
    }
}

fn managed_store_base() -> anyhow::Result<PathBuf> {
    let home = std::env::var("HOME")?;
    Ok(PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("pater")
        .join("plugins"))
}

fn materialize_plugin(name: &str, source_path: &std::path::Path) -> anyhow::Result<PathBuf> {
    let base = managed_store_base()?;
    std::fs::create_dir_all(&base)?;
    let dst = base.join(name);
    copy_dir_all(source_path, &dst)?;
    Ok(dst)
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> anyhow::Result<()> {
    if dst.exists() {
        std::fs::remove_dir_all(dst)?;
    }
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &to)?;
        } else {
            std::fs::copy(entry.path(), to)?;
        }
    }
    std::fs::write(dst.join(".pater-managed"), "managed-by=pater\n")?;
    Ok(())
}

fn adapter_base(target: &AdapterTarget) -> anyhow::Result<PathBuf> {
    let home = std::env::var("HOME")?;
    let p = match target {
        AdapterTarget::Claude => PathBuf::from(home).join(".claude/plugins"),
        AdapterTarget::Codex => PathBuf::from(home).join(".codex/plugins"),
        AdapterTarget::Openclaw => PathBuf::from(home).join(".openclaw/workspace/skills"),
        AdapterTarget::All => PathBuf::from(home).join(".pater/adapters"),
    };
    Ok(p)
}

fn sync_target(state: &State, target: AdapterTarget) -> anyhow::Result<()> {
    let base = adapter_base(&target)?;
    std::fs::create_dir_all(&base)?;
    let mut installed_dirs = Vec::new();
    let desired: std::collections::HashSet<String> =
        state.installed.iter().map(|p| p.name.clone()).collect();

    for p in &state.installed {
        let mut src = PathBuf::from(&p.local_path);
        if !src.exists() {
            if let Ok(r) = rack::resolve_plugin_path(&p.marketplace_source, &p.source) {
                src = r;
            }
        }
        if !src.exists() {
            continue;
        }
        let dst = base.join(&p.name);
        copy_dir_all(&src, &dst)?;
        installed_dirs.push(dst.to_string_lossy().to_string());
    }

    // cleanup stale, previously managed plugin dirs
    for entry in std::fs::read_dir(&base)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();
        let managed = path.join(".pater-managed").exists();
        if managed && !desired.contains(&name) {
            std::fs::remove_dir_all(path)?;
        }
    }

    write_activation_shim(&target, &installed_dirs)?;
    Ok(())
}

fn sync_installed(state: &State, target: AdapterTarget) -> anyhow::Result<()> {
    match target {
        AdapterTarget::All => {
            sync_target(state, AdapterTarget::Claude)?;
            sync_target(state, AdapterTarget::Codex)?;
            sync_target(state, AdapterTarget::Openclaw)?;
        }
        t => sync_target(state, t)?,
    }
    Ok(())
}

fn adapter_smoke(state: &State, target: AdapterTarget) -> anyhow::Result<Vec<SmokeReport>> {
    let targets = match target {
        AdapterTarget::All => vec![
            AdapterTarget::Claude,
            AdapterTarget::Codex,
            AdapterTarget::Openclaw,
        ],
        t => vec![t],
    };

    let mut out = Vec::new();
    for t in targets {
        let base = adapter_base(&t)?;
        let mut missing = Vec::new();
        for p in &state.installed {
            if !base.join(&p.name).exists() {
                missing.push(p.name.clone());
            }
        }

        let shim_ok = match t {
            AdapterTarget::Claude => std::env::var("HOME")
                .map(|h| PathBuf::from(h).join(".claude/pater.plugins.json").exists())
                .unwrap_or(false),
            AdapterTarget::Codex => std::env::var("HOME")
                .map(|h| PathBuf::from(h).join(".codex/pater.plugins.json").exists())
                .unwrap_or(false),
            AdapterTarget::Openclaw => std::env::var("HOME")
                .map(|h| {
                    PathBuf::from(h)
                        .join(".openclaw/workspace/skills/.pater-index.json")
                        .exists()
                })
                .unwrap_or(false),
            AdapterTarget::All => true,
        };

        let status = if missing.is_empty() && shim_ok {
            "ok".to_string()
        } else if !shim_ok {
            "missing_shim".to_string()
        } else {
            "missing_plugins".to_string()
        };

        out.push(SmokeReport {
            adapter: format!("{:?}", t).to_lowercase(),
            status,
            checked_plugins: state.installed.len(),
            missing_plugins: missing,
        });
    }
    Ok(out)
}

fn adapter_doctor(state: &State) -> anyhow::Result<DoctorReport> {
    let home = std::env::var("HOME")?;
    let smoke = adapter_smoke(state, AdapterTarget::All)?;

    let configs = vec![
        CheckItem {
            name: "claude_settings".to_string(),
            status: if PathBuf::from(&home).join(".claude/settings.json").exists() {
                "ok".to_string()
            } else {
                "missing".to_string()
            },
        },
        CheckItem {
            name: "codex_config".to_string(),
            status: if PathBuf::from(&home).join(".codex/config.toml").exists() {
                "ok".to_string()
            } else {
                "missing".to_string()
            },
        },
        CheckItem {
            name: "openclaw_index".to_string(),
            status: if PathBuf::from(&home)
                .join(".openclaw/workspace/skills/.pater-index.json")
                .exists()
            {
                "ok".to_string()
            } else {
                "missing".to_string()
            },
        },
    ];

    let wrappers = vec![
        CheckItem {
            name: "pater-claude".to_string(),
            status: if PathBuf::from(&home)
                .join(".local/bin/pater-claude")
                .exists()
            {
                "ok".to_string()
            } else {
                "missing".to_string()
            },
        },
        CheckItem {
            name: "pater-codex".to_string(),
            status: if PathBuf::from(&home).join(".local/bin/pater-codex").exists() {
                "ok".to_string()
            } else {
                "missing".to_string()
            },
        },
        CheckItem {
            name: "pater-openclaw".to_string(),
            status: if PathBuf::from(&home)
                .join(".local/bin/pater-openclaw")
                .exists()
            {
                "ok".to_string()
            } else {
                "missing".to_string()
            },
        },
    ];

    let path_has_local_bin = std::env::var("PATH")
        .unwrap_or_default()
        .split(':')
        .any(|p| p == PathBuf::from(&home).join(".local/bin").to_string_lossy());

    let all_ok = smoke.iter().all(|s| s.status == "ok")
        && configs.iter().all(|c| c.status == "ok")
        && wrappers.iter().all(|w| w.status == "ok");

    Ok(DoctorReport {
        overall: if all_ok { "ok" } else { "needs_attention" }.to_string(),
        path_has_local_bin,
        smoke,
        configs,
        wrappers,
    })
}

fn write_activation_shim(target: &AdapterTarget, plugin_dirs: &[String]) -> anyhow::Result<()> {
    let home = std::env::var("HOME")?;
    match target {
        AdapterTarget::Claude => {
            let root = PathBuf::from(&home).join(".claude");
            std::fs::create_dir_all(&root)?;
            let data = serde_json::json!({
                "managedBy": "pater",
                "adapter": "claude",
                "plugin_dirs": plugin_dirs,
                "note": "Generated by pater. Load these plugin dirs in Claude Code startup config."
            });
            std::fs::write(
                root.join("pater.plugins.json"),
                serde_json::to_string_pretty(&data)?,
            )?;
            patch_claude_config(&root, plugin_dirs)?;
            write_wrapper(&home, "pater-claude", "claude", plugin_dirs)?;
        }
        AdapterTarget::Codex => {
            let root = PathBuf::from(&home).join(".codex");
            std::fs::create_dir_all(&root)?;
            let data = serde_json::json!({
                "managedBy": "pater",
                "adapter": "codex",
                "plugin_dirs": plugin_dirs,
                "note": "Generated by pater. Use these plugin dirs in Codex startup config."
            });
            std::fs::write(
                root.join("pater.plugins.json"),
                serde_json::to_string_pretty(&data)?,
            )?;
            patch_codex_config(&root, plugin_dirs)?;
            write_wrapper(&home, "pater-codex", "codex", plugin_dirs)?;
        }
        AdapterTarget::Openclaw => {
            let root = PathBuf::from(&home)
                .join(".openclaw")
                .join("workspace")
                .join("skills");
            std::fs::create_dir_all(&root)?;
            let data = serde_json::json!({
                "managedBy": "pater",
                "adapter": "openclaw",
                "plugin_dirs": plugin_dirs,
                "note": "Generated by pater. Skills are materialized under this directory."
            });
            std::fs::write(
                root.join(".pater-index.json"),
                serde_json::to_string_pretty(&data)?,
            )?;
            write_wrapper(&home, "pater-openclaw", "openclaw", plugin_dirs)?;
        }
        AdapterTarget::All => {}
    }
    Ok(())
}

fn patch_claude_config(root: &std::path::Path, plugin_dirs: &[String]) -> anyhow::Result<()> {
    let cfg = root.join("settings.json");
    let mut v = if cfg.exists() {
        serde_json::from_str::<serde_json::Value>(&std::fs::read_to_string(&cfg)?)
            .unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    v["pater"] = serde_json::json!({ "plugin_dirs": plugin_dirs });
    std::fs::write(cfg, serde_json::to_string_pretty(&v)?)?;
    Ok(())
}

fn patch_codex_config(root: &std::path::Path, plugin_dirs: &[String]) -> anyhow::Result<()> {
    let cfg = root.join("config.toml");
    let mut content = if cfg.exists() {
        std::fs::read_to_string(&cfg)?
    } else {
        String::new()
    };
    let start = "# >>> pater managed start >>>";
    let end = "# <<< pater managed end <<<";
    if let (Some(s), Some(e)) = (content.find(start), content.find(end)) {
        content.replace_range(s..(e + end.len()), "");
    }
    let dirs = plugin_dirs
        .iter()
        .map(|d| format!("\"{}\"", d.replace('"', "\\\"")))
        .collect::<Vec<_>>()
        .join(", ");
    content.push_str(&format!(
        "\n{start}\n[pater]\nplugin_dirs = [{dirs}]\n{end}\n"
    ));
    std::fs::write(cfg, content)?;
    Ok(())
}

fn write_wrapper(home: &str, name: &str, cmd: &str, plugin_dirs: &[String]) -> anyhow::Result<()> {
    let bin = PathBuf::from(home).join(".local/bin");
    std::fs::create_dir_all(&bin)?;
    let mut args = String::new();
    for d in plugin_dirs {
        args.push_str(&format!(" --plugin-dir '{}'", d.replace('\'', "'\\''")));
    }
    let script = format!("#!/usr/bin/env sh\nexec {cmd}{args} \"$@\"\n");
    let path = bin.join(name);
    std::fs::write(&path, script)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&path, perms)?;
    }
    Ok(())
}

fn parse_target(target: &str) -> (String, Option<String>) {
    let parts: Vec<&str> = target.split('@').collect();
    if parts.len() == 2 {
        (parts[0].to_string(), Some(parts[1].to_string()))
    } else {
        (target.to_string(), None)
    }
}

fn state_path() -> anyhow::Result<PathBuf> {
    let home = std::env::var("HOME")?;
    Ok(PathBuf::from(home).join(".config/pater/state.json"))
}

fn lockfile_path() -> anyhow::Result<PathBuf> {
    let home = std::env::var("HOME")?;
    Ok(PathBuf::from(home).join(".config/pater/pater.lock"))
}

fn load_state() -> anyhow::Result<State> {
    let p = state_path()?;
    if !p.exists() {
        return Ok(State::default());
    }
    let raw = std::fs::read_to_string(p)?;
    Ok(serde_json::from_str(&raw)?)
}

fn save_state(s: &State) -> anyhow::Result<()> {
    let p = state_path()?;
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(p, serde_json::to_string_pretty(s)?)?;
    Ok(())
}

fn save_lockfile(state: &State) -> anyhow::Result<()> {
    let lock = Lockfile {
        version: 1,
        plugins: state.installed.clone(),
    };
    let p = lockfile_path()?;
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(p, serde_json::to_string_pretty(&lock)?)?;
    Ok(())
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
