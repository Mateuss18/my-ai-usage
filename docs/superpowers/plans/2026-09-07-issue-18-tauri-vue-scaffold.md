# Issue #18: rewrite: scaffold Tauri 2 + Vue 3 application

## Confirmed scope

The user explicitly confirmed this scope with `s`; clarification questions used: 1/6. Task: https://github.com/Mateuss18/my-ai-usage/issues/18. The supplied confirmed scope is the authority for this specification; the issue was not fetched again during this spec-only retry.

- Objective: establish the minimum runnable rewrite using Tauri 2, Vue 3, TypeScript and Vite, targeting Windows x64.
- Deliverables: minimal Vue frontend and Rust/Tauri host, one simple frontend-to-Tauri command with a visible response, runnable lint/typecheck/test/build scripts, README instructions and appropriate `.gitignore` additions.
- Preserve the C#/WinUI application intact as reference. Do not migrate, move, rename or modify its source, projects, solution, checks, packaging files or `global.json`.
- Excluded: real provider/runtime integrations, complete tray behavior, final UI, WinUI migration, installer generation, signing, publication and release. Existing V1 product requirements do not become rewrite scaffold acceptance criteria.
- Confirmed future Git workflow: create `tauri-version` from `main`; create the implementation branch from `tauri-version`; target the implementation PR at `tauri-version`; do not merge. The current feature branch is not the implementation base.
- This retry authorizes only creation/update of this task specification. No implementation, other file edits, dependency installation, branch creation, commit, push or PR is permitted in this phase. Git workflow below is a handoff requirement, not an action for the specification agent.

## Loop state

- Phase: blocked.
- Clarification questions used: 1/6.
- User scope confirmation: explicit `s`, supplied in the handoff; no reconfirmation required.
- Round: 4/4.
- Implementation verdict: REVISE.
- Blocker and next action: review 4 returned REVISE for a stale contradictory Gaps statement. No round 5 is permitted; correct the statement in a future run before delivery.
- Baseline and evidence: inspected on 2026-09-07 in `C:\Users\mateu\Desktop\Projetos\my-ai-usage`. `git branch --show-current` returned `feature/issue-8-runtime-validation`; `git rev-parse HEAD` returned `01f9d3940763843caf4a2b4d2f46eefce5a3dc70`. Initial `git status --short`, `git diff --stat` and `git diff --cached --stat` were empty. No preexisting dirty paths were observed.
- `git remote get-url origin` confirmed `https://github.com/Mateuss18/my-ai-usage.git`; `git branch --list tauri-version` returned no local branch. Default branch `main`, ADMIN access and absence of remote `tauri-version` are supplied preflight evidence, not reverified in this retry. Recheck relevant remote state before future Git delivery.
- Applicable instruction: user-supplied AGENTS reference to `C:\Users\mateu\.codex\RTK.md`; prefix shell commands with `rtk`. No repository AGENTS.md was reported by preflight.
- Format evidence: existing specs are under `docs/superpowers/plans`; inspected `2026-09-07-issue-8-codex-runtime-windows.md` for structure only. Its verdicts and checks are not evidence for issue #18.
- Technical evidence: repository file listing contains no existing `package.json` or `Cargo.toml`. `src/MyAiUsage.App/MyAiUsage.App.csproj` targets WinUI/.NET 8 and x64 with MSIX; Core targets .NET 8. Existing `.gitignore` covers .NET/local artifacts but not Node/Rust output. README still describes the legacy stack. No Node/Rust/Windows build prerequisites were tested in this phase.

## Acceptance criteria

- [x] A1 — A minimal Tauri 2 host and Vue 3/TypeScript/Vite frontend exist in a clearly isolated rewrite directory, with their manifests, configuration and lockfiles. Reuse the usual scaffold layout inside that directory; avoid colliding with legacy root `src`. The implementer records the chosen directory in README and this spec before verification.
- [x] A2 — On Windows x64, the documented development command opens the Tauri application and renders the minimal Vue page. A labeled, keyboard-accessible action invokes one registered Rust command and displays its deterministic response. Invocation failure is handled visibly without an unhandled rejection. No real integration or simulated quota dashboard is needed.
- [x] A3 — Documented `lint`, `typecheck`, `test` and `build` package scripts execute real checks and terminate with nonzero status on failure. Typechecking covers Vue/TypeScript; lint covers the scaffold frontend and Rust formatting/linting; tests run once without watch mode. Build produces the frontend and a Windows x64 Tauri executable without installer/bundle/signing steps.
- [x] A4 — The simple command behavior has a focused runnable automated check; check the frontend invoke/result/error path with a small test if it contains that logic. Any mocked bridge test is labeled as such and does not replace the real Tauri invocation in A2. Do not add unrelated test infrastructure or coverage gates.
- [x] A5 — README documents the rewrite location, actual required tool versions and Windows prerequisites, dependency installation, development, all four verification scripts and the executable output location. It distinguishes the rewrite scaffold from the retained C#/WinUI reference and does not claim excluded functionality or release readiness.
- [x] A6 — `.gitignore` additions exclude the rewrite's dependency and generated output directories, including Node modules, frontend distribution output and Rust target output, while keeping source/configuration and lockfiles trackable. Existing ignore rules remain effective.
- [x] A7 — Legacy source, projects, solution, checks, packaging and SDK configuration remain unchanged relative to the implementation base. Changes are limited to the rewrite scaffold, README, `.gitignore` and this task specification/evidence. No excluded feature or unrelated cleanup is included.

## Verification

All implementation checks below were run in `rewrite/` using npm's Windows entrypoint and the committed npm/Cargo lockfiles.

The chosen rewrite directory is `rewrite/`. Package scripts provide the checks described below; the `build` script uses the Windows x64 target and disables bundling. The desktop build was checked separately from the Vite output.

Evidence below refers to the 2026-09-07 working snapshot: base `01f9d3940763843caf4a2b4d2f46eefce5a3dc70`, tested HEAD unchanged, tracked-diff SHA-1 `5388e05be78d48655eb701daff2dd2c4cf0c1217`, and status limited to `.gitignore`, `README.md`, this spec and `rewrite/`. Executable: `rewrite/src-tauri/target/x86_64-pc-windows-msvc/release/rewrite.exe`, 4,116,992 bytes, SHA-256 `3C61A4965C2FDE9AD9CF015975A1B1B3CE7B2E4CD26A63FB983C47906D5D1419`.

| Check | Command/action and expected result | Actual result / exit code | Evidence / snapshot |
| --- | --- | --- | --- |
| Environment | `node --version`; `npm.cmd --version`; `rustc -vV`; `cargo --version`. | Windows 11 x64; Node v24.13.1, npm 11.10.0, Rust/Cargo 1.93.0 MSVC x86_64; exit 0 | Current environment and executable snapshot above |
| Reproducible dependencies | In `rewrite/`, `npm.cmd ci`; `cargo fetch --locked --manifest-path src-tauri/Cargo.toml`. | exit 0; lockfiles present | `rewrite/package-lock.json`, `rewrite/src-tauri/Cargo.lock` |
| Lint — A3 | In `rewrite/`, `npm.cmd run lint`. | exit 0; frontend lint, rustfmt and Clippy passed | Round 2 reviewer evidence |
| Typecheck — A3 | In `rewrite/`, `npm.cmd run typecheck`. | exit 0; Vue/TypeScript passed | Round 2 reviewer evidence |
| Tests — A4 | In `rewrite/`, `npm.cmd run test`. | exit 0; 2 Vitest + 1 Rust test, no watch mode | `rewrite/src/bridge.test.ts` and Rust test output |
| Windows desktop build — A3 | In `rewrite/`, `npm.cmd run build`. | exit 0; frontend and x86_64-pc-windows-msvc executable produced, no installer/signing | Executable path/SHA above |
| Real bridge — A2 | Run dev and release; activate with mouse and keyboard; observe response; close and verify cleanup. | Dev: mouse + Enter → `Resposta: Hello from Rust!`, cleanup 0. Release: mouse + Tab/Enter → `Resposta: Hello from Rust!`, exit 0, cleanup 0 | Final implementation validation; executable snapshot above |
| Documentation/ignore — A5/A6 | Follow README; `git check-ignore` generated paths; inspect lockfiles. | exit 0; `rewrite/node_modules`, `rewrite/dist`, `rewrite/src-tauri/target` ignored; lockfiles not ignored | Ignore verification output |
| Scope/legacy — A7 | `git diff --check`; inspect complete status/diff. | exit 0; only rewrite, README, `.gitignore`, spec changed; legacy untouched | Current status and reviewers 1–3 |

No .NET rebuild or legacy runtime validation is required for an unchanged reference application. If legacy files need changes, return that conflict to the orchestrator rather than silently expanding scope. Missing local prerequisites are execution gaps to record; do not mark unexecuted checks as passed.

## Remediation

### Round 1 review

- [x] [CRITICAL] A2: execute and record mouse and keyboard interaction with the real bridge in both development and release windows, observe the deterministic Rust response, and confirm process cleanup - spec:31,54.
- [x] [NON_CRITICAL] Update loop state, sustained acceptance checkboxes, verification table, snapshot identity, command exit codes and executable hash with actual evidence - spec:16,40.

### Round 2 review

- [x] [CRITICAL] A2: validate the bridge in the release executable with mouse and keyboard, observe `Resposta: Hello from Rust!`, and confirm process cleanup - spec:31,54.
- [x] [NON_CRITICAL] Reconcile A1-A7 checkboxes, verification table, snapshot/hash evidence and remediation history with actual results - spec:30,40,46,64.

### Round 3 review

- [x] [CRITICAL] A2: prove keyboard activation of the bridge command in the release executable, observe `Resposta: Hello from Rust!`, and record cleanup - spec:31.
- [x] [NON_CRITICAL] Reconcile A1-A7 checkboxes, verification table, interaction evidence, snapshot/hash and remediation history - spec:16,48.

### Round 4 review

- [ ] [NON_CRITICAL] Replace the contradictory Gaps statement at spec:84 with the remaining Git delivery state or state that no technical gap remains - spec:84.

## Delivery

- Review verdict and evidence: REVISE; final reviewer found the contradictory Gaps statement at spec:84 after verifying the technical diff and evidence.
- Delivery status: blocked by final review REVISE; no delivery allowed under the four-round limit.
- Specification delivery: only `docs/superpowers/plans/2026-09-07-issue-18-tauri-vue-scaffold.md`; no implementation or Git mutations in this retry.
- Future delivery: after independent PASS and the orchestrator's delivery gate, implementation PR targets `tauri-version`, created from `main`, with implementation work on a branch derived from `tauri-version`. Record the actual base SHA and branch names; do not carry unrelated issue #8 changes from the inspection branch. No merge.
- MR URL: none; no PR created.
- Gaps: no technical gap remains. Only the future Git delivery state remains: create `tauri-version` from `main`, derive the implementation branch from it, then open the PR; this was not performed because review 4 returned REVISE.
