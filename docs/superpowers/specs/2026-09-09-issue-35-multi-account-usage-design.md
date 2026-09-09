# Multiple Codex Accounts with Cached Usage Snapshots

## Context

The Tauri rewrite currently discards the `account/read` result and exposes one
Codex provider usage value. Issue #35 requires retaining the last known usage
for each identifiable Codex account while refreshing only the account returned
by the current app-server session.

## Design

Rust remains responsible for account discovery, snapshot merging, and local
persistence. `account/read` runs before `account/rateLimits/read`; ChatGPT
accounts use a stable `codex:<lowercase-trimmed-email>` key. Accounts without a
stable identifier are reported explicitly as an identity error; no synthetic
identifier is created. The known protocol limitation that two workspaces may
share one email is documented.

The provider-neutral DTO becomes:

```text
UsageSnapshot(schemaVersion: 2)
├── accounts: AccountUsageSnapshot[]
├── activeAccountKey: string | null
├── fetchedAt: string | null
└── error: UsageError | null

AccountUsageSnapshot
├── account: AccountIdentity { key, provider, email, accountType, plan }
├── usage: ProviderUsage
└── fetchedAt: string | null
```

`UsageRepository` stores only this sanitized DTO in
`app_data_dir/usage-snapshots.json`. Missing or malformed files load as empty;
writes use a temporary file and replacement. Tokens, cookies, auth payloads,
and raw server responses never enter the DTO or logs.

On a successful refresh, the matching account is upserted, its state is
updated, and previous accounts with usable data become `stale`. On a failure,
the last valid quotas remain and the affected/current account is marked stale
with the controlled provider error; an account with no prior usage remains an
error entry. If account discovery itself fails, the previous cache remains
visible and the root error identifies the active refresh failure.

The frontend receives the list unchanged, maps each record to a compact
account card, labels the current account `Active` and others `Cached`, and
shows each record's last-update time. It keeps one serialized timer and does
not perform account-specific refreshes. Loading/error UI is global only when
there are no cached account records; one account's partial/error state does not
hide the others.

## Files and tests

- Rust: `usage_contract.rs`, new `usage_repository.rs`,
  `codex_provider.rs`, and `lib.rs`.
- Frontend: `domain/usage.ts`, `usageRefresh.ts`, `App.vue`, usage panel/types,
  fixtures, and focused Vitest tests.
- Documentation: `docs/TECHNICAL_NOTES.md` and `docs/ROADMAP.md`.
- Tests cover account parsing/keying, A -> B -> A deduplication, stale/error
  preservation, malformed persistence, restart restoration, active-only
  refresh, and absence of credential fields in serialized data.

## Deliberate non-goals

No simultaneous authenticated sessions, second app-server processes, manual
token handling, account switching controls, database, or new dependency.
