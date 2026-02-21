use crate::*;

pub fn handle_runtime_commands(
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
