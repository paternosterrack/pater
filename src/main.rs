use clap::{Parser, Subcommand};
use serde::Serialize;

mod rack;

#[derive(Parser, Debug)]
#[command(name = "pater", version, about = "Paternoster Rack CLI")]
struct Cli {
    #[arg(long, global = true, help = "Output machine-readable JSON")]
    json: bool,
    #[arg(long, global = true, default_value = "../rack/index/skills.json", help = "Path to rack index JSON")]
    index: String,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Search { query: String },
    Show { id: String },
    Hooks {
        #[command(subcommand)]
        command: HookCommands,
    },
    Validate,
}

#[derive(Subcommand, Debug)]
enum HookCommands {
    List {
        #[arg(long)]
        agent: Option<String>,
    },
}

#[derive(Serialize)]
struct JsonOut<T: Serialize> {
    ok: bool,
    data: T,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let idx = rack::load_index(&cli.index)?;

    match cli.command {
        Commands::Search { query } => {
            let results = rack::search(&idx, &query);
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&JsonOut { ok: true, data: results })?);
            } else {
                for s in results {
                    println!("{}\t{}\t{}", s.id, s.version, s.summary);
                }
            }
        }
        Commands::Show { id } => {
            let skill = rack::show(&idx, &id)?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&JsonOut { ok: true, data: skill })?);
            } else {
                println!("id: {}", skill.id);
                println!("version: {}", skill.version);
                println!("summary: {}", skill.summary);
                println!("agents: {}", skill.agents.join(", "));
            }
        }
        Commands::Hooks { command } => match command {
            HookCommands::List { agent } => {
                let hooks = rack::list_hooks(&idx, agent.as_deref());
                if cli.json {
                    println!("{}", serde_json::to_string_pretty(&JsonOut { ok: true, data: hooks })?);
                } else {
                    for h in hooks {
                        println!("{}\t{}\t{}", h.skill_id, h.agent, h.event);
                    }
                }
            }
        },
        Commands::Validate => {
            rack::validate(&idx)?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&JsonOut { ok: true, data: "valid" })?);
            } else {
                println!("index valid");
            }
        }
    }

    Ok(())
}
