use canton_api_client::models;
use reqwest::blocking::Client;
use reqwest::StatusCode;
use url::Url;

use crate::auth;
use crate::config::LedgerConfig;
use crate::domain::error::{LedgerError, PartyError};
use crate::domain::holding::{Amount, Holding, InstrumentId, InstrumentName};
use crate::domain::ledger::Ledger;
use crate::domain::party::{Party, ParticipantId, PartyHint, PartyId};
use crate::domain::user::{User, UserId};

pub struct JsonApiLedger {
    client: Client,
    base_url: Url,
    token: String,
}

impl JsonApiLedger {
    pub fn new(config: LedgerConfig) -> Result<Self, LedgerError> {
        let token = auth::obtain_token(&config.auth)?;
        Ok(Self {
            client: Client::new(),
            base_url: config.ledger_url,
            token,
        })
    }

    fn get(&self, path: &str) -> Result<reqwest::blocking::Response, LedgerError> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self.client
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .map_err(|e| LedgerError::ConnectionFailed(e.to_string()))?;
        Self::check_response(resp)
    }

    fn post_json<T: serde::Serialize>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<reqwest::blocking::Response, LedgerError> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self.client
            .post(&url)
            .bearer_auth(&self.token)
            .json(body)
            .send()
            .map_err(|e| LedgerError::ConnectionFailed(e.to_string()))?;
        Self::check_response(resp)
    }

    fn check_response(
        resp: reqwest::blocking::Response,
    ) -> Result<reqwest::blocking::Response, LedgerError> {
        let status = resp.status();
        if status.is_success() {
            Ok(resp)
        } else {
            let body = resp.text().unwrap_or_default();
            Err(Self::handle_error(status, &body))
        }
    }

    fn parse_json<T: serde::de::DeserializeOwned>(
        resp: reqwest::blocking::Response,
    ) -> Result<T, LedgerError> {
        resp.json()
            .map_err(|e| LedgerError::Api(format!("failed to parse response: {}", e)))
    }

    fn get_ledger_end_offset(&self) -> Result<i64, LedgerError> {
        let json: serde_json::Value = Self::parse_json(self.get("/v2/state/ledger-end")?)?;
        json["offset"]
            .as_i64()
            .ok_or_else(|| LedgerError::Api("missing offset in ledger-end".to_string()))
    }

    fn handle_error(status: StatusCode, body: &str) -> LedgerError {
        match status {
            StatusCode::UNAUTHORIZED => LedgerError::Unauthorized,
            StatusCode::NOT_FOUND => {
                LedgerError::Party(PartyError::NotFound(body.to_string()))
            }
            StatusCode::CONFLICT => {
                LedgerError::Party(PartyError::AlreadyExists(body.to_string()))
            }
            StatusCode::BAD_REQUEST if body.contains("already exists") => {
                LedgerError::Party(PartyError::AlreadyExists(body.to_string()))
            }
            _ => LedgerError::Api(format!("HTTP {}: {}", status, body)),
        }
    }
}

fn to_domain_party(p: models::PartyDetails) -> Party {
    Party::new(
        PartyId::new(p.party),
        p.is_local.unwrap_or(false),
        p.local_metadata
            .map(|m| m.annotations.unwrap_or_default())
            .unwrap_or_default(),
    )
}

impl Ledger for JsonApiLedger {
    fn list_parties(&self, hint: Option<&str>) -> Result<Vec<Party>, LedgerError> {
        let api_resp: models::ListKnownPartiesResponse =
            Self::parse_json(self.get("/v2/parties")?)?;

        let parties = api_resp.party_details.into_iter().map(to_domain_party);
        Ok(match hint {
            Some(h) => parties
                .filter(|p| {
                    let id: &str = p.id.as_ref();
                    id.starts_with(h)
                })
                .collect(),
            None => parties.collect(),
        })
    }

    fn create_party(&self, hint: Option<&PartyHint>) -> Result<Party, LedgerError> {
        let mut req = models::AllocatePartyRequest::new();
        req.party_id_hint = hint.map(|h| h.as_ref().to_string());

        let api_resp: models::AllocatePartyResponse =
            Self::parse_json(self.post_json("/v2/parties", &req)?)?;

        Ok(to_domain_party(*api_resp.party_details))
    }

    fn get_party(&self, id: &PartyId) -> Result<Party, LedgerError> {
        let path = format!("/v2/parties/{}", id);
        let api_resp: models::ListKnownPartiesResponse =
            Self::parse_json(self.get(&path)?)?;

        api_resp
            .party_details
            .into_iter()
            .next()
            .map(to_domain_party)
            .ok_or_else(|| LedgerError::Party(PartyError::NotFound(id.to_string())))
    }

    fn get_participant_id(&self) -> Result<ParticipantId, LedgerError> {
        let api_resp: models::GetParticipantIdResponse =
            Self::parse_json(self.get("/v2/parties/participant-id")?)?;

        Ok(ParticipantId::new(api_resp.participant_id))
    }

    fn query_holdings(&self, party: &PartyId) -> Result<Vec<Holding>, LedgerError> {
        let offset = self.get_ledger_end_offset()?;
        let party_id = party.to_string();
        let request_body = serde_json::json!({
            "activeAtOffset": offset,
            "eventFormat": {
                "filtersByParty": {
                    party_id: {
                        "cumulative": [{
                            "identifierFilter": {
                                "InterfaceFilter": {
                                    "value": {
                                        "interfaceId": "#splice-api-token-holding-v1:Splice.Api.Token.HoldingV1:Holding",
                                        "includeInterfaceView": true,
                                        "includeCreatedEventBlob": false
                                    }
                                }
                            }
                        }]
                    }
                },
                "verbose": true
            }
        });

        let items: Vec<serde_json::Value> =
            Self::parse_json(self.post_json("/v2/state/active-contracts", &request_body)?)?;

        items
            .iter()
            .flat_map(|item| {
                item["contractEntry"]["JsActiveContract"]["createdEvent"]["interfaceViews"]
                    .as_array()
                    .into_iter()
                    .flatten()
            })
            .filter_map(|view| view.get("viewValue"))
            .map(|v| {
                let amount_str = v["amount"].as_str()
                    .ok_or_else(|| LedgerError::Api("missing 'amount' in holding".to_string()))?;
                let amount = Amount::parse(amount_str)
                    .map_err(|e| LedgerError::Api(format!("invalid amount '{}': {}", amount_str, e)))?;
                let owner = v["owner"].as_str()
                    .ok_or_else(|| LedgerError::Api("missing 'owner' in holding".to_string()))?;
                let admin = v["instrumentId"]["admin"].as_str()
                    .ok_or_else(|| LedgerError::Api("missing 'instrumentId.admin' in holding".to_string()))?;
                let instrument_name = v["instrumentId"]["id"].as_str()
                    .ok_or_else(|| LedgerError::Api("missing 'instrumentId.id' in holding".to_string()))?;

                Ok(Holding {
                    owner: PartyId::new(owner.to_string()),
                    instrument: InstrumentId {
                        admin: PartyId::new(admin.to_string()),
                        name: InstrumentName::new(instrument_name.to_string()),
                    },
                    amount,
                    locked: !v["lock"].is_null(),
                })
            })
            .collect()
    }

    fn get_authenticated_user(&self) -> Result<User, LedgerError> {
        let api_resp: models::GetUserResponse =
            Self::parse_json(self.get("/v2/authenticated-user")?)?;

        let u = api_resp.user;
        let username = u
            .metadata
            .and_then(|m| m.annotations)
            .and_then(|a| a.get("username").cloned());
        let primary_party = u
            .primary_party
            .filter(|s| !s.is_empty())
            .map(PartyId::new);

        Ok(User {
            id: UserId::new(u.id),
            username,
            primary_party,
        })
    }
}
