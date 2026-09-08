# Issue #23 Usage Refresh Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Connect the Codex usage provider to the compact panel with safe manual and automatic refreshes.

**Architecture:** Keep scheduling and snapshot state in one frontend controller. It calls the existing aggregate Tauri bridge, coalesces concurrent callers, retains a successful Codex payload after transient failures, and projects the provider-neutral DTO into the existing panel model. Vue only starts and disposes that controller.

**Tech Stack:** Vue 3 Composition API, TypeScript, Vitest, existing Tauri bridge.

**Spec:** https://github.com/Mateuss18/my-ai-usage/issues/23

## Global Constraints

- Base all behavior on the #21 DTO and existing `getUsage()` bridge; add no dependencies and no backend scheduler.
- Poll only while the webview is visible; start once, dispose its interval/listener once, and coalesce manual and timer refreshes.
- A valid Codex snapshot is `available`, `partial`, or `stale`; a failed refresh must keep it and project it as `stale`.
- Status text is derived from `fetchedAt` or `capturedAt`; never use fixture text for live values.

---

### Task 1: Implement the live usage refresh controller and wire it into the app

**Files:**
- Create: `rewrite/src/usageRefresh.ts`
- Test: `rewrite/src/usageRefresh.test.ts`
- Modify: `rewrite/src/App.vue`

**Interfaces:**
- Consumes: `getUsage(): Promise<UsageSnapshot>` from `rewrite/src/bridge.ts`.
- Produces: `createUsageRefresh(load, options)` with `provider`, `refresh`, `start`, and `stop` for Vue lifecycle wiring.

- [x] **Step 1: Write the failing controller tests**

Cover initial loading and timestamp-derived availability, concurrent refresh coalescing, stale preservation after a rejection, an error before any valid snapshot, and repeated start/stop with a visibility-driven timer.

- [x] **Step 2: Run the focused test to verify it fails**

Run: `npm.cmd run test:frontend -- src/usageRefresh.test.ts`
Expected: FAIL because `src/usageRefresh.ts` does not exist.

- [x] **Step 3: Implement the smallest controller and mapping**

Use the existing `UsagePanel` props: transform quota labels, colors, resets, state, and status label from the real DTO. `start()` owns one interval plus one visibility listener; `stop()` clears both. Do not change the Rust provider or its mutex.

- [x] **Step 4: Wire the controller into Vue lifecycle**

Render the controller's provider and connect the existing refresh event to its `refresh`; remove the fixture and page-reload flow.

- [x] **Step 5: Run focused and project checks**

Run: `npm.cmd run test:frontend -- src/usageRefresh.test.ts && npm.cmd run typecheck && npm.cmd run lint:frontend`
Expected: exit code 0.

- [x] **Step 6: Commit**

Commit the controller, test, app wiring, and this plan with `feat: orchestrate live usage refresh`.
