use reqwest::blocking::Client;
use thiserror::Error;
use url::Url;

use crate::config::AuthConfig;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("failed to fetch OIDC configuration: {0}")]
    OidcDiscoveryFailed(String),

    #[error("token endpoint not found in OIDC configuration")]
    TokenEndpointNotFound,

    #[error("failed to obtain token: {0}")]
    TokenRequestFailed(String),

    #[error("access_token not found in token response")]
    AccessTokenMissing,
}

pub fn obtain_token(config: &AuthConfig) -> Result<String, AuthError> {
    match config {
        AuthConfig::ClientCredentials {
            oidc_url,
            client_id,
            client_secret,
            audience,
        } => obtain_client_credentials_token(oidc_url, client_id, client_secret, audience),
        
        AuthConfig::Token(token) => Ok(token.clone())
    }
}

fn discover_token_endpoint(oidc_url: &Url) -> Result<String, AuthError> {
    let oidc_config: serde_json::Value = Client::new()
        .get(oidc_url.as_str())
        .send()
        .map_err(|e| AuthError::OidcDiscoveryFailed(e.to_string()))?
        .json()
        .map_err(|e| AuthError::OidcDiscoveryFailed(e.to_string()))?;

    oidc_config["token_endpoint"]
        .as_str()
        .map(String::from)
        .ok_or(AuthError::TokenEndpointNotFound)
}

fn obtain_client_credentials_token(
    oidc_url: &Url,
    client_id: &str,
    client_secret: &str,
    audience: &str,
) -> Result<String, AuthError> {
    let token_endpoint = discover_token_endpoint(oidc_url)?;

    let resp: serde_json::Value = Client::new()
        .post(&token_endpoint)
        .form(&[
            ("grant_type", "client_credentials"),
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("audience", audience),
        ])
        .send()
        .map_err(|e| AuthError::TokenRequestFailed(e.to_string()))?
        .json()
        .map_err(|e| AuthError::TokenRequestFailed(e.to_string()))?;

    resp["access_token"]
        .as_str()
        .map(String::from)
        .ok_or(AuthError::AccessTokenMissing)
}
