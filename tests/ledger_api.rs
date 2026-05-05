#![cfg(feature = "integration")]

use url::Url;

use crusty::config::{AuthConfig, LedgerConfig};
use crusty::domain::error::{LedgerError, PartyError};
use crusty::domain::party::{PartyHint, PartyId};
use crusty::domain::services::{LedgerService, PartyFilter};
use crusty::json_api::JsonApiLedger;

fn get_env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("env var {} must be set", name))
}

fn create_service() -> LedgerService<JsonApiLedger> {
    dotenvy::dotenv().ok();

    let config = LedgerConfig {
        ledger_url: Url::parse(&get_env("LEDGER_API_URL")).expect("invalid LEDGER_API_URL"),
        auth: AuthConfig::ClientCredentials {
            oidc_url: Url::parse(&get_env("OAUTH_OIDC_CONF_URL"))
                .expect("invalid OAUTH_OIDC_CONF_URL"),
            client_id: get_env("OAUTH_CLIENT_ID"),
            client_secret: get_env("OAUTH_CLIENT_SECRET"),
            audience: get_env("OAUTH_AUDIENCE"),
        },
    };

    let ledger = JsonApiLedger::new(config).expect("failed to create ledger");
    LedgerService::new(ledger)
}

#[test]
fn test_get_participant_id() {
    let service = create_service();
    let participant_id = service.get_participant_id().expect("get_participant_id failed");
    let id_str: &str = participant_id.as_ref();
    assert!(!id_str.is_empty(), "participant ID should not be empty");
    println!("\n--- Get Participant ID ---");
    println!("  {}", participant_id);
    println!();
}

#[test]
fn test_list_parties_default_filter() {
    let service = create_service();
    let filter = PartyFilter::default();
    let parties = service.list_parties(&filter).expect("list_parties failed");
    println!("\n--- List Parties: local, no system ({} total) ---", parties.len());
    for party in &parties {
        assert!(party.is_local, "default filter should only return local parties");
        println!("  {}", party.id);
    }
    println!();
}

#[test]
fn test_list_parties_all() {
    let service = create_service();
    let filter = PartyFilter {
        include_remote: true,
        include_system: true,
    };
    let parties = service.list_parties(&filter).expect("list_parties failed");
    assert!(!parties.is_empty(), "should have at least one party");
    println!("\n--- List Parties: all ({} total) ---", parties.len());
    for party in &parties {
        let local_marker = if party.is_local { "local" } else { "remote" };
        println!("  [{}] {}", local_marker, party.id);
    }
    println!();
}

const TEST_PARTY_HINT: &str = "crusty-test";

fn ensure_test_party(service: &LedgerService<JsonApiLedger>) -> PartyId {
    let hint = PartyHint::new(TEST_PARTY_HINT.to_string());
    match service.create_party(Some(&hint)) {
        Ok(party) => {
            println!("  Created new test party: {}", party.id);
            party.id
        }
        Err(LedgerError::Party(PartyError::AlreadyExists(_))) => {
            // Party already exists — find it by hint
            let party = service
                .find_local_party_by_hint(TEST_PARTY_HINT)
                .expect("test party should exist after creation attempt");
            println!("  Test party already exists: {}", party.id);
            party.id
        }
        Err(e) => panic!("unexpected error creating party: {}", e),
    }
}

#[test]
fn test_create_party_is_idempotent() {
    let service = create_service();
    println!("\n--- Create Party (idempotent) ---");
    let party_id = ensure_test_party(&service);
    let id_str: &str = party_id.as_ref();
    assert!(
        id_str.starts_with(TEST_PARTY_HINT),
        "party ID should start with the hint"
    );
    println!();
}

#[test]
fn test_get_party() {
    let service = create_service();
    println!("\n--- Get Party ---");
    let party_id = ensure_test_party(&service);

    let fetched = service.get_party(&party_id).expect("get_party failed");
    assert_eq!(
        fetched.id.as_ref() as &str,
        party_id.as_ref() as &str,
        "fetched party should match created party"
    );
    assert!(fetched.is_local, "test party should be local");
    println!("  Fetched: {}", fetched.id);
    println!();
}

#[test]
fn test_find_local_party_by_hint() {
    let service = create_service();
    println!("\n--- Find Local Party by Hint ---");
    ensure_test_party(&service);

    let found = service
        .find_local_party_by_hint(TEST_PARTY_HINT)
        .expect("find_local_party_by_hint failed");
    let id_str: &str = found.id.as_ref();
    assert!(id_str.starts_with(TEST_PARTY_HINT));
    assert!(found.is_local);
    println!("  Found: {}", found.id);
    println!();
}
