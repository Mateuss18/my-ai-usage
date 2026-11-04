use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::codex_provider::{CodexProvider, LoginStatus, ProviderError};
use crate::usage_contract::UsageSnapshot;
use crate::usage_repository::UsageRepository;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Profile {
    id: String,
    home: PathBuf,
}

struct ProfileProvider {
    home: PathBuf,
    provider: CodexProvider,
}

#[derive(Default)]
pub struct CodexAccounts {
    providers: HashMap<String, ProfileProvider>,
    pending_profile: Option<String>,
}

impl CodexAccounts {
    fn profiles(&self) -> Vec<Profile> {
        let user = std::env::var_os("USERPROFILE")
            .map(PathBuf::from)
            .unwrap_or_default();
        let settings = fs::read_to_string(user.join(".t3/userdata/settings.json"))
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or(Value::Null);
        profiles(
            &user,
            std::env::var_os("CODEX_HOME").map(PathBuf::from),
            &settings,
        )
    }

    fn provider(&mut self, profile: &Profile) -> &mut CodexProvider {
        let entry = self
            .providers
            .entry(profile.id.clone())
            .or_insert_with(|| ProfileProvider {
                home: profile.home.clone(),
                provider: CodexProvider::with_home(profile.home.clone()),
            });
        if entry.home != profile.home {
            entry.provider.shutdown();
            entry.home = profile.home.clone();
            entry.provider = CodexProvider::with_home(profile.home.clone());
        }
        &mut entry.provider
    }

    fn resolve(&self, id: Option<&str>) -> Result<Profile, ProviderError> {
        let id = id.unwrap_or("codex");
        self.profiles()
            .into_iter()
            .find(|profile| profile.id == id)
            .ok_or(ProviderError::Protocol)
    }

    pub fn usage(&mut self, repository: &mut UsageRepository) -> UsageSnapshot {
        let profiles = self.profiles();
        self.providers
            .retain(|id, _| profiles.iter().any(|profile| &profile.id == id));
        let mut error = None;
        for profile in profiles {
            if profile.id != "codex" && !profile.home.is_dir() {
                continue;
            }
            if self.pending_profile.as_deref() == Some(&profile.id) {
                continue;
            }
            let mut scoped = repository.for_profile(&profile.id);
            let snapshot = self.provider(&profile).usage(&mut scoped);
            if let Some(profile_error) = &snapshot.error {
                error.get_or_insert_with(|| profile_error.clone());
            }
            repository.merge_profile(&profile.id, snapshot);
        }
        repository.set_refresh_error(error);
        repository.snapshot().clone()
    }

    pub fn start_login(&mut self, id: Option<&str>) -> Result<String, ProviderError> {
        if self.pending_profile.is_some() {
            return Err(ProviderError::Protocol);
        }
        let profile = self.resolve(id)?;
        fs::create_dir_all(&profile.home).map_err(|_| ProviderError::Protocol)?;
        let url = self.provider(&profile).start_login()?;
        self.pending_profile = Some(profile.id);
        Ok(url)
    }

    pub fn poll_login(&mut self) -> Result<LoginStatus, ProviderError> {
        let id = self
            .pending_profile
            .clone()
            .ok_or(ProviderError::Protocol)?;
        let result = self
            .providers
            .get_mut(&id)
            .ok_or(ProviderError::Protocol)
            .and_then(|entry| entry.provider.poll_login());
        if !matches!(result, Ok(LoginStatus::Pending)) {
            self.pending_profile = None;
        }
        result
    }

    pub fn cancel_login(&mut self) -> Result<(), ProviderError> {
        let id = self.pending_profile.take().ok_or(ProviderError::Protocol)?;
        self.providers
            .get_mut(&id)
            .ok_or(ProviderError::Protocol)?
            .provider
            .cancel_login()
    }

    pub fn logout(&mut self, id: Option<&str>) -> Result<(), ProviderError> {
        let profile = self.resolve(id)?;
        self.provider(&profile).logout()
    }

    pub fn logout_account_if_matches(
        &mut self,
        id: Option<&str>,
        key: &str,
    ) -> Result<(), ProviderError> {
        let profile = self.resolve(id)?;
        let provider = self.provider(&profile);
        match provider.current_account_key() {
            Ok(current_key) if current_key == key => provider.logout(),
            Ok(_) | Err(ProviderError::Unauthenticated) => Ok(()),
            Err(error) => Err(error),
        }
    }

    pub fn shutdown(&mut self) {
        self.pending_profile = None;
        for entry in self.providers.values_mut() {
            entry.provider.shutdown();
        }
    }
}

fn configured_home(value: &Value, user: &Path) -> Option<PathBuf> {
    let value = value.as_str()?.trim();
    if value.is_empty() {
        return None;
    }
    let home = if value == "~" {
        user.to_path_buf()
    } else if let Some(relative) = value
        .strip_prefix("~/")
        .or_else(|| value.strip_prefix("~\\"))
    {
        user.join(relative)
    } else {
        PathBuf::from(value)
    };
    home.is_absolute().then_some(home)
}

fn profiles(user: &Path, inherited_home: Option<PathBuf>, settings: &Value) -> Vec<Profile> {
    let instances = settings.get("providerInstances").and_then(Value::as_object);
    let primary_config = instances
        .and_then(|instances| instances.get("codex"))
        .and_then(|instance| instance.get("config"))
        .filter(|config| config["setupMode"].as_str() != Some("managed"));
    let primary_home = primary_config
        .and_then(|config| {
            configured_home(&config["shadowHomePath"], user)
                .or_else(|| configured_home(&config["homePath"], user))
        })
        .or(inherited_home)
        .unwrap_or_else(|| user.join(".codex"));
    let mut secondary_home = user.join(".codex-secondary");
    if home_key(&primary_home) == home_key(&secondary_home) {
        secondary_home = user.join(".codex");
    }
    let mut result = vec![Profile {
        id: "codex".into(),
        home: primary_home,
    }];
    let mut add = |profile: Profile| {
        let home = home_key(&profile.home);
        if !result
            .iter()
            .any(|existing| home_key(&existing.home) == home)
        {
            result.push(profile);
        }
    };
    add(Profile {
        id: "codex-secondary".into(),
        home: secondary_home,
    });
    if let Some(instances) = instances {
        for (id, instance) in instances {
            let config = &instance["config"];
            if id == "codex"
                || id == "codex-secondary"
                || instance["driver"].as_str() != Some("codex")
                || instance["enabled"].as_bool() == Some(false)
                || config["enabled"].as_bool() == Some(false)
                || config["setupMode"].as_str() == Some("managed")
            {
                continue;
            }
            // shortcut: managed T3 tokens are external, add support when T3 exposes a usage API.
            let home = configured_home(&config["shadowHomePath"], user)
                .or_else(|| configured_home(&config["homePath"], user))
                .unwrap_or_else(|| user.join(".codex"));
            add(Profile {
                id: id.clone(),
                home,
            });
        }
    }
    result
}

fn home_key(home: &Path) -> String {
    home.canonicalize()
        .unwrap_or_else(|_| home.to_path_buf())
        .to_string_lossy()
        .replace('/', "\\")
        .trim_end_matches('\\')
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn discovers_isolated_cli_homes_without_duplicating_or_importing_managed_accounts() {
        let user = Path::new("C:\\Users\\owner");
        let settings = json!({"providerInstances": {
            "codex": {"driver":"codex", "config":{"homePath":""}},
            "codex_work": {"driver":"codex", "config":{"homePath":"~/.codex", "shadowHomePath":"~/.codex-work"}},
            "codex_same": {"driver":"codex", "config":{"shadowHomePath":"C:/Users/OWNER/.codex-secondary/"}},
            "codex_managed": {"driver":"codex", "config":{"setupMode":"managed"}},
            "codex_disabled": {"driver":"codex", "enabled":false, "config":{"shadowHomePath":"~/.disabled"}}
        }});
        let found = profiles(user, Some(user.join(".codex-env")), &settings);
        assert_eq!(found.len(), 3);
        assert_eq!(found[0].home, user.join(".codex-env"));
        assert!(found
            .iter()
            .any(|profile| profile.home == user.join(".codex-work")));
        assert!(!found.iter().any(|profile| profile.id == "codex_managed"));
        assert!(configured_home(&json!("../relative"), user).is_none());

        let found = profiles(
            user,
            Some(user.join(".codex-env")),
            &json!({"providerInstances": {
                "codex": {"driver":"codex", "config":{"setupMode":"managed", "shadowHomePath":"~/.managed"}}
            }}),
        );
        assert_eq!(found[0].home, user.join(".codex-env"));
        let found = profiles(
            user,
            Some(user.join(".codex-env")),
            &json!({"providerInstances": {
                "codex": {"driver":"codex", "config":{"homePath":"~/.shared", "shadowHomePath":"~/.custom"}}
            }}),
        );
        assert_eq!(found[0].home, user.join(".custom"));
        let found = profiles(user, Some(user.join(".codex-secondary")), &Value::Null);
        assert_eq!(found.len(), 2);
        assert_ne!(home_key(&found[0].home), home_key(&found[1].home));
    }
}
