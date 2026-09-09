# Codex authentication design

## Goal

Let the Windows Tauri app sign in, sign out, or switch the active Codex ChatGPT account through the official `codex app-server` flow, then refresh the displayed usage.

## Boundaries

- Use only `account/logout`, `account/login/start`, `account/login/cancel`, `account/login/completed`, `account/read`, and `account/rateLimits/read`.
- Start the supported ChatGPT flow with `{ "type": "chatgpt" }`; open only the returned `authUrl` in the Windows default browser.
- Keep credentials, cookies, tokens, full authentication responses, and URLs out of app state, storage, and logs.
- Keep account history/cache and multi-account presentation out of scope; that belongs to #35.

## Design

`CodexProvider` keeps a pending login ID and buffers app-server notifications that arrive while a request is waiting for its response. `start_login` logs out, starts ChatGPT login, validates the returned HTTPS URL and ID, and retains only the ID. `poll_login` drains notifications without blocking: only a completion with the exact pending ID changes the login; terminal protocol errors clear state and shut down the child. Completion, cancellation, and standalone logout all shut down the child so the next `account/read` starts a fresh app-server for the current account.

The Tauri layer exposes start, poll, cancel, and logout commands. Starting opens the returned URL with `explorer.exe`, which delegates to the registered Windows browser without a shell command. The Vue composable distinguishes starting, browser waiting, cancellation, idle, and recoverable failure. `App.vue` invalidates an in-flight usage refresh before auth changes, disables account actions during logout, resumes afterward, and immediately reads `account/read` and rate limits through the fresh provider.

The `account/read` response contributes only the active ChatGPT email to the in-memory usage snapshot, so the header can confirm the current account. No credentials, tokens, cookies, URLs, or account history are retained.

## Component map

- `src/codexLogin.ts`: owns login polling state and invokes the typed Tauri bridge.
- `src/components/usage/ProviderHeader.vue`: presents sign-in, switch, cancel, and sign-out actions plus browser-waiting/login-failure/logout-failure feedback.
- `src/components/usage/UsagePanel.vue`: forwards those events; it does not own authentication state.
- `src/App.vue`: composes usage refresh and login flows, keeping the root as wiring only.

## Verification

- Rust unit tests prove ChatGPT login response parsing, active-identity extraction, exact completion matching, logout normalization, and recovery after terminal app-server failure without starting Codex or changing a real account.
- Vitest proves the frontend polling loop, explicit UI states, disabled logout actions, and stale in-flight refresh invalidation.
- The full frontend/Rust test suite, lint, build, diff check, and a read-only verification of the generated app-server schema are run before the MR.
