use crate::*;

pub fn handle_trust_commands(cli: &Cli, policy: &PolicyFile) -> anyhow::Result<bool> {
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

pub fn handle_rack_commands(cli: &Cli) -> anyhow::Result<bool> {
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

pub fn handle_author_commands(cli: &Cli) -> anyhow::Result<bool> {
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
