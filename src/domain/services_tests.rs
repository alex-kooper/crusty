use std::collections::HashMap;

use super::*;

struct MockLedger {
    parties: Vec<Party>,
}

impl Ledger for MockLedger {
    fn list_parties(&self) -> Result<Vec<Party>, LedgerError> {
        Ok(self.parties.clone())
    }

    fn create_party(&self, _hint: Option<&PartyHint>) -> Result<Party, LedgerError> {
        unimplemented!()
    }

    fn get_party(&self, _id: &PartyId) -> Result<Party, LedgerError> {
        unimplemented!()
    }

    fn get_participant_id(&self) -> Result<ParticipantId, LedgerError> {
        unimplemented!()
    }
}

fn make_party(id: &str, is_local: bool) -> Party {
    Party::new(PartyId::new(id.to_string()), is_local, HashMap::new())
}

fn sample_parties() -> Vec<Party> {
    vec![
        make_party("alice::1220abc", true),
        make_party("bob::1220def", true),
        make_party("participant::1220abc", true),
        make_party("DSO::1220xyz", false),
        make_party("sv::1220uvw", false),
        make_party("remote-party::1220ghi", false),
    ]
}

fn mock_service(parties: Vec<Party>) -> LedgerService<MockLedger> {
    LedgerService::new(MockLedger { parties })
}

#[test]
fn list_parties_default_returns_local_non_system() {
    let service = mock_service(sample_parties());
    let result = service.list_parties(&PartyFilter::default()).unwrap();
    let ids: Vec<&str> = result.iter().map(|p| p.id.as_ref()).collect();
    assert_eq!(ids, vec!["alice::1220abc", "bob::1220def"]);
}

#[test]
fn list_parties_include_remote() {
    let service = mock_service(sample_parties());
    let filter = PartyFilter { include_remote: true, include_system: false };
    let result = service.list_parties(&filter).unwrap();
    let ids: Vec<&str> = result.iter().map(|p| p.id.as_ref()).collect();
    assert_eq!(ids, vec!["alice::1220abc", "bob::1220def", "remote-party::1220ghi"]);
}

#[test]
fn list_parties_include_system() {
    let service = mock_service(sample_parties());
    let filter = PartyFilter { include_remote: false, include_system: true };
    let result = service.list_parties(&filter).unwrap();
    let ids: Vec<&str> = result.iter().map(|p| p.id.as_ref()).collect();
    assert_eq!(ids, vec!["alice::1220abc", "bob::1220def", "participant::1220abc"]);
}

#[test]
fn list_parties_include_all() {
    let service = mock_service(sample_parties());
    let filter = PartyFilter { include_remote: true, include_system: true };
    let result = service.list_parties(&filter).unwrap();
    assert_eq!(result.len(), 6);
}

#[test]
fn find_by_hint_exact_match() {
    let service = mock_service(sample_parties());
    let party = service.find_local_party_by_hint("alice").unwrap();
    assert_eq!(party.id.as_ref() as &str, "alice::1220abc");
}

#[test]
fn find_by_hint_no_prefix_collision() {
    let parties = vec![
        make_party("foo::1220aaa", true),
        make_party("foobar::1220bbb", true),
    ];
    let service = mock_service(parties);
    let party = service.find_local_party_by_hint("foo").unwrap();
    assert_eq!(party.id.as_ref() as &str, "foo::1220aaa");
}

#[test]
fn find_by_hint_not_found() {
    let service = mock_service(sample_parties());
    let result = service.find_local_party_by_hint("nonexistent");
    assert!(matches!(result, Err(LedgerError::Party(PartyError::NotFound(_)))));
}

#[test]
fn find_by_hint_ignores_remote() {
    let service = mock_service(sample_parties());
    let result = service.find_local_party_by_hint("remote-party");
    assert!(matches!(result, Err(LedgerError::Party(PartyError::NotFound(_)))));
}
