use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "crusty", about = "A minimal CLI for Canton Network")]
pub struct Cli {
    /// Path to .env file with connection settings
    #[arg(long, default_value = ".env")]
    pub env_file: String,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Manage parties on the ledger
    Party(PartyArgs),

    /// Show the participant node ID
    ParticipantId,
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
        /// Include remote (non-local) parties
        #[arg(short, long)]
        all: bool,

        /// Include system parties (participant, DSO, sv)
        #[arg(short, long)]
        system: bool,
    },

    /// Create a new party
    Create {
        /// Party name hint (optional; Canton generates a UUID if omitted)
        hint: Option<String>,
    },

    /// Get a party by hint (searches local parties)
    Get {
        /// Party hint to search for
        hint: String,
    },
}
