use std::process;

use anyhow::{bail, Context, Result};
use clap::Parser;
use url::Url;

use crusty::cli::{Cli, Command, ConfigCommand, PartyCommand};
use crusty::config::{AuthConfig, ConfigFile, LedgerConfig};
use crusty::domain::party::PartyHint;
use crusty::domain::services::LedgerService;
use crusty::json_api::JsonApiLedger;

fn load_config_from_env(env_file: Option<&str>) -> Result<LedgerConfig> {
    match env_file {
        Some(path) => {
            dotenvy::from_filename(path)
                .with_context(|| format!("failed to load env file: {}", path))?;
        }
        None => {
            let _ = dotenvy::from_filename(".env");
        }
    }

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

fn load_config(profile: Option<&str>, env_file: Option<&str>) -> Result<LedgerConfig> {
    if env_file.is_some() {
        return load_config_from_env(env_file);
    }

    let config_file = ConfigFile::load()?;
    if config_file.profiles.is_empty() && profile.is_none() {
        return load_config_from_env(None);
    }

    let p = config_file.get_profile(profile)?;
    p.to_ledger_config()
}

fn run_config_command(command: ConfigCommand) -> Result<()> {
    match command {
        ConfigCommand::Init { template } => {
            let profile = match template.as_str() {
                "quickstart" => ConfigFile::quickstart_profile(),
                _ => bail!("unknown template: '{}'. Available: quickstart", template),
            };

            let mut config = ConfigFile::load()?;
            if config.profiles.contains_key(&template) {
                bail!(
                    "profile '{}' already exists. Remove it from the config file first to reinitialize.",
                    template
                );
            }
            config.profiles.insert(template.clone(), profile);
            if config.default_profile.is_none() {
                config.default_profile = Some(template.clone());
            }
            config.save()?;

            let path = ConfigFile::config_path()?;
            println!("Profile '{}' added to {}", template, path.display());
            if config.default_profile.as_deref() == Some(&template) {
                println!("Set as default profile");
            }
        }

        ConfigCommand::Use { profile } => {
            let mut config = ConfigFile::load()?;
            if !config.profiles.contains_key(&profile) {
                bail!("profile '{}' not found in config", profile);
            }
            config.default_profile = Some(profile.clone());
            config.save()?;
            println!("Default profile set to '{}'", profile);
        }

        ConfigCommand::Show => {
            let path = ConfigFile::config_path()?;
            let config = ConfigFile::load()?;
            if config.profiles.is_empty() {
                if path.exists() {
                    println!("Config file exists at {} but has no profiles", path.display());
                } else {
                    println!("No config file found at {}", path.display());
                }
                println!("Run 'crusty config init quickstart' to create one");
            } else {
                println!("Config: {}", path.display());
                if let Some(default) = &config.default_profile {
                    println!("Default profile: {}", default);
                }
                println!("\nProfiles:");
                for (name, profile) in &config.profiles {
                    let marker = if config.default_profile.as_deref() == Some(name.as_str()) {
                        " (default)"
                    } else {
                        ""
                    };
                    println!("  {}{} -> {}", name, marker, profile.ledger_url);
                }
            }
        }
    }
    Ok(())
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Config(args) => run_config_command(args.command),

        command => {
            let config = load_config(cli.profile.as_deref(), cli.env_file.as_deref())?;
            let ledger = JsonApiLedger::new(config)?;
            let service = LedgerService::new(ledger);

            match command {
                Command::Party(args) => match args.command {
                    PartyCommand::List { hint, all } => {
                        let parties = service.list_parties(hint.as_deref(), all)?;
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
                },

                Command::ParticipantId => {
                    let id = service.get_participant_id()?;
                    println!("{}", id);
                }

                Command::Balance { party } => {
                    let party_id = match party {
                        Some(hint) => service.resolve_party_by_hint(&hint)?.id,
                        None => {
                            let user = service.get_authenticated_user()?;
                            user.primary_party
                                .ok_or_else(|| anyhow::anyhow!("no primary party set for authenticated user"))?
                        }
                    };
                    println!("Party: {}\n", party_id);
                    let balances = service.get_balance(&party_id)?;
                    if balances.is_empty() {
                        println!("No holdings found");
                    } else {
                        for b in &balances {
                            println!("{} ({} holdings)", b.instrument.name, b.holding_count);
                            println!("  Total:     {}", b.total);
                            println!("  Available: {}", b.available);
                            if b.locked_count > 0 {
                                println!("  Locked:    {} ({} holdings)", b.locked, b.locked_count);
                            }
                        }
                    }
                }

                Command::Whoami => {
                    let user = service.get_authenticated_user()?;
                    println!("User ID:       {}", user.id);
                    println!("Username:      {}", user.username.as_deref().unwrap_or("(none)"));
                    println!(
                        "Primary party: {}",
                        user.primary_party
                            .as_ref()
                            .map(|p| p.to_string())
                            .unwrap_or_else(|| "(none)".to_string())
                    );
                }

                Command::Config(_) => unreachable!(),
            }

            Ok(())
        }
    }
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {:#}", err);
        process::exit(1);
    }
}
