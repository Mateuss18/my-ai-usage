import { describe, expect, it, vi } from 'vitest'

import { invoke } from '@tauri-apps/api/core'

import { cancelCodexLogin, describeBridgeError, getUsage, invokeGreeting, pollCodexLogin, startCodexLogin } from './bridge'

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
    const payload = { schemaVersion: 1 as const, providers: [], fetchedAt: null }
    vi.mocked(invoke).mockResolvedValue(payload)
    await expect(getUsage()).resolves.toEqual(payload)
    expect(invoke).toHaveBeenCalledWith('get_usage')
  })

  it('uses dedicated native commands for the Codex account flow', async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce(undefined)
      .mockResolvedValueOnce('completed')
      .mockResolvedValueOnce(undefined)

    await expect(startCodexLogin()).resolves.toBeUndefined()
    await expect(pollCodexLogin()).resolves.toBe('completed')
    await expect(cancelCodexLogin()).resolves.toBeUndefined()

    expect(invoke).toHaveBeenNthCalledWith(1, 'start_codex_login')
    expect(invoke).toHaveBeenNthCalledWith(2, 'poll_codex_login')
    expect(invoke).toHaveBeenNthCalledWith(3, 'cancel_codex_login')
  })
})
