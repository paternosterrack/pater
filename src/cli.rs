use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

pub const DEFAULT_MARKETPLACE_SOURCE: &str = "paternosterrack/rack";

#[derive(Parser, Debug)]
#[command(name = "pater", version, about = "Paternoster Rack CLI")]
pub struct Cli {
    #[arg(long, global = true, help = "Output machine-readable JSON")]
    pub json: bool,
    #[arg(
        long,
        global = true,
        default_value = DEFAULT_MARKETPLACE_SOURCE,
        help = "Default marketplace source (dir, marketplace.json, url, or owner/repo)"
    )]
    pub marketplace: String,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
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
pub enum RemoteCommands {
    Add { source: String },
    List,
    Update,
}

#[derive(Subcommand, Debug)]
pub enum HookCommands {
    List {
        #[arg(long)]
        agent: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum TrustCommands {
    Init,
    List,
    Status,
}

#[derive(Subcommand, Debug)]
pub enum PolicyCommands {
    Eval {
        plugin: String,
        #[arg(long, value_enum, default_value_t = AdapterTarget::All)]
        agent: AdapterTarget,
    },
}

#[derive(Subcommand, Debug)]
pub enum RackCommands {
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
pub enum AuthorCommands {
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
pub enum PluginCommands {
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
pub enum SkillCommands {
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
pub enum SubagentCommands {
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
pub enum HookCommandsAdmin {
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
pub enum McpCommands {
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
pub enum AdapterCommands {
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
pub enum AdapterTarget {
    All,
    Claude,
    Codex,
    Openclaw,
}

#[derive(Clone, Debug, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum InstallScope {
    User,
    Project,
    Local,
}
