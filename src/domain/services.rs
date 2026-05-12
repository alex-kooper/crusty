use super::error::{LedgerError, PartyError};
use super::ledger::Ledger;
use super::party::{Party, ParticipantId, PartyHint, PartyId};

pub struct LedgerService<L: Ledger> {
    ledger: L,
}

impl<L: Ledger> LedgerService<L> {
    pub fn new(ledger: L) -> Self {
        Self { ledger }
    }

    pub fn list_parties(
        &self,
        hint: Option<&str>,
        include_remote: bool,
    ) -> Result<Vec<Party>, LedgerError> {
        let parties = self.ledger.list_parties(hint)?;
        Ok(if include_remote {
            parties
        } else {
            parties.into_iter().filter(|p| p.is_local).collect()
        })
    }

    pub fn resolve_party_by_hint(&self, hint: &str) -> Result<Party, LedgerError> {
        let parties = self.ledger.list_parties(Some(hint))?;
        let prefix = format!("{}::", hint);
        let mut matches: Vec<Party> = parties
            .into_iter()
            .filter(|p| {
                let id: &str = p.id.as_ref();
                id.starts_with(&prefix)
            })
            .collect();

        match matches.len() {
            0 => Err(LedgerError::Party(PartyError::NotFound(hint.to_string()))),
            1 => Ok(matches.remove(0)),
            _ => Err(LedgerError::Party(PartyError::Ambiguous(hint.to_string()))),
        }
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

#[cfg(test)]
#[path = "services_tests.rs"]
mod tests;
