# Codex authentication design

## Goal

Let the Windows Tauri app switch the active Codex ChatGPT account through the official `codex app-server` flow, then refresh the displayed usage.

## Boundaries

- Use only `account/logout`, `account/login/start`, `account/login/cancel`, `account/login/completed`, `account/read`, and `account/rateLimits/read`.
- Start the supported ChatGPT flow with `{ "type": "chatgpt" }`; open only the returned `authUrl` in the Windows default browser.
- Keep credentials, cookies, tokens, full authentication responses, and URLs out of app state, storage, and logs.
- Keep account history/cache and multi-account presentation out of scope; that belongs to #35.

## Design

`CodexProvider` keeps a pending login ID and buffers app-server notifications that arrive while a request is waiting for its response. `start_login` logs out, starts ChatGPT login, validates the returned `authUrl` and ID, and retains only the ID. `poll_login` drains notifications without blocking: a matching successful completion clears the pending ID; a matching failure returns a recoverable error; unmatched notifications are ignored. `cancel_login` sends the protocol cancellation request and clears local pending state.

The Tauri layer exposes start, poll, and cancel commands. Starting opens the returned URL with `explorer.exe`, which delegates to the registered Windows browser without a shell command. The Vue composable runs the polling loop and exposes only `idle`, `authenticating`, and `error` state. `App.vue` stops periodic refresh before login, resumes it after cancellation/failure, and refreshes once immediately after successful completion.

## Component map

- `src/codexLogin.ts`: owns login polling state and invokes the typed Tauri bridge.
- `src/components/usage/ProviderHeader.vue`: presents the explicit switch/cancel action and emits events upward.
- `src/components/usage/UsagePanel.vue`: forwards those events; it does not own authentication state.
- `src/App.vue`: composes usage refresh and login flows, keeping the root as wiring only.

## Verification

- Rust unit tests prove ChatGPT login response parsing and matching completion/cancellation behavior without starting Codex or changing a real account.
- Vitest proves the frontend polling loop returns success, error, and cancellation to its caller.
- The full frontend/Rust test suite, lint, build, diff check, and a read-only verification of the generated app-server schema are run before the MR.
