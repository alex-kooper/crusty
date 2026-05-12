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
        self.client
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .map_err(|e| LedgerError::ConnectionFailed(e.to_string()))
    }

    fn post_json<T: serde::Serialize>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<reqwest::blocking::Response, LedgerError> {
        let url = format!("{}{}", self.base_url, path);
        self.client
            .post(&url)
            .bearer_auth(&self.token)
            .json(body)
            .send()
            .map_err(|e| LedgerError::ConnectionFailed(e.to_string()))
    }

    fn get_ledger_end_offset(&self) -> Result<i64, LedgerError> {
        let resp = self.get("/v2/state/ledger-end")?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().unwrap_or_default();
            return Err(Self::handle_error(status, &body));
        }
        let json: serde_json::Value = resp
            .json()
            .map_err(|e| LedgerError::Api(format!("failed to parse ledger-end: {}", e)))?;
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
        let resp = self.get("/v2/parties")?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().unwrap_or_default();
            return Err(Self::handle_error(status, &body));
        }

        let api_resp: models::ListKnownPartiesResponse = resp
            .json()
            .map_err(|e| LedgerError::Api(format!("failed to parse response: {}", e)))?;

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

        let resp = self.post_json("/v2/parties", &req)?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().unwrap_or_default();
            return Err(Self::handle_error(status, &body));
        }

        let api_resp: models::AllocatePartyResponse = resp
            .json()
            .map_err(|e| LedgerError::Api(format!("failed to parse response: {}", e)))?;

        Ok(to_domain_party(*api_resp.party_details))
    }

    fn get_party(&self, id: &PartyId) -> Result<Party, LedgerError> {
        let path = format!("/v2/parties/{}", id);
        let resp = self.get(&path)?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().unwrap_or_default();
            return Err(Self::handle_error(status, &body));
        }

        let api_resp: models::ListKnownPartiesResponse = resp
            .json()
            .map_err(|e| LedgerError::Api(format!("failed to parse response: {}", e)))?;

        api_resp
            .party_details
            .into_iter()
            .next()
            .map(to_domain_party)
            .ok_or_else(|| LedgerError::Party(PartyError::NotFound(id.to_string())))
    }

    fn get_participant_id(&self) -> Result<ParticipantId, LedgerError> {
        let resp = self.get("/v2/parties/participant-id")?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().unwrap_or_default();
            return Err(Self::handle_error(status, &body));
        }

        let api_resp: models::GetParticipantIdResponse = resp
            .json()
            .map_err(|e| LedgerError::Api(format!("failed to parse response: {}", e)))?;

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

        let resp = self.post_json("/v2/state/active-contracts", &request_body)?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().unwrap_or_default();
            return Err(Self::handle_error(status, &body));
        }

        let items: Vec<serde_json::Value> = resp
            .json()
            .map_err(|e| LedgerError::Api(format!("failed to parse active-contracts: {}", e)))?;

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
                let amount = Amount::parse(v["amount"].as_str().unwrap_or("0"))
                    .map_err(|e| LedgerError::Api(format!("invalid amount: {}", e)))?;

                Ok(Holding {
                    owner: PartyId::new(v["owner"].as_str().unwrap_or_default().to_string()),
                    instrument: InstrumentId {
                        admin: PartyId::new(v["instrumentId"]["admin"].as_str().unwrap_or_default().to_string()),
                        name: InstrumentName::new(v["instrumentId"]["id"].as_str().unwrap_or_default().to_string()),
                    },
                    amount,
                    locked: !v["lock"].is_null(),
                })
            })
            .collect()
    }

    fn get_authenticated_user(&self) -> Result<User, LedgerError> {
        let resp = self.get("/v2/authenticated-user")?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().unwrap_or_default();
            return Err(Self::handle_error(status, &body));
        }

        let api_resp: models::GetUserResponse = resp
            .json()
            .map_err(|e| LedgerError::Api(format!("failed to parse response: {}", e)))?;

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
