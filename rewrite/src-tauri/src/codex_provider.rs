use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde_json::{json, Value};

use crate::usage_contract::{
    Provider, ProviderUsage, UsageError, UsageQuota, UsageSnapshot, UsageState,
};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Default)]
pub struct CodexProvider {
    child: Option<Child>,
    input: Option<ChildStdin>,
    responses: Option<Receiver<String>>,
    notifications: VecDeque<Value>,
    next_request_id: u64,
    pending_login_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LoginStatus {
    Pending,
    Completed,
    Failed,
}

struct LoginStart {
    id: String,
    auth_url: String,
}

impl CodexProvider {
    pub fn usage(&mut self) -> UsageSnapshot {
        let fetched_at = now_iso8601();
        if self.pending_login_id.is_some() {
            return UsageSnapshot {
                schema_version: 1,
                providers: vec![provider_usage(
                    UsageState::Unauthenticated,
                    None,
                    Vec::new(),
                    Some(ProviderError::Unauthenticated),
                )],
                fetched_at: Some(fetched_at),
            };
        }

        let result = self.read_usage();

        match result {
            Ok((quotas, partial)) => UsageSnapshot {
                schema_version: 1,
                providers: vec![provider_usage(
                    if partial {
                        UsageState::Partial
                    } else {
                        UsageState::Available
                    },
                    Some(fetched_at),
                    quotas,
                    None,
                )],
                fetched_at: Some(now_iso8601()),
            },
            Err(error) => {
                self.shutdown();
                UsageSnapshot {
                    schema_version: 1,
                    providers: vec![provider_usage(error.state(), None, Vec::new(), Some(error))],
                    fetched_at: Some(fetched_at),
                }
            }
        }
    }

    pub fn shutdown(&mut self) {
        self.pending_login_id = None;
        self.input = None;
        self.responses = None;
        self.notifications.clear();
        if let Some(mut child) = self.child.take() {
            let _ = Command::new("taskkill.exe")
                .args(["/pid", &child.id().to_string(), "/t", "/f"])
                .output();
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    pub fn start_login(&mut self) -> Result<String, ProviderError> {
        if self.pending_login_id.is_some() {
            return Err(ProviderError::Protocol);
        }

        let result = (|| {
            self.start()?;
            normalize_logout_result(self.request("account/logout", None).map(|_| ()))?;
            let login = parse_login_start(
                &self.request("account/login/start", Some(json!({ "type": "chatgpt" })))?,
            )?;
            self.pending_login_id = Some(login.id);
            Ok(login.auth_url)
        })();
        if result.is_err() {
            self.shutdown();
        }
        result
    }

    pub fn logout(&mut self) -> Result<(), ProviderError> {
        let result = self
            .start()
            .and_then(|()| self.request("account/logout", None).map(|_| ()));
        self.shutdown();
        normalize_logout_result(result)
    }

    pub fn poll_login(&mut self) -> Result<LoginStatus, ProviderError> {
        let login_id = self
            .pending_login_id
            .clone()
            .ok_or(ProviderError::Protocol)?;
        while let Some(message) = self.notifications.pop_front() {
            if let Some(status) = login_status(&login_id, &message) {
                self.shutdown();
                return Ok(status);
            }
        }

        loop {
            let line = match self
                .responses
                .as_ref()
                .ok_or(ProviderError::Protocol)?
                .try_recv()
            {
                Ok(line) => line,
                Err(TryRecvError::Empty) => return Ok(LoginStatus::Pending),
                Err(TryRecvError::Disconnected) => {
                    self.shutdown();
                    return Err(ProviderError::EndOfStream);
                }
            };
            let message: Value = match serde_json::from_str(&line) {
                Ok(message) => message,
                Err(_) => {
                    self.shutdown();
                    return Err(ProviderError::InvalidJson);
                }
            };
            if let Some(status) = login_status(&login_id, &message) {
                self.shutdown();
                return Ok(status);
            }
        }
    }

    pub fn cancel_login(&mut self) -> Result<(), ProviderError> {
        let login_id = self
            .pending_login_id
            .take()
            .ok_or(ProviderError::Protocol)?;
        let result = self
            .request("account/login/cancel", Some(json!({ "loginId": login_id })))
            .map(|_| ());
        self.shutdown();
        result
    }

    fn read_usage(&mut self) -> Result<(Vec<UsageQuota>, bool), ProviderError> {
        self.start()?;
        self.request("account/read", Some(json!({ "refreshToken": false })))?;
        let result = self.request("account/rateLimits/read", None)?;
        parse_quotas(&result)
    }

    fn start(&mut self) -> Result<(), ProviderError> {
        if self
            .child
            .as_mut()
            .is_some_and(|child| child.try_wait().ok().flatten().is_none())
        {
            return Ok(());
        }
        self.shutdown();

        if !codex_is_installed() {
            return Err(ProviderError::NotInstalled);
        }

        let mut child = Command::new("cmd.exe")
            .args(["/d", "/c", "codex", "app-server"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| ProviderError::NotInstalled)?;
        let input = child.stdin.take().ok_or(ProviderError::Protocol)?;
        let output = child.stdout.take().ok_or(ProviderError::Protocol)?;
        let (sender, responses) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(output).lines().map_while(Result::ok) {
                if sender.send(line).is_err() {
                    break;
                }
            }
        });

        self.child = Some(child);
        self.input = Some(input);
        self.responses = Some(responses);
        let initialized = self
            .request(
                "initialize",
                Some(json!({ "clientInfo": { "name": "my-ai-usage", "version": "0.1.0" } })),
            )
            .and_then(|_| self.notify("initialized"));
        if initialized.is_err() {
            self.shutdown();
        }
        initialized
    }

    fn request(&mut self, method: &str, params: Option<Value>) -> Result<Value, ProviderError> {
        self.next_request_id += 1;
        let id = self.next_request_id;
        let message = json!({ "id": id, "method": method, "params": params });
        self.write_message(&message)?;
        let deadline = Instant::now() + REQUEST_TIMEOUT;
        let responses = self.responses.as_ref().ok_or(ProviderError::Protocol)?;

        loop {
            let remaining = deadline
                .checked_duration_since(Instant::now())
                .ok_or(ProviderError::Timeout)?;
            let line = responses
                .recv_timeout(remaining)
                .map_err(|error| match error {
                    mpsc::RecvTimeoutError::Timeout => ProviderError::Timeout,
                    mpsc::RecvTimeoutError::Disconnected => ProviderError::EndOfStream,
                })?;
            let response: Value =
                serde_json::from_str(&line).map_err(|_| ProviderError::InvalidJson)?;
            let object = response.as_object().ok_or(ProviderError::Protocol)?;
            if object.get("id").and_then(Value::as_u64) != Some(id) {
                self.notifications.push_back(response);
                continue;
            }
            if let Some(error) = object.get("error") {
                return Err(if is_authentication_error(error) {
                    ProviderError::Unauthenticated
                } else {
                    ProviderError::Protocol
                });
            }
            return object.get("result").cloned().ok_or(ProviderError::Protocol);
        }
    }

    fn notify(&mut self, method: &str) -> Result<(), ProviderError> {
        self.write_message(&json!({ "method": method }))
    }

    fn write_message(&mut self, message: &Value) -> Result<(), ProviderError> {
        let input = self.input.as_mut().ok_or(ProviderError::Protocol)?;
        serde_json::to_writer(&mut *input, message).map_err(|_| ProviderError::Protocol)?;
        input
            .write_all(b"\n")
            .map_err(|_| ProviderError::EndOfStream)?;
        input.flush().map_err(|_| ProviderError::EndOfStream)
    }
}

impl Drop for CodexProvider {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ProviderError {
    NotInstalled,
    Unauthenticated,
    Timeout,
    EndOfStream,
    InvalidJson,
    Protocol,
    Partial,
}

impl ProviderError {
    fn state(self) -> UsageState {
        match self {
            Self::NotInstalled => UsageState::NotInstalled,
            Self::Unauthenticated => UsageState::Unauthenticated,
            _ => UsageState::Error,
        }
    }

    fn details(self) -> UsageError {
        let (code, message) = match self {
            Self::NotInstalled => ("not-installed", "Codex is not installed."),
            Self::Unauthenticated => ("unauthenticated", "Sign in to Codex to read usage."),
            Self::Timeout => ("timeout", "Codex did not respond in time."),
            Self::EndOfStream => (
                "end-of-stream",
                "Codex app-server stopped before responding.",
            ),
            Self::InvalidJson => ("invalid-json", "Codex returned an invalid response."),
            Self::Protocol => ("protocol-error", "Codex returned an unsupported response."),
            Self::Partial => ("partial-data", "Codex did not return usable quota windows."),
        };
        UsageError {
            code: code.into(),
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.details().message)
    }
}

fn provider_usage(
    state: UsageState,
    captured_at: Option<String>,
    quotas: Vec<UsageQuota>,
    error: Option<ProviderError>,
) -> ProviderUsage {
    ProviderUsage {
        schema_version: 1,
        id: Provider::Codex,
        name: "Codex".into(),
        vendor: "OpenAI".into(),
        state,
        captured_at,
        quotas,
        error: error.map(ProviderError::details),
    }
}

fn codex_is_installed() -> bool {
    Command::new("where.exe")
        .arg("codex")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn is_authentication_error(error: &Value) -> bool {
    error
        .get("message")
        .and_then(Value::as_str)
        .is_some_and(|message| {
            let message = message.to_ascii_lowercase();
            message.contains("authentication")
                || message.contains("unauthorized")
                || message.contains("not authenticated")
        })
}

fn normalize_logout_result(result: Result<(), ProviderError>) -> Result<(), ProviderError> {
    match result {
        Ok(()) | Err(ProviderError::Unauthenticated) => Ok(()),
        Err(error) => Err(error),
    }
}

fn parse_login_start(result: &Value) -> Result<LoginStart, ProviderError> {
    if result.get("type").and_then(Value::as_str) != Some("chatgpt") {
        return Err(ProviderError::Protocol);
    }
    let id = result
        .get("loginId")
        .and_then(Value::as_str)
        .filter(|id| !id.is_empty())
        .ok_or(ProviderError::Protocol)?;
    let auth_url = result
        .get("authUrl")
        .and_then(Value::as_str)
        .filter(|url| is_valid_https_url(url))
        .ok_or(ProviderError::Protocol)?;

    Ok(LoginStart {
        id: id.into(),
        auth_url: auth_url.into(),
    })
}

fn is_valid_https_url(url: &str) -> bool {
    url.strip_prefix("https://")
        .and_then(|url| url.split('/').next())
        .is_some_and(|host| {
            !host.is_empty()
                && !host
                    .chars()
                    .any(|character| character.is_whitespace() || character.is_control())
        })
}

fn login_status(login_id: &str, message: &Value) -> Option<LoginStatus> {
    if message.get("method").and_then(Value::as_str) != Some("account/login/completed") {
        return None;
    }
    let params = message.get("params")?;
    if params.get("loginId").and_then(Value::as_str) != Some(login_id) {
        return None;
    }

    match params.get("success").and_then(Value::as_bool) {
        Some(true) => Some(LoginStatus::Completed),
        Some(false) => Some(LoginStatus::Failed),
        None => None,
    }
}

fn parse_quotas(result: &Value) -> Result<(Vec<UsageQuota>, bool), ProviderError> {
    let bucket = result
        .get("rateLimitsByLimitId")
        .and_then(Value::as_object)
        .and_then(|buckets| {
            buckets
                .get("codex")
                .or_else(|| buckets.values().find(|value| value.is_object()))
        })
        .or_else(|| result.get("rateLimits"))
        .and_then(Value::as_object)
        .ok_or(ProviderError::Partial)?;

    let session = bucket
        .get("primary")
        .or_else(|| find_window(bucket, |minutes| minutes == 300));
    let weekly = bucket
        .get("secondary")
        .or_else(|| find_window(bucket, |minutes| minutes >= 7 * 24 * 60));
    let mut partial = session.is_none() || weekly.is_none();
    let mut quotas = Vec::new();

    if let Some(window) = session {
        quotas.push(parse_quota(
            "session",
            "5 hours / session",
            window,
            &mut partial,
        ));
    }
    if let Some(window) = weekly {
        quotas.push(parse_quota(
            "weekly",
            "Weekly / all models",
            window,
            &mut partial,
        ));
    }

    if quotas.is_empty() {
        return Err(ProviderError::Partial);
    }
    Ok((quotas, partial))
}

fn find_window(
    bucket: &serde_json::Map<String, Value>,
    predicate: impl Fn(i64) -> bool,
) -> Option<&Value> {
    bucket.values().find(|window| {
        window
            .get("windowDurationMins")
            .and_then(Value::as_i64)
            .is_some_and(&predicate)
    })
}

fn parse_quota(id: &str, label: &str, window: &Value, partial: &mut bool) -> UsageQuota {
    let percentage = window
        .get("usedPercent")
        .and_then(Value::as_f64)
        .filter(|value| (0.0..=100.0).contains(value));
    let reset_at = window
        .get("resetsAt")
        .and_then(Value::as_i64)
        .and_then(unix_timestamp_to_iso8601);
    if percentage.is_none()
        || reset_at.is_none()
        || window
            .get("windowDurationMins")
            .and_then(Value::as_i64)
            .filter(|value| *value > 0)
            .is_none()
    {
        *partial = true;
    }
    UsageQuota {
        id: id.into(),
        label: label.into(),
        used: None,
        limit: None,
        percentage,
        reset_at,
    }
}

fn now_iso8601() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs() as i64);
    unix_timestamp_to_iso8601(seconds).expect("current Unix timestamp is representable")
}

fn unix_timestamp_to_iso8601(seconds: i64) -> Option<String> {
    let days = seconds.div_euclid(86_400);
    let seconds_of_day = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days)?;
    Some(format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        seconds_of_day / 3_600,
        (seconds_of_day % 3_600) / 60,
        seconds_of_day % 60
    ))
}

fn civil_from_days(days: i64) -> Option<(i64, i64, i64)> {
    let days = days.checked_add(719_468)?;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    Some((year + if month <= 2 { 1 } else { 0 }, month, day))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_primary_and_secondary_windows_to_the_codex_contract() {
        let result = json!({
            "rateLimitsByLimitId": { "codex": {
                "primary": { "usedPercent": 17, "windowDurationMins": 300, "resetsAt": 0 },
                "secondary": { "usedPercent": 42, "windowDurationMins": 10080, "resetsAt": 3600 }
            }}
        });

        let (quotas, partial) = parse_quotas(&result).unwrap();
        assert!(!partial);
        assert_eq!(
            quotas
                .iter()
                .map(|quota| quota.id.as_str())
                .collect::<Vec<_>>(),
            ["session", "weekly"]
        );
        assert_eq!(quotas[0].percentage, Some(17.0));
        assert_eq!(quotas[1].reset_at.as_deref(), Some("1970-01-01T01:00:00Z"));
    }

    #[test]
    fn preserves_known_primary_data_when_the_weekly_window_is_partial() {
        let result = json!({ "rateLimits": {
            "primary": { "usedPercent": 9, "windowDurationMins": 300, "resetsAt": "invalid" }
        }});

        let (quotas, partial) = parse_quotas(&result).unwrap();
        assert!(partial);
        assert_eq!(quotas.len(), 1);
        assert_eq!(quotas[0].percentage, Some(9.0));
        assert_eq!(quotas[0].reset_at, None);
    }

    #[test]
    fn does_not_turn_missing_windows_or_authentication_into_available_usage() {
        assert!(matches!(
            parse_quotas(&json!({ "rateLimits": {} })),
            Err(ProviderError::Partial)
        ));
        assert!(matches!(
            ProviderError::Unauthenticated.state(),
            UsageState::Unauthenticated
        ));
        assert!(matches!(
            ProviderError::NotInstalled.state(),
            UsageState::NotInstalled
        ));
    }

    #[test]
    fn parses_the_official_chatgpt_login_response() {
        let login = parse_login_start(&json!({
            "type": "chatgpt",
            "loginId": "new-account",
            "authUrl": "https://auth.openai.com/authorize"
        }))
        .unwrap();

        assert_eq!(login.id, "new-account");
        assert_eq!(login.auth_url, "https://auth.openai.com/authorize");
        assert!(matches!(
            parse_login_start(&json!({ "type": "apiKey" })),
            Err(ProviderError::Protocol)
        ));
        assert!(matches!(
            parse_login_start(&json!({
                "type": "chatgpt",
                "loginId": "new-account",
                "authUrl": "https://"
            })),
            Err(ProviderError::Protocol)
        ));
    }

    #[test]
    fn only_matching_login_completion_changes_the_pending_login() {
        let completed = json!({
            "method": "account/login/completed",
            "params": { "loginId": "new-account", "success": true }
        });
        let failed = json!({
            "method": "account/login/completed",
            "params": { "loginId": "new-account", "success": false, "error": "Cancelled" }
        });

        assert_eq!(
            login_status("new-account", &completed),
            Some(LoginStatus::Completed)
        );
        assert_eq!(login_status("other-account", &completed), None);
        assert_eq!(
            login_status(
                "new-account",
                &json!({
                    "method": "account/login/completed",
                    "params": { "success": true }
                }),
            ),
            None
        );
        assert_eq!(
            login_status("new-account", &failed),
            Some(LoginStatus::Failed)
        );
    }

    #[test]
    fn treats_an_already_signed_out_codex_session_as_a_successful_logout() {
        assert!(normalize_logout_result(Err(ProviderError::Unauthenticated)).is_ok());
        assert!(matches!(
            normalize_logout_result(Err(ProviderError::Timeout)),
            Err(ProviderError::Timeout)
        ));
    }

    #[test]
    fn makes_login_recoverable_after_the_app_server_stops_or_returns_invalid_json() {
        let (sender, responses) = mpsc::channel();
        drop(sender);
        let mut provider = CodexProvider::default();
        provider.responses = Some(responses);
        provider.pending_login_id = Some("new-account".into());

        assert!(matches!(
            provider.poll_login(),
            Err(ProviderError::EndOfStream)
        ));
        assert_eq!(provider.pending_login_id, None);

        let (sender, responses) = mpsc::channel();
        sender.send("not-json".into()).unwrap();
        let mut provider = CodexProvider::default();
        provider.responses = Some(responses);
        provider.pending_login_id = Some("new-account".into());

        assert!(matches!(
            provider.poll_login(),
            Err(ProviderError::InvalidJson)
        ));
        assert_eq!(provider.pending_login_id, None);
    }
}
