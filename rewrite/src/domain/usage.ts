export type ProviderId = 'codex' | 'opencode' | 'claude'
export type UsageState = 'loading' | 'available' | 'partial' | 'stale' | 'unauthenticated' | 'not-installed' | 'error'

export interface UsageQuota {
  id: string
  label: string
  used: number | null
  limit: number | null
  percentage: number | null
  resetAt: string | null
}

export interface ProviderUsage {
  schemaVersion: 1
  id: ProviderId
  name: string
  vendor: string
  state: UsageState
  capturedAt: string | null
  quotas: UsageQuota[]
  error: { code: string; message: string } | null
}

export interface UsageError {
  code: string
  message: string
}

export interface AccountIdentity {
  key: string
  provider: ProviderId
  email: string
  accountType: string
  plan: string | null
}

export interface AccountUsageSnapshot {
  account: AccountIdentity
  usage: ProviderUsage
  fetchedAt: string | null
}

export interface UsageSnapshot {
  schemaVersion: 2
  accounts: AccountUsageSnapshot[]
  activeAccountKey: string | null
  fetchedAt: string | null
  error?: UsageError | null
}

const providerOrder: ProviderId[] = ['codex', 'opencode', 'claude']
export function sortProviderUsage(items: ProviderUsage[]): ProviderUsage[] {
  return [...items]
    .sort((a, b) => providerOrder.indexOf(a.id) - providerOrder.indexOf(b.id))
    .map(item => ({ ...item, quotas: [...item.quotas].sort((a, b) => a.id.localeCompare(b.id)) }))
}
