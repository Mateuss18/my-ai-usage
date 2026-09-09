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
  accountName?: string | null
  state: UsageState
  capturedAt: string | null
  quotas: UsageQuota[]
  error: { code: string; message: string } | null
}

export interface UsageSnapshot {
  schemaVersion: 1
  providers: ProviderUsage[]
  fetchedAt: string | null
}

const providerOrder: ProviderId[] = ['codex', 'opencode', 'claude']
export function sortProviderUsage(items: ProviderUsage[]): ProviderUsage[] {
  return [...items]
    .sort((a, b) => providerOrder.indexOf(a.id) - providerOrder.indexOf(b.id))
    .map(item => ({ ...item, quotas: [...item.quotas].sort((a, b) => a.id.localeCompare(b.id)) }))
}
