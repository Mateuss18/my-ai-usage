import type { ProviderUsage } from './usageTypes'

const codexQuotas = [
  { id: 'session', title: '5 hours / session', percentage: 73, resetLabel: 'Resets in 2 h 18 min', color: '#7dd3fc', glyph: '✦' },
  { id: 'weekly', title: 'Weekly / all models', percentage: 21, resetLabel: 'Resets Monday at 9:00 AM', color: '#c4b5fd', glyph: '✦' },
]

export const usageFixtures: Record<string, ProviderUsage> = {
  codex: { id: 'codex', name: 'Codex', eyebrow: 'OpenAI', glyph: '✦', state: 'available', statusLabel: 'Updated just now', quotas: codexQuotas },
  claude: {
    id: 'claude', name: 'Claude', eyebrow: 'Anthropic', glyph: 'A', state: 'available', statusLabel: 'Updated 1 minute ago',
    quotas: [{ id: 'current-session', title: 'Current session across all supported models', percentage: 48, resetLabel: 'Resets in 3 hours and 42 minutes', color: '#fb923c', glyph: 'A' }],
  },
  opencode: {
    id: 'opencode', name: 'OpenCode', eyebrow: 'OpenCode', glyph: 'O', state: 'partial', statusLabel: 'Some usage data is unavailable.',
    quotas: [{ id: 'session', title: 'Current session', percentage: null, resetLabel: 'Reset time unavailable', color: '#86efac', glyph: 'O' }],
  },
  loading: { id: 'codex-loading', name: 'Codex', eyebrow: 'OpenAI', glyph: '✦', state: 'loading', statusLabel: 'Updating usage…', quotas: [] },
  unavailable: { id: 'codex-unavailable', name: 'Codex', eyebrow: 'OpenAI', glyph: '✦', state: 'unavailable', statusLabel: 'Usage is unavailable right now.', quotas: [] },
  error: { id: 'codex-error', name: 'Codex', eyebrow: 'OpenAI', glyph: '✦', state: 'error', statusLabel: 'Could not update usage. Try again.', quotas: [] },
  stale: { id: 'codex-stale', name: 'Codex', eyebrow: 'OpenAI', glyph: '✦', state: 'stale', statusLabel: 'Last updated 18 minutes ago', quotas: codexQuotas },
}

export function getUsageFixture(name: string | null): ProviderUsage {
  return usageFixtures[name ?? ''] ?? usageFixtures.codex
}
