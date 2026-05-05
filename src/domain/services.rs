use super::error::{LedgerError, PartyError};
use super::ledger::Ledger;
use super::party::{Party, ParticipantId, PartyHint, PartyId};

#[derive(Default)]
pub struct PartyFilter {
    pub include_remote: bool,
    pub include_system: bool,
}

pub struct LedgerService<L: Ledger> {
    ledger: L,
}

impl<L: Ledger> LedgerService<L> {
    pub fn new(ledger: L) -> Self {
        Self { ledger }
    }

    pub fn list_parties(&self, filter: &PartyFilter) -> Result<Vec<Party>, LedgerError> {
        let parties = self.ledger.list_parties()?;
        Ok(parties
            .into_iter()
            .filter(|p| filter.include_remote || p.is_local)
            .filter(|p| filter.include_system || !is_system_party(p))
            .collect())
    }

    pub fn find_local_party_by_hint(&self, hint: &str) -> Result<Party, LedgerError> {
        let parties = self.ledger.list_parties()?;
        parties
            .into_iter()
            .find(|p| {
                let id: &str = p.id.as_ref();
                p.is_local && id.starts_with(&format!("{}::", hint))
            })
            .ok_or_else(|| LedgerError::Party(PartyError::NotFound(hint.to_string())))
    }

    pub fn create_party(&self, hint: Option<&PartyHint>) -> Result<Party, LedgerError> {
        self.ledger.create_party(hint)
    }

    pub fn get_party(&self, id: &PartyId) -> Result<Party, LedgerError> {
        self.ledger.get_party(id)
    }

    pub fn get_participant_id(&self) -> Result<ParticipantId, LedgerError> {
        self.ledger.get_participant_id()
    }
}

const SYSTEM_PARTY_PREFIXES: &[&str] = &["participant::", "DSO::", "sv::"];

fn is_system_party(party: &Party) -> bool {
    let id: &str = party.id.as_ref();
    SYSTEM_PARTY_PREFIXES.iter().any(|prefix| id.starts_with(prefix))
}
