use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use url::Url;

pub struct LedgerConfig {
    pub ledger_url: Url,
    pub auth: AuthConfig,
}

pub enum AuthConfig {
    /// Machine-to-machine: client_id + client_secret via OIDC
    ClientCredentials {
        oidc_url: Url,
        client_id: String,
        client_secret: String,
        audience: String,
    },
    /// Pre-obtained JWT token (for scripting / --token flag)
    Token(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigFile {
    #[serde(default)]
    pub default_profile: Option<String>,
    #[serde(default)]
    pub profiles: BTreeMap<String, Profile>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
    pub ledger_url: String,
    #[serde(flatten)]
    pub auth: ProfileAuth,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "auth_method")]
pub enum ProfileAuth {
    #[serde(rename = "client_credentials")]
    ClientCredentials {
        oidc_url: String,
        client_id: String,
        client_secret: String,
        audience: String,
    },
    #[serde(rename = "token")]
    Token { token: String },
}

impl ConfigFile {
    pub fn config_dir() -> Result<PathBuf> {
        let home = dirs::home_dir().context("could not determine home directory")?;
        Ok(home.join(".crusty"))
    }

    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if !path.exists() {
            return Ok(Self {
                default_profile: None,
                profiles: BTreeMap::new(),
            });
        }
        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read config: {}", path.display()))?;
        toml::from_str(&content)
            .with_context(|| format!("failed to parse config: {}", path.display()))
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        let dir = Self::config_dir()?;
        fs::create_dir_all(&dir)
            .with_context(|| format!("failed to create config dir: {}", dir.display()))?;
        let content = toml::to_string_pretty(self).context("failed to serialize config")?;
        fs::write(&path, content)
            .with_context(|| format!("failed to write config: {}", path.display()))
    }

    pub fn get_profile(&self, name: Option<&str>) -> Result<&Profile> {
        let profile_name = name
            .or(self.default_profile.as_deref())
            .context("no profile specified and no default profile set")?;
        self.profiles
            .get(profile_name)
            .with_context(|| format!("profile '{}' not found in config", profile_name))
    }

    pub fn quickstart_profile() -> Profile {
        Profile {
            ledger_url: "http://localhost:2975".to_string(),
            auth: ProfileAuth::ClientCredentials {
                oidc_url: "http://keycloak.localhost:8082/realms/AppUser/.well-known/openid-configuration".to_string(),
                client_id: "app-user-validator".to_string(),
                client_secret: "6m12QyyGl81d9nABWQXMycZdXho6ejEX".to_string(),
                audience: "https://canton.network.global".to_string(),
            },
        }
    }
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;

impl Profile {
    pub fn to_ledger_config(&self) -> Result<LedgerConfig> {
        let ledger_url = Url::parse(&self.ledger_url)
            .with_context(|| format!("invalid ledger_url: {}", self.ledger_url))?;

        let auth = match &self.auth {
            ProfileAuth::ClientCredentials {
                oidc_url,
                client_id,
                client_secret,
                audience,
            } => AuthConfig::ClientCredentials {
                oidc_url: Url::parse(oidc_url)
                    .with_context(|| format!("invalid oidc_url: {}", oidc_url))?,
                client_id: client_id.clone(),
                client_secret: client_secret.clone(),
                audience: audience.clone(),
            },
            ProfileAuth::Token { token } => AuthConfig::Token(token.clone()),
        };

        Ok(LedgerConfig { ledger_url, auth })
    }
}
