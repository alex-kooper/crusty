use std::process;

use anyhow::{Context, Result};
use clap::Parser;
use url::Url;

use crusty::cli::{Cli, Command, PartyCommand};
use crusty::config::{AuthConfig, LedgerConfig};
use crusty::domain::party::PartyHint;
use crusty::domain::services::{LedgerService, PartyFilter};
use crusty::json_api::JsonApiLedger;

fn load_config(env_file: &str) -> Result<LedgerConfig> {
    dotenvy::from_filename(env_file)
        .with_context(|| format!("failed to load env file: {}", env_file))?;

    let ledger_url = Url::parse(&std::env::var("LEDGER_API_URL")?)
        .context("invalid LEDGER_API_URL")?;

    let auth = match std::env::var("OAUTH_BEARER_TOKEN") {
        Ok(token) => AuthConfig::Token(token),
        Err(_) => AuthConfig::ClientCredentials {
            oidc_url: Url::parse(&std::env::var("OAUTH_OIDC_CONF_URL")?)
                .context("invalid OAUTH_OIDC_CONF_URL")?,
            client_id: std::env::var("OAUTH_CLIENT_ID")
                .context("OAUTH_CLIENT_ID not set")?,
            client_secret: std::env::var("OAUTH_CLIENT_SECRET")
                .context("OAUTH_CLIENT_SECRET not set")?,
            audience: std::env::var("OAUTH_AUDIENCE")
                .context("OAUTH_AUDIENCE not set")?,
        },
    };

    Ok(LedgerConfig { ledger_url, auth })
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    let config = load_config(&cli.env_file)?;
    let ledger = JsonApiLedger::new(config)?;
    let service = LedgerService::new(ledger);

    match cli.command {
        Command::Party(args) => match args.command {
            PartyCommand::List { all, system } => {
                let filter = PartyFilter {
                    include_remote: all,
                    include_system: system,
                };
                let parties = service.list_parties(&filter)?;
                for party in &parties {
                    let local_marker = if party.is_local { "local" } else { "remote" };
                    println!("[{}] {}", local_marker, party.id);
                }
            }

            PartyCommand::Create { hint } => {
                let hint = hint.map(PartyHint::new);
                let party = service.create_party(hint.as_ref())?;
                println!("{}", party.id);
            }

            PartyCommand::Get { hint } => {
                let party = service.find_local_party_by_hint(&hint)?;
                println!("{}", party.id);
            }
        },

        Command::ParticipantId => {
            let id = service.get_participant_id()?;
            println!("{}", id);
        }
    }

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {:#}", err);
        process::exit(1);
    }
}
