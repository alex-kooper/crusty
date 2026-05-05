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
