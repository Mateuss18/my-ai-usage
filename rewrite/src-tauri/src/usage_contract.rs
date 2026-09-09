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
    pub account_name: Option<String>,
    pub state: UsageState,
    pub captured_at: Option<String>,
    pub quotas: Vec<UsageQuota>,
    pub error: Option<UsageError>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageSnapshot {
    pub schema_version: u8,
    pub providers: Vec<ProviderUsage>,
    pub fetched_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageError {
    pub code: String,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

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
