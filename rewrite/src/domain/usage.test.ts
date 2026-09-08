import { describe, expect, it } from 'vitest'
import { claudeAvailable, claudeError, codexAvailable, opencodePartial, usageFixtures } from './usage.fixtures'
import { sortProviderUsage, type UsageState } from './usage'

describe('provider-neutral usage contract', () => {
  it('serializes every fixture without losing unknown values', () => {
    for (const fixture of usageFixtures) expect(JSON.parse(JSON.stringify(fixture))).toEqual(fixture)
    const unknown = opencodePartial.quotas[1]
    expect(unknown.percentage).toBeNull()
    expect(unknown.used).toBeNull()
    expect(unknown.limit).toBeNull()
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
})
