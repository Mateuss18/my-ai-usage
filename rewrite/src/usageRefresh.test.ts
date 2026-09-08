import { describe, expect, it, vi } from 'vitest'

import type { UsageSnapshot } from './domain/usage'
import { createUsageRefresh } from './usageRefresh'

const snapshot: UsageSnapshot = {
  schemaVersion: 1,
  fetchedAt: '2026-09-08T12:00:00Z',
  providers: [{
    schemaVersion: 1,
    id: 'codex',
    name: 'Codex',
    vendor: 'OpenAI',
    state: 'available',
    capturedAt: '2026-09-08T11:59:00Z',
    error: null,
    quotas: [
      { id: 'session', label: '5 hours / session', used: 73, limit: 100, percentage: 73, resetAt: '2026-09-08T14:18:00Z' },
      { id: 'weekly', label: 'Weekly / all models', used: 21, limit: 100, percentage: 21, resetAt: '2026-09-14T12:00:00Z' },
    ],
  }],
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
  it('starts loading then maps an available snapshot with timestamp-derived labels', async () => {
    const controller = createUsageRefresh(() => Promise.resolve(snapshot), { now: () => new Date('2026-09-08T12:00:00Z') })

    expect(controller.provider.value).toMatchObject({ state: 'loading', statusLabel: 'Updating usage…', quotas: [] })

    await controller.refresh()

    expect(controller.provider.value).toMatchObject({
      id: 'codex', name: 'Codex', eyebrow: 'OpenAI', glyph: '✦', state: 'available', statusLabel: 'Updated just now',
      quotas: [
        { id: 'session', title: '5 hours / session', percentage: 73, resetLabel: 'Resets in 2 h 18 min', color: '#7dd3fc' },
        { id: 'weekly', title: 'Weekly / all models', percentage: 21, resetLabel: 'Resets in 6 d', color: '#c4b5fd' },
      ],
    })
  })

  it('coalesces concurrent refreshes into one load', async () => {
    let resolve!: () => void
    const load = vi.fn(() => new Promise<UsageSnapshot>(done => { resolve = () => done(snapshot) }))
    const controller = createUsageRefresh(load)

    const first = controller.refresh()
    const second = controller.refresh()
    expect(load).toHaveBeenCalledTimes(1)

    resolve()
    await Promise.all([first, second])
    expect(controller.provider.value.state).toBe('available')
  })

  it('derives partial status from the snapshot timestamp', async () => {
    const controller = createUsageRefresh(
      () => Promise.resolve({ ...snapshot, providers: [{ ...snapshot.providers[0], state: 'partial' }] }),
      { now: () => new Date('2026-09-08T12:00:00Z') },
    )

    await controller.refresh()

    expect(controller.provider.value).toMatchObject({ state: 'partial', statusLabel: 'Updated just now — some usage data is unavailable.' })
  })

  it('keeps a valid snapshot as stale when a later refresh rejects', async () => {
    const load = vi.fn<() => Promise<UsageSnapshot>>()
      .mockResolvedValueOnce(snapshot)
      .mockRejectedValueOnce(new Error('bridge offline'))
    const controller = createUsageRefresh(load, { now: () => new Date('2026-09-08T12:18:00Z') })

    await controller.refresh()
    await controller.refresh()

    expect(controller.provider.value).toMatchObject({ state: 'stale', statusLabel: 'Last updated 18 minutes ago' })
    expect(controller.provider.value.quotas).toHaveLength(2)
  })

  it('keeps a valid snapshot as stale when the provider returns an error snapshot', async () => {
    const load = vi.fn<() => Promise<UsageSnapshot>>()
      .mockResolvedValueOnce(snapshot)
      .mockResolvedValueOnce({
        ...snapshot,
        fetchedAt: '2026-09-08T12:18:00Z',
        providers: [{ ...snapshot.providers[0], state: 'error', capturedAt: null, quotas: [], error: { code: 'timeout', message: 'Codex did not respond in time.' } }],
      })
    const controller = createUsageRefresh(load, { now: () => new Date('2026-09-08T12:18:00Z') })

    await controller.refresh()
    await controller.refresh()

    expect(controller.provider.value).toMatchObject({ state: 'stale', statusLabel: 'Last updated 18 minutes ago' })
    expect(controller.provider.value.quotas).toHaveLength(2)
  })

  it('shows an error when the first refresh rejects', async () => {
    const controller = createUsageRefresh(() => Promise.reject(new Error('bridge offline')))

    await controller.refresh()

    expect(controller.provider.value).toMatchObject({ state: 'error', statusLabel: 'Could not update usage. Try again.', quotas: [] })
  })

  it('keeps one visibility listener and timer across repeated starts and stops', async () => {
    vi.useFakeTimers()
    const document = fakeDocument()
    const load = vi.fn(() => Promise.resolve(snapshot))
    const controller = createUsageRefresh(load, { document: document as unknown as typeof globalThis.document, intervalMs: 60_000 })

    controller.start()
    controller.start()
    await controller.refresh()
    expect(document.addEventListener).toHaveBeenCalledTimes(1)
    expect(load).toHaveBeenCalledTimes(1)

    await vi.advanceTimersByTimeAsync(60_000)
    expect(load).toHaveBeenCalledTimes(2)

    document.setHidden(true)
    document.emitVisibilityChange()
    await vi.advanceTimersByTimeAsync(60_000)
    expect(load).toHaveBeenCalledTimes(2)

    document.setHidden(false)
    document.emitVisibilityChange()
    await vi.advanceTimersByTimeAsync(60_000)
    expect(load).toHaveBeenCalledTimes(3)

    controller.stop()
    controller.stop()
    expect(document.removeEventListener).toHaveBeenCalledTimes(1)
    await vi.advanceTimersByTimeAsync(60_000)
    expect(load).toHaveBeenCalledTimes(3)

    controller.start()
    await controller.refresh()
    expect(document.addEventListener).toHaveBeenCalledTimes(2)
    expect(load).toHaveBeenCalledTimes(4)
    await vi.advanceTimersByTimeAsync(60_000)
    expect(load).toHaveBeenCalledTimes(5)

    controller.stop()
    expect(document.removeEventListener).toHaveBeenCalledTimes(2)
    vi.useRealTimers()
  })
})
