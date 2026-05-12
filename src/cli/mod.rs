use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "crusty", about = "A minimal CLI for Canton Network")]
pub struct Cli {
    /// Profile to use from ~/.crusty/config.toml
    #[arg(short, long)]
    pub profile: Option<String>,

    /// Path to .env file (overrides config.toml)
    #[arg(long)]
    pub env_file: Option<String>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Manage parties on the ledger
    Party(PartyArgs),

    /// Show the participant node ID
    ParticipantId,

    /// Manage configuration profiles
    Config(ConfigArgs),
}

#[derive(Parser)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommand,
}

#[derive(Subcommand)]
pub enum ConfigCommand {
    /// Initialize a config profile from a template
    Init {
        /// Template name (e.g. "quickstart")
        template: String,
    },

    /// Set the default profile
    Use {
        /// Profile name to set as default
        profile: String,
    },

    /// Show current configuration
    Show,
}

#[derive(Parser)]
pub struct PartyArgs {
    #[command(subcommand)]
    pub command: PartyCommand,
}

#[derive(Subcommand)]
pub enum PartyCommand {
    /// List parties visible to this node
    List {
        /// Filter by party hint prefix
        hint: Option<String>,

        /// Include remote (non-local) parties
        #[arg(short, long)]
        all: bool,
    },

    /// Create a new party
    Create {
        /// Party name hint (optional; Canton generates a UUID if omitted)
        hint: Option<String>,
    },
}
