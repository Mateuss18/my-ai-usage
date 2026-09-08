# Usage contract

`ProviderUsage` is the provider-neutral DTO exchanged by Tauri and Vue. Each provider adapter maps its native response into this shape; UI components consume only the contract fields.

- `percentage`, `used`, `limit`, and `resetAt` are nullable. `null` means the provider did not expose the value; adapters must not estimate it.
- `state` is one of `loading`, `available`, `partial`, `stale`, `unauthenticated`, `not-installed`, or `error`.
- Provider IDs are stable: `codex`, `opencode`, and `claude`. Add a new ID and adapter without adding provider-specific fields to components.
- Keep `schemaVersion: 1` until a migration is defined. Error messages must not contain tokens or token-bearing URLs.
