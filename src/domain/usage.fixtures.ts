import type { AccountUsageSnapshot, ProviderUsage } from './usage'

export const codexAvailable: ProviderUsage = {
  schemaVersion: 1, id: 'codex', name: 'Codex', vendor: 'OpenAI', state: 'available', capturedAt: '2026-09-08T12:00:00Z', error: null,
  quotas: [
    { id: 'session', label: '5 hours / session', used: 73, limit: 100, percentage: 73, resetAt: '2026-09-08T14:18:00Z' },
    { id: 'weekly', label: 'Weekly / all models', used: 21, limit: 100, percentage: 21, resetAt: '2026-09-14T12:00:00Z' },
  ],
}

export const opencodePartial: ProviderUsage = {
  schemaVersion: 1, id: 'opencode', name: 'OpenCode', vendor: 'OpenCode', state: 'partial', capturedAt: '2026-09-08T12:00:00Z', error: null,
  quotas: [
    { id: 'session', label: 'Current session', used: 42, limit: 100, percentage: 42, resetAt: null },
    { id: 'provider-limit', label: 'Provider limit', used: null, limit: null, percentage: null, resetAt: null },
  ],
}

export const claudeAvailable: ProviderUsage = {
  schemaVersion: 1, id: 'claude', name: 'Claude', vendor: 'Anthropic', state: 'available', capturedAt: '2026-09-08T11:00:00Z', error: null,
  quotas: [{ id: 'session', label: 'Current session across all supported models', used: 48, limit: 100, percentage: 48, resetAt: '2026-09-08T15:42:00Z' }],
}

export const claudeError: ProviderUsage = {
  schemaVersion: 1, id: 'claude', name: 'Claude', vendor: 'Anthropic', state: 'error', capturedAt: null, quotas: [],
  error: { code: 'provider-unavailable', message: 'Could not update usage.' },
}

export const codexAccount: AccountUsageSnapshot = {
  account: { key: 'codex:owner@example.com', provider: 'codex', email: 'owner@example.com', accountType: 'personal', plan: 'Pro' },
  usage: codexAvailable,
  fetchedAt: '2026-09-08T12:00:00Z',
}

export const usageFixtures = [codexAvailable, opencodePartial, claudeAvailable, claudeError]
