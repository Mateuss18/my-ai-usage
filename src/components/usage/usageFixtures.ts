import type { AccountUsage } from './usageTypes'

const codexQuotas = [
  { id: 'session', title: '5 hours / session', percentage: 73, resetLabel: 'Resets in 2 h 18 min', color: '#7dd3fc', glyph: '✦' },
  { id: 'weekly', title: 'Weekly / all models', percentage: 21, resetLabel: 'Resets Monday at 9:00 AM', color: '#c4b5fd', glyph: '✦' },
]

const account = (key: string, email: string, accountType: string, plan: string | null, accountStatus: AccountUsage['accountStatus'] = 'Active'): AccountUsage => ({
  account: { key, provider: 'codex', email, accountType, plan },
  accountStatus,
  usage: { id: 'codex', name: 'Codex', eyebrow: 'OpenAI', glyph: '✦', state: 'available', statusLabel: 'Updated just now', quotas: codexQuotas },
})

export const usageFixtures: Record<string, AccountUsage[]> = {
  codex: [account('codex:owner@example.com', 'owner@example.com', 'personal', 'Pro')],
  multi: [
    account('codex:owner@example.com', 'owner@example.com', 'personal', 'Pro'),
    account('codex:team@example.com', 'team@example.com', 'team', null, 'Cached'),
  ],
  loading: [],
  error: [],
  partial: [{ ...account('codex:owner@example.com', 'owner@example.com', 'personal', 'Pro'), usage: { ...account('codex:owner@example.com', 'owner@example.com', 'personal', 'Pro').usage, state: 'partial', statusLabel: 'Updated just now — some usage data is unavailable.' } }],
  stale: [{ ...account('codex:owner@example.com', 'owner@example.com', 'personal', 'Pro'), usage: { ...account('codex:owner@example.com', 'owner@example.com', 'personal', 'Pro').usage, state: 'stale', statusLabel: 'Last updated 18 minutes ago' } }],
  unauthenticated: [{ ...account('codex:owner@example.com', 'owner@example.com', 'personal', 'Pro'), usage: { ...account('codex:owner@example.com', 'owner@example.com', 'personal', 'Pro').usage, state: 'unauthenticated', statusLabel: 'Sign in to Codex to read usage.', quotas: [] } }],
  'not-installed': [{ ...account('codex:owner@example.com', 'owner@example.com', 'personal', 'Pro'), usage: { ...account('codex:owner@example.com', 'owner@example.com', 'personal', 'Pro').usage, state: 'not-installed', statusLabel: 'Codex is not installed.', quotas: [] } }],
}

export function getUsageFixture(name: string | null): AccountUsage[] {
  return usageFixtures[name ?? ''] ?? usageFixtures.codex
}
