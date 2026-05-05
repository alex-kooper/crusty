use thiserror::Error;

use crate::auth::AuthError;

#[derive(Debug, Error)]
pub enum PartyError {
    #[error("party not found: {0}")]
    NotFound(String),

    #[error("party already exists: {0}")]
    AlreadyExists(String),
}

#[derive(Debug, Error)]
pub enum LedgerError {
    #[error(transparent)]
    Party(#[from] PartyError),

    #[error(transparent)]
    Auth(#[from] AuthError),

    #[error("unauthorized")]
    Unauthorized,

    #[error("connection failed: {0}")]
    ConnectionFailed(String),

    #[error("API error: {0}")]
    Api(String),
}
