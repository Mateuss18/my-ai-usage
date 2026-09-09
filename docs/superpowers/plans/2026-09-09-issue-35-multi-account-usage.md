# Multiple Codex Accounts with Cached Usage Snapshots Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Identify the current Codex account, retain sanitized usage snapshots for every known account across restarts, and show active versus cached accounts while refreshing only the active account.

**Architecture:** Rust owns account parsing, the in-memory repository, merge rules, and JSON persistence under Tauri's app data directory. The existing single `get_usage` bridge returns a version-2 provider-neutral snapshot containing account records; Vue maps that list to compact account sections and keeps the existing single serialized refresh timer.

**Tech Stack:** Rust 2021, Tauri 2, `serde`/`serde_json`, standard filesystem APIs, Vue 3 `<script setup>`, TypeScript, Vitest, existing npm/Cargo toolchain.

**Spec:** `docs/superpowers/specs/2026-09-09-issue-35-multi-account-usage-design.md`

## Global Constraints

- Use `codex:<trimmed-lowercase-email>` only when the protocol exposes a non-empty ChatGPT email; never invent an identity for a missing email or unsupported account type.
- Persist only the sanitized DTO: account identity, provider usage fields, errors from the controlled error enum, and timestamps; never persist or log tokens, cookies, credentials, auth payloads, or raw JSON-RPC responses.
- Missing or malformed cache files must behave as an empty repository and must not crash the app.
- A successful refresh upserts only the current account and marks previous usable records stale; a failed refresh preserves the last valid quotas.
- Keep one refresh timer/request path and preserve unrelated changes outside this worktree.

### Task 1: Extend the Rust usage contract and add the local repository

**Files:**
- Modify: `src-tauri/src/usage_contract.rs`
- Create: `src-tauri/src/usage_repository.rs`
- Modify: `src-tauri/src/lib.rs` to expose the module
- Test: Rust unit tests in the two files

**Interfaces:**
- `AccountIdentity { key, provider, email, account_type, plan }`
- `AccountUsageSnapshot { account, usage, fetched_at }`
- `UsageSnapshot { schema_version: 2, accounts, active_account_key, fetched_at, error }`
- `UsageRepository::load(path: &Path) -> Self`
- `UsageRepository::record_success(account, usage, fetched_at) -> UsageSnapshot`
- `UsageRepository::record_failure(account, error, fetched_at) -> UsageSnapshot`
- `UsageRepository::save(path: &Path) -> io::Result<()>`

- [ ] Write failing tests for serde camelCase output, A/B upsert without duplicates, stale marking, malformed-file fallback, and a serialized snapshot containing no credential-shaped fields.
- [ ] Run `cargo test --manifest-path src-tauri/Cargo.toml` and confirm the new tests fail because the types and repository do not exist.
- [ ] Add the version-2 DTOs with nullable identity fields and controlled root error, preserving nullable quota values.
- [ ] Implement the repository with a small `Vec<AccountUsageSnapshot>` store, key-based upsert, stale marking, malformed input fallback, parent-directory creation, temporary-file write, and Windows-compatible replacement.
- [ ] Run the focused Rust tests and confirm they pass.

### Task 2: Parse account identity and merge provider refreshes

**Files:**
- Modify: `src-tauri/src/codex_provider.rs`
- Test: `src-tauri/src/codex_provider.rs`

**Interfaces:**
- `parse_account(&Value) -> Result<AccountIdentity, ProviderError>`
- `read_usage(&mut self) -> Result<UsageRead, UsageFailure>` where `UsageRead` contains identity, quotas, and partial status.
- `CodexProvider::usage(&mut self, repository: &mut UsageRepository) -> UsageSnapshot`

- [ ] Write failing tests for ChatGPT email normalization, plan/type extraction, missing email rejection, account-read authentication failure, and preservation of an earlier A snapshot after a B success/failure sequence.
- [ ] Run the focused Rust tests and confirm the new tests fail for the expected missing parser/merge behavior.
- [ ] Preserve the `account/read` result, derive only the documented identity fields, then read rate limits and pass the sanitized result to the repository.
- [ ] Attach the account identity to rate-limit failures when account/read succeeded; otherwise preserve the previous repository cache and expose a root error.
- [ ] Keep existing quota parsing and process cleanup behavior unchanged.
- [ ] Run all Rust tests and confirm they pass.

### Task 3: Connect persistence to the Tauri command

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Test: existing command/provider compile path and Rust suite

**Interfaces:**
- `get_usage(app: AppHandle, provider: State<SharedCodexProvider>) -> UsageSnapshot`
- Cache path: `app.path().app_data_dir().join("usage-snapshots.json")`

- [ ] Update the command to load the repository before the provider call, pass it to `CodexProvider::usage`, and save the resulting sanitized repository after both success and failure paths.
- [ ] Treat an unavailable app-data path or failed cache write as a non-fatal persistence limitation while returning the live provider result.
- [ ] Compile and run the complete Rust test suite.

### Task 4: Evolve the TypeScript domain and refresh orchestration

**Files:**
- Modify: `src/domain/usage.ts`
- Modify: `src/usageRefresh.ts`
- Modify: `src/bridge.ts` only if the typed command signature requires it
- Test: `src/domain/usage.test.ts`, `src/usageRefresh.test.ts`

**Interfaces:**
- `AccountIdentity`, `AccountUsageSnapshot`, and version-2 `UsageSnapshot` matching Rust camelCase JSON.
- `createUsageRefresh(load)` returns `{ accounts, loading, error, refresh, start, stop }`.

- [ ] Write failing Vitest cases for A -> B -> A, active flags, stale preservation, root errors, restart-loaded accounts, and serialized nullable fields.
- [ ] Run the focused Vitest files and confirm the new cases fail against the singular-provider implementation.
- [ ] Replace the singular `provider`/`lastValid` state with an account list mapped by `account.key`; keep request coalescing, visibility handling, and one timer.
- [ ] Map account identity, plan, per-account state, quotas, timestamps, and controlled errors into a panel type without importing JSON-RPC concepts.
- [ ] Run the focused Vitest files and confirm they pass.

### Task 5: Render multiple accounts accessibly in the compact panel

**Files:**
- Modify: `src/App.vue`
- Modify: `src/components/usage/usageTypes.ts`
- Modify: `src/components/usage/UsagePanel.vue`
- Modify: `src/components/usage/UsagePanel.test.ts`
- Modify: `src/components/usage/UsageCard.vue` only if the shared quota markup needs no other change
- Modify: `src-tauri/tauri.conf.json` and `src-tauri/src/lib.rs` only to restore enough fixed panel height for two compact account sections

**Interfaces:**
- `UsagePanel` accepts `accounts`, `loading`, `error`, and emits one `refresh` event.
- Each account section renders email/label, plan when known, explicit `Active` or `Cached` text, last-update text, and its existing quota cards.

- [ ] Add SSR tests for two account sections, explicit active/cached labels, timestamps, per-account error/partial states, and global loading/error with no cache.
- [ ] Run the panel tests and confirm the new assertions fail.
- [ ] Render cached records without hiding them; show global loading/error only when no records exist; keep status distinguishable by text and accessible roles, not color alone.
- [ ] Run the panel tests and confirm they pass.

### Task 6: Document the feature and verify the whole change

**Files:**
- Modify: `docs/TECHNICAL_NOTES.md`
- Modify: `docs/ROADMAP.md`
- No code changes outside the issue scope

- [ ] Document the app-data cache, identity limitation for missing email/shared workspaces, stale semantics, and explicit non-persistence of credentials.
- [ ] Run `npm.cmd test`, `npm.cmd run lint`, `npm.cmd run typecheck`, `npm.cmd run build`, `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check`, and `git diff --check` from the worktree.
- [ ] Inspect the final diff and verify the root `main` worktree remains untouched; manually validate only if the user separately authorizes desktop interaction.
- [ ] Create a focused semantic commit and report any unverified runtime gates instead of inferring them from tests/builds.
