use super::error::LedgerError;
use super::holding::Holding;
use super::party::{Party, ParticipantId, PartyHint, PartyId};
use super::user::User;

pub trait Ledger {
    fn list_parties(&self, hint: Option<&str>) -> Result<Vec<Party>, LedgerError>;
    fn create_party(&self, hint: Option<&PartyHint>) -> Result<Party, LedgerError>;
    fn get_party(&self, id: &PartyId) -> Result<Party, LedgerError>;
    fn get_participant_id(&self) -> Result<ParticipantId, LedgerError>;
    fn get_authenticated_user(&self) -> Result<User, LedgerError>;
    fn query_holdings(&self, party: &PartyId) -> Result<Vec<Holding>, LedgerError>;
}
