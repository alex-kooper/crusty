use std::collections::HashMap;

use derive_more::{AsRef, Constructor, Display, From};

/// A Canton party identifier (e.g. "app_user_quickstart::122069c2...")
#[derive(Constructor, Display, From, AsRef)]
#[as_ref(str)]
pub struct PartyId(String);

/// A hint used when allocating a new party
#[derive(Constructor, Display, From, AsRef)]
#[as_ref(str)]
pub struct PartyHint(String);

/// Party details as known to the participant node
#[derive(Constructor)]
pub struct Party {
    pub id: PartyId,
    pub is_local: bool,
    pub annotations: HashMap<String, String>,
}

/// The participant node's own identifier
#[derive(Constructor, Display, From, AsRef)]
#[as_ref(str)]
pub struct ParticipantId(String);
