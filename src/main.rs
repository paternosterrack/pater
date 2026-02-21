use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

mod rack;

const DEFAULT_MARKETPLACE_SOURCE: &str = "paternosterrack/rack";

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
    Discover {
        query: Option<String>,
    },
    Show {
        plugin: String,
    },
    Install {
        target: String,
        #[arg(long, value_enum, default_value_t = InstallScope::User)]
        scope: InstallScope,
    },
    Update {
        plugin: Option<String>,
        #[arg(long, default_value_t = false)]
        allow_permission_increase: bool,
    },
    Remove {
        plugin: String,
    },
    Installed,
    Hooks {
        #[command(subcommand)]
        command: HookCommands,
    },
    Validate,
    Marketplace {
        #[command(subcommand)]
        command: MarketplaceCommands,
    },
}

#[derive(Subcommand, Debug)]
enum HookCommands {
    List {
        #[arg(long)]
        agent: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum MarketplaceCommands {
    Add { source: String },
    List,
    Update,
}

#[derive(Clone, Debug, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
enum InstallScope {
    User,
    Project,
    Local,
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
    source: String,
    version: Option<String>,
    permissions: Vec<String>,
    scope: InstallScope,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct Lockfile {
    version: u32,
    plugins: Vec<InstalledPlugin>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut state = load_state()?;

    ensure_default_marketplace(&mut state)?;

    for m in &state.marketplaces {
        let _ = rack::refresh_marketplace(&m.source);
    }
    let _ = rack::refresh_marketplace(&cli.marketplace);

    let default_market = rack::load_marketplace(&cli.marketplace)?;
    let mut all_markets = vec![MarketRef {
        name: default_market.name.clone(),
        source: cli.marketplace.clone(),
    }];
    all_markets.extend(state.marketplaces.clone());
    dedupe_markets(&mut all_markets);

    match cli.command {
        Commands::Discover { query } => {
            let items = discover_across(&all_markets, query.as_deref())?;
            print_out(cli.json, &items, |p| {
                format!("{}\t{}\t{}", p.marketplace, p.name, p.description)
            })?;
        }
        Commands::Show { plugin } => {
            let (name, market) = parse_target(&plugin);
            let p = show_plugin(&all_markets, &name, market.as_deref())?;
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
            let p = show_plugin(&all_markets, &name, market.as_deref())?;
            let entry = InstalledPlugin {
                name: p.name.clone(),
                marketplace: p.marketplace.clone(),
                source: p.source.clone(),
                version: p.version.clone(),
                permissions: p.permissions.clone(),
                scope,
            };
            upsert_installed(&mut state, entry.clone());
            save_state(&state)?;
            save_lockfile(&state)?;

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
            }
        }
        Commands::Update {
            plugin,
            allow_permission_increase,
        } => {
            let report = update_plugins(
                &mut state,
                &all_markets,
                plugin.as_deref(),
                allow_permission_increase,
            )?;
            save_state(&state)?;
            save_lockfile(&state)?;
            print_out(cli.json, &report, |r| format!("{}\t{}", r.name, r.status))?;
        }
        Commands::Remove { plugin } => {
            let before = state.installed.len();
            state.installed.retain(|p| p.name != plugin);
            save_state(&state)?;
            save_lockfile(&state)?;
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
        Commands::Installed => {
            print_out(cli.json, &state.installed, |p| {
                format!("{}\t{}\t{:?}", p.name, p.marketplace, p.scope)
            })?;
        }
        Commands::Hooks { command } => match command {
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
        Commands::Marketplace { command } => match command {
            MarketplaceCommands::Add { source } => {
                let m = rack::load_marketplace(&source)?;
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
            MarketplaceCommands::List => {
                print_out(cli.json, &state.marketplaces, |m| {
                    format!("{}\t{}", m.name, m.source)
                })?;
            }
            MarketplaceCommands::Update => {
                let mut checked = 0usize;
                for m in &state.marketplaces {
                    rack::refresh_marketplace(&m.source)?;
                    let _ = rack::load_marketplace(&m.source)?;
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
    }

    Ok(())
}

#[derive(Serialize, Clone)]
struct DiscoverItem {
    marketplace: String,
    name: String,
    description: String,
    version: Option<String>,
    source: String,
    permissions: Vec<String>,
}

#[derive(Serialize)]
struct UpdateReport {
    name: String,
    status: String,
    old_version: Option<String>,
    new_version: Option<String>,
    added_permissions: Vec<String>,
}

fn update_plugins(
    state: &mut State,
    markets: &[MarketRef],
    only: Option<&str>,
    allow_permission_increase: bool,
) -> anyhow::Result<Vec<UpdateReport>> {
    let mut reports = Vec::new();
    for installed in &mut state.installed {
        if only.map(|o| o != installed.name).unwrap_or(false) {
            continue;
        }
        let latest = show_plugin(markets, &installed.name, Some(&installed.marketplace))?;
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
) -> anyhow::Result<Vec<DiscoverItem>> {
    let mut out = Vec::new();
    for m in markets {
        let loaded = rack::load_marketplace(&m.source)?;
        for p in rack::discover(&loaded, query) {
            out.push(DiscoverItem {
                marketplace: loaded.name.clone(),
                name: p.name.clone(),
                description: p.description.clone().unwrap_or_default(),
                version: p.version.clone(),
                source: p.source.clone(),
                permissions: p.permissions.clone(),
            });
        }
    }
    Ok(out)
}

fn show_plugin(
    markets: &[MarketRef],
    name: &str,
    marketplace: Option<&str>,
) -> anyhow::Result<DiscoverItem> {
    for m in markets {
        let loaded = rack::load_marketplace(&m.source)?;
        if let Some(filter) = marketplace {
            if loaded.name != filter {
                continue;
            }
        }
        if let Ok(p) = rack::show(&loaded, name) {
            return Ok(DiscoverItem {
                marketplace: loaded.name.clone(),
                name: p.name.clone(),
                description: p.description.clone().unwrap_or_default(),
                version: p.version.clone(),
                source: p.source.clone(),
                permissions: p.permissions.clone(),
            });
        }
    }
    anyhow::bail!("plugin not found: {}", name)
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
