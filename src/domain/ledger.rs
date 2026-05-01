use async_trait::async_trait;

use super::error::LedgerError;
use super::party::{Party, ParticipantId, PartyHint, PartyId};

#[async_trait]
pub trait Ledger {
    async fn list_parties(&self) -> Result<Vec<Party>, LedgerError>;
    async fn create_party(&self, hint: &PartyHint) -> Result<Party, LedgerError>;
    async fn get_party(&self, id: &PartyId) -> Result<Party, LedgerError>;
    async fn get_participant_id(&self) -> Result<ParticipantId, LedgerError>;
}
