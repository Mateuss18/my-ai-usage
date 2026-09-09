use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::usage_contract::{
    AccountIdentity, AccountUsageSnapshot, ProviderUsage, UsageError, UsageSnapshot, UsageState,
};

pub struct UsageRepository {
    snapshot: UsageSnapshot,
}

impl UsageRepository {
    pub fn load(path: &Path) -> Self {
        let Ok(raw) = fs::read_to_string(path) else {
            return Self::default();
        };
        let Ok(mut snapshot) = serde_json::from_str::<UsageSnapshot>(&raw) else {
            return Self::default();
        };
        if snapshot.schema_version != 2 {
            return Self::default();
        }
        let mut seen_keys = HashSet::new();
        snapshot.accounts.retain(|item| {
            valid_account_snapshot(item) && seen_keys.insert(item.account.key.clone())
        });
        for item in &mut snapshot.accounts {
            if matches!(
                item.usage.state,
                UsageState::Available | UsageState::Partial
            ) {
                item.usage.state = UsageState::Stale;
                item.usage.error = None;
            }
        }
        snapshot.active_account_key = None;
        snapshot.error = snapshot.error.filter(|error| error.is_controlled());
        Self { snapshot }
    }

    pub fn snapshot(&self) -> &UsageSnapshot {
        &self.snapshot
    }

    pub fn record_success(
        &mut self,
        account: AccountIdentity,
        usage: ProviderUsage,
        fetched_at: String,
    ) {
        for item in &mut self.snapshot.accounts {
            if item.account.key != account.key && valid_usage(&item.usage) {
                item.usage.state = UsageState::Stale;
                item.usage.error = None;
            }
        }

        let key = account.key.clone();
        let mut usage = usage;
        usage.error = None;
        let item = AccountUsageSnapshot {
            account,
            usage,
            fetched_at: Some(fetched_at.clone()),
        };
        if let Some(existing) = self
            .snapshot
            .accounts
            .iter_mut()
            .find(|item| item.account.key == key)
        {
            *existing = item;
        } else {
            self.snapshot.accounts.push(item);
        }
        self.snapshot.active_account_key = Some(key);
        self.snapshot.fetched_at = Some(fetched_at);
        self.snapshot.error = None;
    }

    pub fn record_error(
        &mut self,
        account: AccountIdentity,
        error: UsageError,
        fetched_at: String,
    ) {
        if !error.is_controlled() {
            return;
        }
        let key = account.key.clone();
        if let Some(existing) = self
            .snapshot
            .accounts
            .iter_mut()
            .find(|item| item.account.key == key)
        {
            existing.account = account;
            existing.usage.state = UsageState::Stale;
            existing.usage.error = Some(error);
        } else {
            self.snapshot.accounts.push(AccountUsageSnapshot {
                account,
                usage: empty_usage(UsageState::Stale, Some(error)),
                fetched_at: None,
            });
        }
        self.snapshot.active_account_key = Some(key);
        self.snapshot.fetched_at = Some(fetched_at);
        self.snapshot.error = None;
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let parent = path.parent().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "snapshot path has no parent")
        })?;
        fs::create_dir_all(parent)?;
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos());
        let temp = parent.join(format!(
            ".usage-snapshots-{suffix}-{}.tmp",
            std::process::id()
        ));
        let result = (|| {
            let mut file = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&temp)?;
            let data = serde_json::to_vec(&self.snapshot)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
            file.write_all(&data)?;
            file.sync_all()?;
            match fs::rename(&temp, path) {
                Ok(()) => Ok(()),
                Err(rename_error) => {
                    fs::remove_file(path).map_err(|_| rename_error)?;
                    fs::rename(&temp, path)
                }
            }
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temp);
        }
        result
    }

    pub fn record_root_error(&mut self, error: UsageError, fetched_at: String) {
        if !error.is_controlled() {
            return;
        }
        for item in &mut self.snapshot.accounts {
            if valid_usage(&item.usage) {
                item.usage.state = UsageState::Stale;
                item.usage.error = None;
            }
        }
        self.snapshot.active_account_key = None;
        self.snapshot.fetched_at = Some(fetched_at);
        self.snapshot.error = Some(error);
    }
}

fn valid_account_snapshot(item: &AccountUsageSnapshot) -> bool {
    let Some(email) = item.account.email.as_deref().map(str::trim) else {
        return false;
    };
    !email.is_empty()
        && item.account.key == format!("codex:{}", email.to_ascii_lowercase())
        && item.account.provider == crate::usage_contract::Provider::Codex
        && item.usage.id == item.account.provider
        && item.usage.schema_version == 1
        && item
            .usage
            .error
            .as_ref()
            .is_none_or(UsageError::is_controlled)
}

fn valid_usage(usage: &ProviderUsage) -> bool {
    matches!(usage.state, UsageState::Available | UsageState::Partial) && !usage.quotas.is_empty()
}

fn empty_usage(state: UsageState, error: Option<UsageError>) -> ProviderUsage {
    ProviderUsage {
        schema_version: 1,
        id: crate::usage_contract::Provider::Codex,
        name: "Codex".into(),
        vendor: "OpenAI".into(),
        state,
        captured_at: None,
        quotas: Vec::new(),
        error,
    }
}

impl Default for UsageRepository {
    fn default() -> Self {
        Self {
            snapshot: UsageSnapshot {
                schema_version: 2,
                accounts: Vec::new(),
                active_account_key: None,
                fetched_at: None,
                error: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "my-ai-usage-{name}-{}-{}.json",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    fn account(email: &str) -> AccountIdentity {
        AccountIdentity {
            key: format!("codex:{email}"),
            provider: crate::usage_contract::Provider::Codex,
            email: Some(email.into()),
            account_type: Some("chatgpt".into()),
            plan: Some("plus".into()),
        }
    }

    fn usage(percentage: f64) -> ProviderUsage {
        ProviderUsage {
            schema_version: 1,
            id: crate::usage_contract::Provider::Codex,
            name: "Codex".into(),
            vendor: "OpenAI".into(),
            state: UsageState::Available,
            captured_at: Some("2026-09-09T12:00:00Z".into()),
            quotas: vec![crate::usage_contract::UsageQuota {
                id: "session".into(),
                label: "5 hours / session".into(),
                used: None,
                limit: None,
                percentage: Some(percentage),
                reset_at: Some("2026-09-09T13:00:00Z".into()),
            }],
            error: None,
        }
    }

    #[test]
    fn upserts_accounts_marks_previous_valid_usage_stale_and_does_not_duplicate() {
        let mut repository = UsageRepository::default();
        repository.record_success(account("a@example.com"), usage(10.0), "a1".into());
        repository.record_success(account("b@example.com"), usage(20.0), "b1".into());
        repository.record_success(account("a@example.com"), usage(30.0), "a2".into());

        let snapshot = repository.snapshot();
        assert_eq!(snapshot.accounts.len(), 2);
        assert_eq!(
            snapshot.active_account_key.as_deref(),
            Some("codex:a@example.com")
        );
        let a = snapshot
            .accounts
            .iter()
            .find(|item| item.account.key == "codex:a@example.com")
            .unwrap();
        let b = snapshot
            .accounts
            .iter()
            .find(|item| item.account.key == "codex:b@example.com")
            .unwrap();
        assert_eq!(a.usage.quotas[0].percentage, Some(30.0));
        assert_eq!(a.usage.state, UsageState::Available);
        assert_eq!(b.usage.state, UsageState::Stale);
    }

    #[test]
    fn known_account_error_preserves_previous_quotas_and_marks_it_stale() {
        let mut repository = UsageRepository::default();
        repository.record_success(account("a@example.com"), usage(10.0), "a1".into());
        repository.record_error(
            account("a@example.com"),
            UsageError {
                code: "timeout".into(),
                message: "Codex did not respond in time.".into(),
            },
            "a2".into(),
        );

        let saved = &repository.snapshot().accounts[0];
        assert_eq!(saved.usage.state, UsageState::Stale);
        assert_eq!(saved.usage.quotas[0].percentage, Some(10.0));
        assert_eq!(saved.usage.error.as_ref().unwrap().code, "timeout");
    }

    #[test]
    fn malformed_persistence_loads_empty_without_panicking() {
        let file = path("malformed");
        fs::write(&file, "not json").unwrap();
        let repository = UsageRepository::load(&file);
        assert!(repository.snapshot().accounts.is_empty());
        assert_eq!(repository.snapshot().schema_version, 2);
        let _ = fs::remove_file(file);
    }

    #[test]
    fn persisted_available_accounts_reload_as_stale_without_duplicate_keys() {
        let file = path("reload");
        let mut repository = UsageRepository::default();
        repository.record_success(account("a@example.com"), usage(10.0), "a1".into());
        repository.save(&file).unwrap();

        let mut json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&file).unwrap()).unwrap();
        let accounts = json["accounts"].as_array_mut().unwrap();
        let duplicate = accounts[0].clone();
        accounts.push(duplicate);
        fs::write(&file, serde_json::to_vec(&json).unwrap()).unwrap();

        let loaded = UsageRepository::load(&file);
        assert_eq!(loaded.snapshot().accounts.len(), 1);
        assert_eq!(loaded.snapshot().accounts[0].usage.state, UsageState::Stale);
        assert_eq!(loaded.snapshot().active_account_key, None);
        let _ = fs::remove_file(file);
    }

    #[test]
    fn root_error_marks_cached_accounts_stale_and_clears_active_account() {
        let mut repository = UsageRepository::default();
        repository.record_success(account("a@example.com"), usage(10.0), "a1".into());
        repository.record_root_error(
            UsageError {
                code: "unauthenticated".into(),
                message: "Sign in to Codex to read usage.".into(),
            },
            "a2".into(),
        );

        let saved = &repository.snapshot().accounts[0];
        assert_eq!(saved.usage.state, UsageState::Stale);
        assert_eq!(saved.usage.quotas[0].percentage, Some(10.0));
        assert_eq!(repository.snapshot().active_account_key, None);
        assert_eq!(
            repository.snapshot().error.as_ref().unwrap().code,
            "unauthenticated"
        );
    }

    #[test]
    fn persisted_json_contains_only_sanitized_dto_fields() {
        let file = path("safe");
        let mut repository = UsageRepository::default();
        repository.record_success(account("a@example.com"), usage(10.0), "a1".into());
        repository.save(&file).unwrap();
        let json = fs::read_to_string(&file).unwrap().to_ascii_lowercase();
        for secret_field in ["token", "cookie", "auth", "refresh_token", "access_token"] {
            assert!(
                !json.contains(secret_field),
                "found {secret_field} in persisted DTO"
            );
        }
        let _ = fs::remove_file(file);
    }
}
