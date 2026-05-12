use std::collections::HashMap;

use super::*;

struct MockLedger {
    parties: Vec<Party>,
}

impl Ledger for MockLedger {
    fn list_parties(&self, hint: Option<&str>) -> Result<Vec<Party>, LedgerError> {
        Ok(match hint {
            Some(h) => self
                .parties
                .iter()
                .filter(|p| {
                    let id: &str = p.id.as_ref();
                    id.starts_with(h)
                })
                .cloned()
                .collect(),
            None => self.parties.clone(),
        })
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

    fn get_authenticated_user(&self) -> Result<User, LedgerError> {
        unimplemented!()
    }

    fn query_holdings(&self, _party: &PartyId) -> Result<Vec<Holding>, LedgerError> {
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
        make_party("remote-party::1220ghi", false),
    ]
}

fn mock_service(parties: Vec<Party>) -> LedgerService<MockLedger> {
    LedgerService::new(MockLedger { parties })
}

#[test]
fn list_parties_local_only() {
    let service = mock_service(sample_parties());
    let result = service.list_parties(None, false).unwrap();
    let ids: Vec<&str> = result.iter().map(|p| p.id.as_ref()).collect();
    assert_eq!(ids, vec!["alice::1220abc", "bob::1220def"]);
}

#[test]
fn list_parties_all() {
    let service = mock_service(sample_parties());
    let result = service.list_parties(None, true).unwrap();
    assert_eq!(result.len(), 3);
}

#[test]
fn list_parties_with_hint() {
    let service = mock_service(sample_parties());
    let result = service.list_parties(Some("alice"), false).unwrap();
    let ids: Vec<&str> = result.iter().map(|p| p.id.as_ref()).collect();
    assert_eq!(ids, vec!["alice::1220abc"]);
}

#[test]
fn list_parties_with_hint_all() {
    let service = mock_service(sample_parties());
    let result = service.list_parties(Some("remote"), true).unwrap();
    let ids: Vec<&str> = result.iter().map(|p| p.id.as_ref()).collect();
    assert_eq!(ids, vec!["remote-party::1220ghi"]);
}

#[test]
fn resolve_party_exact_match() {
    let service = mock_service(sample_parties());
    let party = service.resolve_party_by_hint("alice").unwrap();
    assert_eq!(party.id.as_ref() as &str, "alice::1220abc");
}

#[test]
fn resolve_party_no_prefix_collision() {
    let parties = vec![
        make_party("foo::1220aaa", true),
        make_party("foobar::1220bbb", true),
    ];
    let service = mock_service(parties);
    let party = service.resolve_party_by_hint("foo").unwrap();
    assert_eq!(party.id.as_ref() as &str, "foo::1220aaa");
}

#[test]
fn resolve_party_not_found() {
    let service = mock_service(sample_parties());
    let result = service.resolve_party_by_hint("nonexistent");
    assert!(matches!(result, Err(LedgerError::Party(PartyError::NotFound(_)))));
}

#[test]
fn resolve_party_ambiguous() {
    let parties = vec![
        make_party("alice::1220aaa", true),
        make_party("alice::1220bbb", false),
    ];
    let service = mock_service(parties);
    let result = service.resolve_party_by_hint("alice");
    assert!(matches!(result, Err(LedgerError::Party(PartyError::Ambiguous(_)))));
}
