export type UsageState = 'available' | 'loading' | 'partial' | 'unauthenticated' | 'not-installed' | 'unavailable' | 'error' | 'stale'

export interface UsageQuota {
  id: string
  title: string
  percentage: number | null
  resetLabel: string
  color: string
  glyph: string
}

export interface ProviderUsage {
  id: string
  name: string
  eyebrow: string
  glyph: string
  state: UsageState
  statusLabel: string
  quotas: UsageQuota[]
}
