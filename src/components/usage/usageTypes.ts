import type { AccountIdentity, UsageError, UsageState } from '../../domain/usage'

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

export interface AccountUsage {
  account: AccountIdentity
  usage: ProviderUsage
  accountStatus: 'Active' | 'Cached'
}

export type { UsageError, UsageState }
