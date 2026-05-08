use super::*;

fn sample_config() -> ConfigFile {
    let mut profiles = BTreeMap::new();
    profiles.insert("quickstart".to_string(), ConfigFile::quickstart_profile());
    profiles.insert(
        "localnet".to_string(),
        Profile {
            ledger_url: "http://localhost:6575".to_string(),
            auth: ProfileAuth::Token {
                token: "test-jwt-token".to_string(),
            },
        },
    );
    ConfigFile {
        default_profile: Some("quickstart".to_string()),
        profiles,
    }
}

#[test]
fn get_profile_returns_default() {
    let config = sample_config();
    let profile = config.get_profile(None).unwrap();
    assert_eq!(profile.ledger_url, "http://localhost:2975");
}

#[test]
fn get_profile_returns_named() {
    let config = sample_config();
    let profile = config.get_profile(Some("localnet")).unwrap();
    assert_eq!(profile.ledger_url, "http://localhost:6575");
}

#[test]
fn get_profile_named_overrides_default() {
    let config = sample_config();
    let profile = config.get_profile(Some("localnet")).unwrap();
    assert_eq!(profile.ledger_url, "http://localhost:6575");
}

#[test]
fn get_profile_errors_on_missing() {
    let config = sample_config();
    let result = config.get_profile(Some("nonexistent"));
    assert!(result.is_err());
}

#[test]
fn get_profile_errors_when_no_default() {
    let config = ConfigFile {
        default_profile: None,
        profiles: BTreeMap::new(),
    };
    let result = config.get_profile(None);
    assert!(result.is_err());
}

#[test]
fn to_ledger_config_client_credentials() {
    let profile = ConfigFile::quickstart_profile();
    let config = profile.to_ledger_config().unwrap();
    assert_eq!(config.ledger_url.as_str(), "http://localhost:2975/");
    assert!(matches!(config.auth, AuthConfig::ClientCredentials { .. }));
}

#[test]
fn to_ledger_config_token() {
    let profile = Profile {
        ledger_url: "http://localhost:6575".to_string(),
        auth: ProfileAuth::Token {
            token: "my-token".to_string(),
        },
    };
    let config = profile.to_ledger_config().unwrap();
    assert_eq!(config.ledger_url.as_str(), "http://localhost:6575/");
    assert!(matches!(config.auth, AuthConfig::Token(ref t) if t == "my-token"));
}

#[test]
fn to_ledger_config_rejects_invalid_url() {
    let profile = Profile {
        ledger_url: "not-a-url".to_string(),
        auth: ProfileAuth::Token {
            token: "x".to_string(),
        },
    };
    assert!(profile.to_ledger_config().is_err());
}

#[test]
fn toml_round_trip_client_credentials() {
    let config = sample_config();
    let toml_str = toml::to_string_pretty(&config).unwrap();
    let parsed: ConfigFile = toml::from_str(&toml_str).unwrap();
    assert_eq!(parsed.default_profile, Some("quickstart".to_string()));
    assert!(parsed.profiles.contains_key("quickstart"));
    assert!(parsed.profiles.contains_key("localnet"));
}

#[test]
fn toml_round_trip_token_profile() {
    let config = ConfigFile {
        default_profile: Some("dev".to_string()),
        profiles: BTreeMap::from([(
            "dev".to_string(),
            Profile {
                ledger_url: "http://localhost:1234".to_string(),
                auth: ProfileAuth::Token {
                    token: "abc123".to_string(),
                },
            },
        )]),
    };
    let toml_str = toml::to_string_pretty(&config).unwrap();
    let parsed: ConfigFile = toml::from_str(&toml_str).unwrap();
    let profile = parsed.get_profile(None).unwrap();
    assert!(matches!(&profile.auth, ProfileAuth::Token { token } if token == "abc123"));
}
