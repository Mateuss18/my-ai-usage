# Issue #8: validate Codex runtime and failure scenarios on Windows

## Confirmed scope

The user explicitly confirmed the following scope with `s` after two clarification questions. Validate the actual Windows application against issue #8: https://github.com/Mateuss18/my-ai-usage/issues/8. Reproduce authenticated Codex, logged-out account, Codex absent from the app's PATH, unavailable network/timeout, and a partial response after a valid snapshot. Observe distinct actionable messages without user-facing stack traces, unknown rather than invented zero usage, stale identification of retained data, sanitized logs, and cleanup of every app-owned `codex app-server` process.

Only fix minimal defects that prevent these criteria, with focused executable checks. Record actions, environment, results and traceable evidence in `docs/TECHNICAL_NOTES.md`; update `docs/ROADMAP.md` only for scenarios actually reproduced. Synthetic evidence complements but never replaces manual Windows evidence.

Session authentication and connectivity may be changed temporarily, reversibly and in coordination with the user, and must be restored and documented. Use official authentication commands only; never read, copy or manipulate credential files. Preserve preexisting work. Excluded: MSIX/clean installation validation, performance, full accessibility audit, Explorer behavior, new features, visual redesign, telemetry, new architecture, and credential access outside official commands.

This specification phase may write only this file. It does not authorize this agent to implement, change product documentation, alter authentication/connectivity, or perform Git mutations, branch creation/deletion, commit, push or MR operations.

## Loop state

- Phase: implementation complete; manual matrix partially complete.
- User scope confirmation: explicit `s`; questions used: 2/6.
- Round: 4/4. Implementation: REVISE after Round 3 review. Independent review: REVISE.
- Blocker and next action: final review 4 returned REVISE with unresolved evidence/state findings; the matching packaged artifact is also unavailable, so manual S1–S6 remain blocked. Stop the eloop without delivery or MR; resume only in a new authorized run after resolving the findings and providing the matching runnable artifact.
- Confirmed implementation base: `main`; PR #16 merged at `532b92736ef082c0fdc8ec8e994d2161feed05e7` (handoff fact; recheck before implementation).
- Spec inspection: current branch `feature/issue-13-tray-usage-panel`; `git status --short` was empty before this spec. Do not delete that branch or treat it as the confirmed implementation base.
- Issue read live during spec: OPEN; title and body agree with confirmed scope. Existing format reused from `2026-09-07-issue-13-tray-usage-panel.md`; its completed checks and delivery are not evidence for issue #8.
- Next action: obtain an authorized runnable artifact with package identity, then repeat the unchanged matrix on that final source snapshot. No S1-S5 application scenario passed in Round 1.
- Scope gaps: none identified. Availability of a real partial response and a runnable artifact matching the tested source are execution prerequisites, not assumed successes. If unavailable, record the precise blocker and required next action without substituting simulation or expanding scope.

## Acceptance criteria

- [ ] A1 — All five scenario families below have dated local Windows actions, environment, actual result and evidence references in `docs/TECHNICAL_NOTES.md`. Unexecuted, blocked, failed and simulated cases are labeled accurately; none count as a reproduced pass.
- [ ] A2 — Authenticated: the real app reads the official runtime successfully and displays supplied quota windows, values and last-success time. Record a valid complete snapshot as the baseline for subsequent failure/partial transitions, without account identity or full protocol payloads.
- [ ] A3 — Logged out: after official logout and a fresh read, the app visibly identifies the disconnected account and gives an actionable login instruction. It is distinguishable from missing executable and network/timeout failure; no stack trace is shown.
- [ ] A4 — Missing PATH: in the actual app process environment, Codex cannot be resolved. The panel identifies the missing executable with installation/PATH guidance, without a stack trace. Do not uninstall Codex or permanently edit user/system PATH to stage this case.
- [ ] A5 — Network/timeout: exercise the actual app with a verified unavailable network or observed timeout, recording which condition occurred and the elapsed read outcome. The message is actionable and distinguishable from logout/missing Codex; identify timeout separately when reproduced. Do not mark both network and timeout verified from one ambiguous failure. Restore connectivity and demonstrate a successful fresh read.
- [ ] A6 — Partial after valid snapshot: record a complete snapshot, then an actual partial response and the rendered transition. The prior valid snapshot remains identifiable as stale, with its original success time; missing fields are not fabricated, and the partial read does not silently become a complete fresh success. Synthetic partial input alone cannot satisfy this criterion.
- [ ] A7 — For failure and partial reads with a prior valid snapshot, retained values are explicitly stale and keep their last-success time. With no valid snapshot, unavailable data is visibly unknown, never `0%`; an actual supplied zero is allowed. Exercise the no-snapshot state separately from the retained-snapshot state and document the distinction.
- [ ] A8 — Inspect the actual app diagnostic outputs for every scenario. No credentials, tokens, keys, e-mail or full account/protocol payloads appear; no stack trace reaches the user. Record inspected sinks, time boundaries and safe findings. If no logs are emitted, document that observation and the inspected output paths; source inspection alone is insufficient runtime evidence.
- [ ] A9 — Identify the app-owned runtime tree by PID, parent PID and creation time before/during/after each case. On actual app exit (`Sair`), including exit during an outstanding failed/timed-out read, all owned app-server processes terminate. A failed start leaves none. Do not kill unrelated Codex/editor/session processes; force cleanup, if needed, is a failed observation, not proof of correct cleanup.
- [ ] A10 — Login state, connectivity and any temporary environment change are restored to their recorded initial state and verified at the end, including after failure/interruption. Unfinished restoration is a blocker and is explicitly documented without secrets.
- [ ] A11 — Any correction is limited to a demonstrated acceptance failure, uses existing paths/patterns and has a focused regression check plus a repeated affected manual scenario on the final artifact. Roadmap changes claim only reproduced results and do not imply release/MSIX or other excluded validation.

## Verification

All runtime/build/check results below are **PENDING**. Run shell commands with `rtk`. Resolve actual launch and log locations from the checked-out application and local environment; never invent an executable path or assume an older installed package matches the source.

Evidence root to fill at execution: `<local-evidence-root>/issue-8/<timestamp>/`. Record the resolved absolute path in the technical notes. Suggested artifact names: `environment.md`, `snapshot.md`, `S1-authenticated.md`, `S2-logged-out.md`, `S3-missing-path.md`, `S4-network-timeout.md`, `S5-partial.md`, `logs-inspection.md`, `process-lifecycle.md`, `restoration.md`, and sanitized UI captures. These are future evidence locations, not files created by this spec phase. Never retain raw secret-bearing output or full runtime payloads in evidence; record sanitized findings and only necessary quota/state metadata.

Snapshot record: base SHA, tested HEAD, scoped diff/file hashes if dirty, artifact absolute path and SHA-256, build configuration, Windows version/build, .NET SDK, Codex version/resolved executable, app launch method, local timestamp/timezone. Capture a new snapshot after fixes and associate each observation with the artifact actually run. Preserve unrelated work and do not attribute it to this task.

| Check | Command/action | Actual result / exit code | Evidence / snapshot |
| --- | --- | --- | --- |
| Source baseline | `rtk git status --short`; `rtk git branch --show-current`; `rtk git rev-parse HEAD`; `rtk git rev-parse main`; inspect relationship to confirmed merge SHA without changing branches in spec phase | Reproduced; HEAD/main `532b92736ef082c0fdc8ec8e994d2161feed05e7`; only task-local spec was initially untracked | artifacts/issue-8/2026-09-07-round-1/environment.md |
| Tool environment | `rtk dotnet --info`; `rtk proxy powershell.exe -NoProfile -Command "Get-Command codex | Select-Object Source; [Environment]::OSVersion.Version"`; `rtk proxy codex --version` | Reproduced; Windows `10.0.26200.0`, SDK `8.0.424`, Codex `0.153.4`; exit 0 | artifacts/issue-8/2026-09-07-round-1/environment.md |
| Build if needed for tested source | `rtk dotnet restore MyAiUsage.sln`; `rtk dotnet build MyAiUsage.sln -c Release -p:Platform=x64 -warnaserror` | Reproduced; both exit 0, 4 projects, no errors/warnings | artifacts/issue-8/2026-09-07-round-1/environment.md / snapshot.md; not manual proof |
| Existing focused checks | `rtk dotnet run --project checks/MyAiUsage.Core.Checks/MyAiUsage.Core.Checks.csproj -c Release -p:Platform=x64` | Reproduced; exit 0, `Tray callback check passed.` and `Core checks passed.`; synthetic coverage only | artifacts/issue-8/2026-09-07-round-1/environment.md |
| Auth coordination | Inspect installed official command help via `rtk proxy codex login --help` and `rtk proxy codex logout --help`; coordinate logout/login window and restoration before changing session state | Help and status inspected; official status remained logged in; no auth mutation performed | artifacts/issue-8/2026-09-07-round-1/environment.md / restoration.md |
| Process ownership | `rtk proxy powershell.exe -NoProfile -Command "Get-CimInstance Win32_Process | Select-Object Name,ProcessId,ParentProcessId,CreationDate"`; correlate app-owned descendants at each transition and after exit | Partial; final artifact PID tracked and no child remained, but app exited before runtime | artifacts/issue-8/2026-09-07-round-1/process-lifecycle.md; no full command lines retained |
| Artifact identity | `rtk proxy powershell.exe -NoProfile -Command "Get-FileHash -Algorithm SHA256 -LiteralPath '<actual-app-artifact>'"` after substituting verified path | Reproduced; final artifact SHA-256 `718EF48A1E1AA394C8F20E9EE4E1FDA7A702939E4D85AA657BB1D80FD8FC2E63` | artifacts/issue-8/2026-09-07-round-1/snapshot.md |
| Scope check | `rtk git diff --check`; `rtk git status --short`; inspect scoped diff and newly created evidence/docs | Reproduced for whitespace and ownership review; no Git mutation; roadmap unchanged | artifacts/issue-8/2026-09-07-round-1/snapshot.md |

Manual matrix (execute in the real Windows UI, with harmless sanitized captures or timestamped observation records):

| ID | Procedure and observable result | Actual result | Evidence |
| --- | --- | --- | --- |
| S1 | Start authenticated app; read complete quota and success time; exercise existing refresh; record owned process tree and valid snapshot | BLOCKED/UNEXECUTED; final WinUI artifact crashed before UI | artifacts/issue-8/2026-09-07-round-1/matrix.md |
| S2 | Coordinate official logout; perform fresh app read, then repeat from no valid snapshot; observe login guidance, stale/unknown semantics; restore via official login | BLOCKED/UNEXECUTED; final WinUI artifact unavailable | artifacts/issue-8/2026-09-07-round-1/matrix.md / restoration.md |
| S3 | Launch actual app in temporary Codex-free process PATH; verify effective absence, not merely a changed terminal PATH; observe missing-runtime guidance and unknown data; restore launch environment | BLOCKED/UNEXECUTED; no final app process reached runtime | artifacts/issue-8/2026-09-07-round-1/matrix.md |
| S4 | Record valid snapshot; coordinate reversible network interruption or timeout reproduction; refresh and observe stale time/message; repeat without prior snapshot; restore and verify successful read | BLOCKED/UNEXECUTED; no final app read could run | artifacts/issue-8/2026-09-07-round-1/matrix.md / restoration.md |
| S5 | Record complete snapshot then real partial response; observe retained stale snapshot and unknown fields; also check partial with no valid snapshot. Record exactly how partial input occurred without storing full payload. If unavailable, leave pending/blocked; label any supplemental simulation separately | BLOCKED/UNEXECUTED; no real final-app partial response | artifacts/issue-8/2026-09-07-round-1/matrix.md |
| S6 | Inspect actual diagnostics across S1–S5; record sinks and safe conclusions; exit normally and during an outstanding read, then verify every owned runtime PID is gone | PARTIAL/BLOCKED; launch WER inspected, runtime exit cases unavailable | artifacts/issue-8/2026-09-07-round-1/logs-inspection.md / process-lifecycle.md |
| S7 | Verify original login/connectivity/environment restored; map each roadmap edit to reproduced evidence; repeat affected scenarios after remediation | PASS for unchanged state; no temporary session mutation | artifacts/issue-8/2026-09-07-round-1/restoration.md |

Use existing focused checks for parser, retained snapshot, unknown values and process cleanup where relevant. Extend only for a demonstrated defect. A local test double or separate temporary test artifact must be labeled simulated and cannot close S5 or another manual gate. If staging a case requires new production switches, credential access, installation work or another scope expansion, report the blocker rather than implement that mechanism.

## Remediation

### Round 1

- Implementation: executed; no product code changed. The final source artifact was built but could not enter the WinUI runtime without package identity.
- Findings: A1–A9 manual gates remain incomplete because S1–S5 and app lifecycle UI could not execute. The demonstrated launch blocker is recorded in `artifacts/issue-8/2026-09-07-round-1/snapshot.md` and `logs-inspection.md`.
- Minimal correction: none. Installing or staging a fresh MSIX would be an excluded installation step; no alternate existing artifact matched the tested source hashes.
- Reverification: restore/build/checks exit 0; direct final artifact launch exits `-532462766` (`0xE0434352`) with Windows App SDK `REGDB_E_CLASSNOTREG`. No affected manual scenario could be repeated.
- Independent review: REVISE; findings recorded below and in the reviewer handoff. Round 1 remains blocked on the missing matching package identity.
- Remediation items for Round 2: preserve sanitized command outputs with timestamps and exit codes; correct the `MyAiUsage.App.dll` SHA-256 in `snapshot.md`; update this loop state and record the Round 1 verdict without erasing history.
- Round 2 implementation: no remediation applied; checks were rerun, but the evidence files and spec history remained unchanged. The matching packaged artifact remains unavailable.
- Round 2 independent review: REVISE; the three Round 1 findings remain unresolved.
- Round 3 implementation: no remediation applied; checks and hash were rerun, but capture failed before producing auditable output and no documents were edited. The matching packaged artifact remains unavailable.
- Round 3 independent review: REVISE; the three documentary findings remain unresolved.
- Round 4: final permitted implementation/review round. Preserve all prior history; if the final review is not PASS, stop without delivery.
- Round 4 implementation: no remediation applied; checks/hash were rerun but no auditable capture or document update was persisted. No final post-interruption ownership check was run by the implementer.
- Round 4 independent review: REVISE; final unresolved findings are recorded below. Delivery is blocked by the four-round limit and must not proceed.

### Round 1 review findings

- [ ] [CRITICAL] Preserve auditable sanitized outputs for restore, build and Core Checks, including command, timestamp and exit code - `artifacts/issue-8/2026-09-07-round-1/environment.md:17`.
- [ ] [NON_CRITICAL] Correct the truncated/incorrect `MyAiUsage.App.dll` SHA-256 to `3A3005E5362E9ACB4ABEA615E0BFDF7D920EF80670E44E7BD2252B49ED7BD2B1` - `artifacts/issue-8/2026-09-07-round-1/snapshot.md:8`.
- [ ] [NON_CRITICAL] Record the review phase, Round 1 REVISE verdict and remediation state in this spec - `docs/superpowers/plans/2026-09-07-issue-8-codex-runtime-windows.md:15`.

### Round 2 review findings

- [ ] [CRITICAL] Preserve complete sanitized restore, build and Core Checks outputs with command, timestamp and exit code; summaries alone are not auditable - `artifacts/issue-8/2026-09-07-round-1/environment.md:17`.
- [ ] [NON_CRITICAL] Replace the truncated DLL hash with the complete verified SHA-256 `3A3005E5362E9ACB4ABEA615E0BFDF7D920EF80670E44E7BD2252B49ED7BD2B1` - `artifacts/issue-8/2026-09-07-round-1/snapshot.md:8`.
- [ ] [NON_CRITICAL] Record Round 2 implementation/review state and verdict in this spec as a separate history entry - `docs/superpowers/plans/2026-09-07-issue-8-codex-runtime-windows.md:15`.

### Round 3 review findings

- [ ] [CRITICAL] Execute restore, build and Core Checks with a functional capture and preserve complete sanitized outputs, timestamps and exit codes - `artifacts/issue-8/2026-09-07-round-1/environment.md:17`.
- [ ] [NON_CRITICAL] Replace the truncated DLL hash with the complete verified SHA-256 `3A3005E5362E9ACB4ABEA615E0BFDF7D920EF80670E44E7BD2252B49ED7BD2B1` - `artifacts/issue-8/2026-09-07-round-1/snapshot.md:8`.
- [ ] [NON_CRITICAL] Record Round 3 implementation/review history, update phase/verdict, and remove obsolete round-stop instructions - `docs/superpowers/plans/2026-09-07-issue-8-codex-runtime-windows.md:15`.

### Round 4 review findings

- [ ] [CRITICAL] Preserve complete sanitized restore, build and Core Checks outputs with command, timestamp and exit code - `artifacts/issue-8/2026-09-07-round-1/environment.md:13`.
- [ ] [NON_CRITICAL] Replace the truncated DLL hash with the complete verified SHA-256 `3A3005E5362E9ACB4ABEA615E0BFDF7D920EF80670E44E7BD2252B49ED7BD2B1` - `artifacts/issue-8/2026-09-07-round-1/snapshot.md:8`.
- [ ] [NON_CRITICAL] Record Round 4 implementation/review history, final phase/verdict and blocked delivery state - `docs/superpowers/plans/2026-09-07-issue-8-codex-runtime-windows.md:15`.

## Delivery

- Spec delivery: this file only. No implementation, runtime validation, product documentation update or Git mutation performed by the spec agent.
- Implementation delivery: Round 4 final review returned REVISE; no MR or delivery was authorized. Tested source/artifact, blocked scenarios, evidence paths, restoration outcome and checks are recorded in `docs/TECHNICAL_NOTES.md` and `artifacts/issue-8/2026-09-07-round-1`.
- Technical notes updated; roadmap intentionally unchanged because no application scenario was reproduced.
- Manual gaps and unresolved evidence findings prevent claiming issue #8 complete even though automated checks pass. Delivery status: blocked; MR URL: none. The matching packaged artifact and a new authorized loop are required before continuing. This spec creates no additional commit/push/MR/merge authority.

### Authorized continuation after Round 4

The user explicitly requested finalization in a new run. A matching signed package was produced and installed, removing the package-identity blocker. S1 authenticated and S3 missing PATH passed in the real packaged UI; environment restoration passed. Minimal actionable-message and stale-partial fixes were implemented with focused checks. S2 logout, S4 real network/timeout, S5 real partial response and normal tray-exit proof remain honestly pending; see the updated matrix and technical notes. Delivery is no longer blocked by package identity, but issue #8 must not be closed as fully validated until those manual cases are coordinated.
