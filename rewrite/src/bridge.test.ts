import { describe, expect, it, vi } from 'vitest'

import { invoke } from '@tauri-apps/api/core'

import { describeBridgeError, getUsage, invokeGreeting } from './bridge'

// Mocked bridge check; A2 still requires the real Tauri invocation.
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))

describe('frontend-to-Tauri bridge', () => {
  it('returns the Rust command response', async () => {
    vi.mocked(invoke).mockResolvedValue('Hello from Rust!')

    await expect(invokeGreeting()).resolves.toBe('Hello from Rust!')
    expect(invoke).toHaveBeenCalledWith('greet')
  })

  it('preserves invocation failures for visible UI handling', async () => {
    const failure = new Error('bridge unavailable')
    vi.mocked(invoke).mockRejectedValue(failure)

    await expect(invokeGreeting()).rejects.toBe(failure)
    expect(describeBridgeError(failure)).toBe('bridge unavailable')
  })

  it('keeps the provider-neutral usage payload typed at the bridge', async () => {
    const payload = { schemaVersion: 2 as const, accounts: [], activeAccountKey: null, fetchedAt: null, error: null }
    vi.mocked(invoke).mockResolvedValue(payload)
    await expect(getUsage()).resolves.toEqual(payload)
    expect(invoke).toHaveBeenCalledWith('get_usage')
  })
})
