import { describe, expect, it } from 'vitest'
import { claudeAvailable, claudeError, codexAccount, codexAvailable, opencodePartial, usageFixtures } from './usage.fixtures'
import { sortProviderUsage, type AccountIdentity, type UsageSnapshot, type UsageState } from './usage'

describe('provider-neutral usage contract', () => {
  it('keeps identity on account snapshots and not on ProviderUsage', () => {
    const identity: AccountIdentity = codexAccount.account
    const snapshot: UsageSnapshot = {
      schemaVersion: 2,
      accounts: [codexAccount],
      activeAccountKey: identity.key,
      fetchedAt: codexAccount.fetchedAt,
      error: null,
    }

    expect(JSON.parse(JSON.stringify(snapshot))).toEqual(snapshot)
    expect(codexAccount.usage).not.toHaveProperty('account')
    expect(identity).toEqual({ key: 'codex:owner@example.com', provider: 'codex', email: 'owner@example.com', accountType: 'personal', plan: 'Pro' })
  })

  it('orders providers predictably and keeps quota order from adapters', () => {
    expect(sortProviderUsage([claudeAvailable, codexAvailable, opencodePartial]).map(item => item.id)).toEqual(['codex', 'opencode', 'claude'])
    expect(codexAvailable.quotas.map(quota => quota.id)).toEqual(['session', 'weekly'])
  })

  it('represents all supported states and a clean error snapshot', () => {
    const states: UsageState[] = ['loading', 'available', 'partial', 'stale', 'unauthenticated', 'not-installed', 'error']
    expect(states).toHaveLength(7)
    expect(claudeError.quotas).toEqual([])
    expect(claudeError.error).toEqual({ code: 'provider-unavailable', message: 'Could not update usage.' })
  })

  it('serializes every provider fixture without losing unknown values', () => {
    for (const fixture of usageFixtures) expect(JSON.parse(JSON.stringify(fixture))).toEqual(fixture)
    expect(opencodePartial.quotas[1].percentage).toBeNull()
    expect(opencodePartial.quotas[1].used).toBeNull()
    expect(opencodePartial.quotas[1].limit).toBeNull()
  })
})
