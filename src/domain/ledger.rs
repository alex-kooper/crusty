use super::error::LedgerError;
use super::party::{Party, ParticipantId, PartyHint, PartyId};

pub trait Ledger {
    fn list_parties(&self) -> Result<Vec<Party>, LedgerError>;
    fn create_party(&self, hint: Option<&PartyHint>) -> Result<Party, LedgerError>;
    fn get_party(&self, id: &PartyId) -> Result<Party, LedgerError>;
    fn get_participant_id(&self) -> Result<ParticipantId, LedgerError>;
}
