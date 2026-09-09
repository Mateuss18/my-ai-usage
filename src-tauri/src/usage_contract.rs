use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Provider {
    Codex,
    #[serde(rename = "opencode")]
    OpenCode,
    Claude,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UsageState {
    Loading,
    Available,
    Partial,
    Stale,
    Unauthenticated,
    NotInstalled,
    Error,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageQuota {
    pub id: String,
    pub label: String,
    pub used: Option<f64>,
    pub limit: Option<f64>,
    pub percentage: Option<f64>,
    pub reset_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderUsage {
    pub schema_version: u8,
    pub id: Provider,
    pub name: String,
    pub vendor: String,
    pub state: UsageState,
    pub captured_at: Option<String>,
    pub quotas: Vec<UsageQuota>,
    pub error: Option<UsageError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountIdentity {
    pub key: String,
    pub provider: Provider,
    pub email: Option<String>,
    pub account_type: Option<String>,
    pub plan: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountUsageSnapshot {
    pub account: AccountIdentity,
    pub usage: ProviderUsage,
    pub fetched_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageSnapshot {
    pub schema_version: u8,
    pub accounts: Vec<AccountUsageSnapshot>,
    pub active_account_key: Option<String>,
    pub fetched_at: Option<String>,
    pub error: Option<UsageError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageError {
    pub code: String,
    pub message: String,
}

impl UsageError {
    pub(crate) fn is_controlled(&self) -> bool {
        matches!(
            (self.code.as_str(), self.message.as_str()),
            ("not-installed", "Codex is not installed.")
                | ("unauthenticated", "Sign in to Codex to read usage.")
                | ("timeout", "Codex did not respond in time.")
                | (
                    "end-of-stream",
                    "Codex app-server stopped before responding."
                )
                | ("invalid-json", "Codex returned an invalid response.")
                | ("protocol-error", "Codex returned an unsupported response.")
                | ("partial-data", "Codex did not return usable quota windows.")
                | ("missing-identity", "Codex account identity is unavailable.")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn usage() -> ProviderUsage {
        ProviderUsage {
            schema_version: 1,
            id: Provider::Codex,
            name: "Codex".into(),
            vendor: "OpenAI".into(),
            state: UsageState::Available,
            captured_at: Some("2026-09-09T12:00:00Z".into()),
            quotas: Vec::new(),
            error: None,
        }
    }

    #[test]
    fn serializes_the_v2_account_contract_without_identity_inside_provider_usage() {
        let snapshot = UsageSnapshot {
            schema_version: 2,
            accounts: vec![AccountUsageSnapshot {
                account: AccountIdentity {
                    key: "codex:owner@example.com".into(),
                    provider: Provider::Codex,
                    email: Some("owner@example.com".into()),
                    account_type: Some("chatgpt".into()),
                    plan: Some("plus".into()),
                },
                usage: usage(),
                fetched_at: Some("2026-09-09T12:00:00Z".into()),
            }],
            active_account_key: Some("codex:owner@example.com".into()),
            fetched_at: Some("2026-09-09T12:00:00Z".into()),
            error: None,
        };

        let json = serde_json::to_value(&snapshot).unwrap();
        assert_eq!(json["schemaVersion"], 2);
        assert!(json["accounts"][0]["account"]["key"].is_string());
        assert!(json["accounts"][0]["usage"].get("account").is_none());
        assert!(json["error"].is_null());
    }

    #[test]
    fn round_trip_preserves_unknown_values_as_null() {
        let dto = UsageQuota {
            id: "limit".into(),
            label: "Provider limit".into(),
            used: None,
            limit: None,
            percentage: None,
            reset_at: None,
        };
        let json = serde_json::to_string(&dto).unwrap();
        assert!(json.contains("\"percentage\":null"));
        assert_eq!(serde_json::from_str::<UsageQuota>(&json).unwrap(), dto);
    }
}
