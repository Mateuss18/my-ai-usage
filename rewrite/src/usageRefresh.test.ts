import { describe, expect, it, vi } from 'vitest'

import type { AccountIdentity, AccountUsageSnapshot, ProviderUsage, UsageSnapshot } from './domain/usage'
import { createUsageRefresh } from './usageRefresh'

const accountA: AccountIdentity = { key: 'codex:a@example.com', provider: 'codex', email: 'a@example.com', accountType: 'personal', plan: 'Pro' }
const accountB: AccountIdentity = { key: 'codex:b@example.com', provider: 'codex', email: 'b@example.com', accountType: 'team', plan: null }

function usage(account: AccountIdentity, state: ProviderUsage['state'] = 'available', capturedAt = '2026-09-08T11:59:00Z'): ProviderUsage {
  return {
    schemaVersion: 1, id: account.provider, name: 'Codex', vendor: 'OpenAI', state, capturedAt,
    error: state === 'error' ? { code: 'timeout', message: `${account.email} timed out.` } : null,
    quotas: state === 'error' ? [] : [{ id: 'session', label: '5 hours / session', used: 73, limit: 100, percentage: 73, resetAt: '2026-09-08T14:18:00Z' }],
  }
}

function accountSnapshot(account: AccountIdentity, state: ProviderUsage['state'] = 'available', fetchedAt = '2026-09-08T12:00:00Z'): AccountUsageSnapshot {
  return { account, usage: usage(account, state), fetchedAt }
}

function snapshot(accounts: AccountUsageSnapshot[], activeAccountKey: string | null = accounts[0]?.account.key ?? null, error: UsageSnapshot['error'] = null): UsageSnapshot {
  return { schemaVersion: 2, accounts, activeAccountKey, fetchedAt: '2026-09-08T12:00:00Z', error }
}

function fakeDocument(hidden = false) {
  const listeners = new Set<() => void>()
  return {
    get hidden() { return hidden },
    setHidden(value: boolean) { hidden = value },
    addEventListener: vi.fn((_event: 'visibilitychange', listener: () => void) => listeners.add(listener)),
    removeEventListener: vi.fn((_event: 'visibilitychange', listener: () => void) => listeners.delete(listener)),
    emitVisibilityChange() { listeners.forEach(listener => listener()) },
  }
}

describe('live usage refresh', () => {
  it('returns accounts, global loading and root error instead of a provider', () => {
    const controller = createUsageRefresh(() => Promise.resolve(snapshot([])))

    expect(controller).toHaveProperty('accounts')
    expect(controller).toHaveProperty('loading')
    expect(controller).toHaveProperty('error')
    expect(controller).not.toHaveProperty('provider')
    expect(controller.loading.value).toBe(true)
  })

  it('loads every account and marks exactly one Active while the rest are Cached', async () => {
    const controller = createUsageRefresh(() => Promise.resolve(snapshot([accountSnapshot(accountA), accountSnapshot(accountB)], accountB.key)))

    await controller.refresh()

    expect(controller.accounts.value.map(item => item.account.key)).toEqual([accountA.key, accountB.key])
    expect(controller.accounts.value.map(item => item.account.email)).toEqual(['a@example.com', 'b@example.com'])
    expect(controller.accounts.value.map(item => item.accountStatus)).toEqual(['Cached', 'Active'])
    expect(controller.accounts.value.filter(item => item.accountStatus === 'Active')).toHaveLength(1)
  })

  it('keeps A and B once across A to B to A refreshes', async () => {
    const load = vi.fn()
      .mockResolvedValueOnce(snapshot([accountSnapshot(accountA)], accountA.key))
      .mockResolvedValueOnce(snapshot([accountSnapshot(accountB)], accountB.key))
      .mockResolvedValueOnce(snapshot([accountSnapshot(accountA)], accountA.key))
    const controller = createUsageRefresh(load)

    await controller.refresh()
    await controller.refresh()
    await controller.refresh()

    expect(controller.accounts.value.map(item => item.account.key)).toEqual([accountA.key, accountB.key])
    expect(controller.accounts.value.filter(item => item.account.key === accountA.key)).toHaveLength(1)
    expect(controller.accounts.value.find(item => item.account.key === accountA.key)?.accountStatus).toBe('Active')
    expect(controller.accounts.value.find(item => item.account.key === accountB.key)?.accountStatus).toBe('Cached')
  })

  it('preserves a valid account as stale when that account errors, without hiding others', async () => {
    const load = vi.fn()
      .mockResolvedValueOnce(snapshot([accountSnapshot(accountA), accountSnapshot(accountB)], accountA.key))
      .mockResolvedValueOnce(snapshot([accountSnapshot(accountA, 'error'), accountSnapshot(accountB, 'partial')], accountB.key))
    const controller = createUsageRefresh(load, { now: () => new Date('2026-09-08T12:18:00Z') })

    await controller.refresh()
    await controller.refresh()

    expect(controller.accounts.value).toEqual(expect.arrayContaining([
      expect.objectContaining({ account: accountA, accountStatus: 'Cached', usage: expect.objectContaining({ state: 'stale', statusLabel: 'Last updated 18 minutes ago' }) }),
      expect.objectContaining({ account: accountB, accountStatus: 'Active', usage: expect.objectContaining({ state: 'partial', statusLabel: 'Updated 18 minutes ago — some usage data is unavailable.' }) }),
    ]))
    expect(controller.accounts.value.find(item => item.account.key === accountA.key)?.usage.quotas).toHaveLength(1)
  })

  it('preserves all cached accounts after a bridge failure and exposes root error', async () => {
    const load = vi.fn()
      .mockResolvedValueOnce(snapshot([accountSnapshot(accountA), accountSnapshot(accountB)], accountA.key))
      .mockRejectedValueOnce(new Error('bridge offline'))
    const controller = createUsageRefresh(load)

    await controller.refresh()
    await controller.refresh()

    expect(controller.accounts.value).toHaveLength(2)
    expect(controller.accounts.value.every(item => item.usage.state === 'stale')).toBe(true)
    expect(controller.error.value).toEqual({ code: 'bridge-error', message: 'Could not update usage. Try again.' })
  })

  it('shows an error with no cache and preserves loaded accounts alongside root error', async () => {
    const empty = createUsageRefresh(() => Promise.reject(new Error('bridge offline')))
    await empty.refresh()
    expect(empty.accounts.value).toEqual([])
    expect(empty.loading.value).toBe(false)
    expect(empty.error.value).toEqual({ code: 'bridge-error', message: 'Could not update usage. Try again.' })

    const loaded = createUsageRefresh(() => Promise.resolve(snapshot([accountSnapshot(accountA)], accountA.key, { code: 'partial-root', message: 'One provider failed.' })))
    await loaded.refresh()
    expect(loaded.accounts.value).toHaveLength(1)
    expect(loaded.error.value).toEqual({ code: 'partial-root', message: 'One provider failed.' })
  })

  it('does not invent an Active account when the bridge reports no current identity', async () => {
    const controller = createUsageRefresh(() => Promise.resolve(snapshot([
      accountSnapshot(accountA, 'stale'),
      accountSnapshot(accountB, 'stale'),
    ], null, { code: 'unauthenticated', message: 'Sign in to Codex to read usage.' })))

    await controller.refresh()

    expect(controller.accounts.value.every(item => item.accountStatus === 'Cached')).toBe(true)
    expect(controller.error.value?.code).toBe('unauthenticated')
  })

  it('coalesces concurrent refreshes into one load', async () => {
    let resolve!: () => void
    const load = vi.fn(() => new Promise<UsageSnapshot>(done => { resolve = () => done(snapshot([accountSnapshot(accountA)])) }))
    const controller = createUsageRefresh(load)

    const first = controller.refresh()
    const second = controller.refresh()
    expect(load).toHaveBeenCalledTimes(1)

    resolve()
    await Promise.all([first, second])
    expect(controller.accounts.value).toHaveLength(1)
  })

  it('refreshes immediately when a hidden document becomes visible and keeps one timer', async () => {
    vi.useFakeTimers()
    const document = fakeDocument(true)
    const load = vi.fn(() => Promise.resolve(snapshot([accountSnapshot(accountA)])))
    const controller = createUsageRefresh(load, { document: document as unknown as typeof globalThis.document, intervalMs: 60_000 })

    controller.start()
    expect(load).not.toHaveBeenCalled()
    document.setHidden(false)
    document.emitVisibilityChange()
    await Promise.resolve()
    await Promise.resolve()
    expect(load).toHaveBeenCalledTimes(1)
    await vi.advanceTimersByTimeAsync(60_000)
    expect(load).toHaveBeenCalledTimes(2)
    controller.stop()
    vi.useRealTimers()
  })
})
