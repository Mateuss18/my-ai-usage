# Codex Authentication Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add official Codex login, logout, cancellation, and account-switch refresh to the Windows Tauri app.

**Architecture:** The existing single `CodexProvider` owns the app-server child and pending login ID. Tauri exposes short start/poll/cancel calls while Vue owns the visible polling state and pauses the existing usage interval.

**Tech Stack:** Rust, Tauri 2, Vue 3 `<script setup>`, TypeScript, Vitest.

**Spec:** `docs/superpowers/specs/2026-09-08-codex-auth-design.md`

## Global Constraints

- Windows-only MVP; use the current `codex app-server` protocol and no new dependency.
- Start only `chatgpt` login; never store, display, or log credentials, cookies, tokens, or auth URLs.
- Do not implement #35 cache/history, other providers, multiple sessions, or an embedded browser.

---

### Task 1: Provider authentication lifecycle

**Files:**
- Modify: `rewrite/src-tauri/src/codex_provider.rs`
- Modify: `rewrite/src-tauri/src/lib.rs`
- Test: `rewrite/src-tauri/src/codex_provider.rs`

**Interfaces:**
- Produces: `CodexProvider::start_login`, `poll_login`, and `cancel_login`; Tauri commands `start_codex_login`, `poll_codex_login`, and `cancel_codex_login`.
- Consumes: the existing app-server request transport and `UsageSnapshot` read path.

- [ ] **Step 1: Write failing provider tests**

```rust
#[test]
fn completed_login_only_matches_the_pending_login_id() {
    let message = json!({
        "method": "account/login/completed",
        "params": { "loginId": "new-account", "success": true }
    });
    assert_eq!(login_completion("new-account", &message), Some(LoginStatus::Completed));
    assert_eq!(login_completion("other-account", &message), None);
}
```

- [ ] **Step 2: Run the focused Rust test and verify it fails because `login_completion` is missing.**

Run: `cargo.exe test --manifest-path src-tauri/Cargo.toml --locked -- --nocapture`

- [ ] **Step 3: Add the minimum protocol parsing and pending-login lifecycle.**

```rust
let response = self.request("account/login/start", Some(json!({ "type": "chatgpt" })))?;
let login_id = response.get("loginId").and_then(Value::as_str).ok_or(ProviderError::Protocol)?;
let auth_url = response.get("authUrl").and_then(Value::as_str).ok_or(ProviderError::Protocol)?;
```

- [ ] **Step 4: Add Tauri start/poll/cancel commands and open only the validated URL with `explorer.exe`.**

```rust
Command::new("explorer.exe").arg(auth_url).spawn().map_err(|_| "Could not open the browser.")?;
```

- [ ] **Step 5: Run the Rust test suite and verify it passes.**

Run: `npm.cmd run test:rust`

### Task 2: Typed frontend login flow

**Files:**
- Modify: `rewrite/src/bridge.ts`
- Create: `rewrite/src/codexLogin.ts`
- Create: `rewrite/src/codexLogin.test.ts`

**Interfaces:**
- Produces: `createCodexLogin(bridge, options)` with `state`, `begin`, and `cancel`.
- Consumes: Tauri commands `start_codex_login`, `poll_codex_login`, and `cancel_codex_login`.

- [ ] **Step 1: Write failing composable tests.**

```ts
it('returns completion after the native login reports complete', async () => {
  const login = createCodexLogin({ start: vi.fn(), poll: vi.fn().mockResolvedValue('completed'), cancel: vi.fn() }, { wait: async () => {} })
  await expect(login.begin()).resolves.toBe(true)
  expect(login.state.value).toBe('idle')
})
```

- [ ] **Step 2: Run the focused Vitest file and verify it fails because the composable is missing.**

Run: `npm.cmd run test:frontend -- src/codexLogin.test.ts`

- [ ] **Step 3: Add the minimal typed bridge and composable.**

```ts
while (state.value === 'authenticating') {
  if (await bridge.poll() === 'completed') return true
  await wait()
}
return false
```

- [ ] **Step 4: Run the frontend suite and verify it passes.**

Run: `npm.cmd run test:frontend`

### Task 3: Usage-panel action wiring

**Files:**
- Modify: `rewrite/src/components/usage/ProviderHeader.vue`
- Modify: `rewrite/src/components/usage/UsagePanel.vue`
- Modify: `rewrite/src/App.vue`
- Test: `rewrite/src/components/usage/UsagePanel.test.ts`

**Interfaces:**
- Consumes: `createCodexLogin` and the existing `createUsageRefresh` controls.
- Produces: `switch-account` and `cancel-login` component events.

- [ ] **Step 1: Write a failing panel test for forwarding `switch-account`.**

```ts
await wrapper.get('button[aria-label="Switch Codex account"]').trigger('click')
expect(wrapper.emitted('switch-account')).toHaveLength(1)
```

- [ ] **Step 2: Run the focused test and verify it fails because the action is absent.**

Run: `npm.cmd run test:frontend -- src/components/usage/UsagePanel.test.ts`

- [ ] **Step 3: Add the explicit action and compose the two flows in `App.vue`.**

```ts
stop()
try {
  await login.begin()
} finally {
  start()
}
```

- [ ] **Step 4: Run all checks.**

Run: `npm.cmd test && npm.cmd run lint && npm.cmd run build && git diff --check`
