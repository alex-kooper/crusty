use derive_more::{AsRef, Constructor, Display, From};

use super::party::PartyId;

/// A Canton user identifier
#[derive(Constructor, Display, From, AsRef)]
#[as_ref(str)]
pub struct UserId(String);

/// An authenticated user on the participant node
pub struct User {
    pub id: UserId,
    pub username: Option<String>,
    pub primary_party: Option<PartyId>,
}
