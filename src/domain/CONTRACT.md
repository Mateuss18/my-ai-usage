# Usage contract

`UsageSnapshot` is the provider-neutral DTO exchanged by Tauri and Vue. It is schema version 2 and contains account snapshots; each `ProviderUsage` inside it remains schema version 1. UI components consume only the contract fields.

- `percentage`, `used`, `limit`, and `resetAt` are nullable. `null` means the provider did not expose the value; adapters must not estimate it.
- `state` is one of `loading`, `available`, `partial`, `stale`, `unauthenticated`, `not-installed`, or `error`.
- Provider IDs are stable: `codex`, `opencode`, and `claude`. Add a new ID and adapter without adding provider-specific fields to components.
- Account identity is stored separately from provider usage. The Codex cache key is `codex:<normalized email>`; without an email, adapters must not create a synthetic key.
- Cached accounts retain their last known quotas with `stale` state. Error messages must not contain tokens or token-bearing URLs.
