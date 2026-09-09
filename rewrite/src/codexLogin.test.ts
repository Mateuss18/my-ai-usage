import { describe, expect, it, vi } from 'vitest'

import { createCodexLogin } from './codexLogin'

describe('Codex account switching', () => {
  it('returns completion after the native login reports complete', async () => {
    const bridge = {
      start: vi.fn().mockResolvedValue(undefined),
      poll: vi.fn().mockResolvedValue('completed'),
      cancel: vi.fn().mockResolvedValue(undefined),
    }
    const login = createCodexLogin(bridge, { wait: async () => {} })

    await expect(login.begin()).resolves.toBe(true)

    expect(login.state.value).toBe('idle')
    expect(bridge.poll).toHaveBeenCalledOnce()
  })

  it('leaves a failed login recoverable', async () => {
    const login = createCodexLogin({
      start: async () => {},
      poll: async () => 'failed',
      cancel: async () => {},
    }, { wait: async () => {} })

    await expect(login.begin()).resolves.toBe(false)

    expect(login.state.value).toBe('error')
  })

  it('cancels a pending login without continuing to poll', async () => {
    const bridge = {
      start: vi.fn().mockResolvedValue(undefined),
      poll: vi.fn().mockResolvedValue('pending'),
      cancel: vi.fn().mockResolvedValue(undefined),
    }
    const login = createCodexLogin(bridge, { wait: () => login.cancel() })

    await expect(login.begin()).resolves.toBe(false)

    expect(login.state.value).toBe('idle')
    expect(bridge.cancel).toHaveBeenCalledOnce()
    expect(bridge.poll).toHaveBeenCalledOnce()
  })

  it('shows browser waiting only after native login has opened the URL', async () => {
    let start!: () => void
    let complete!: () => void
    const login = createCodexLogin({
      start: () => new Promise<void>(resolve => { start = resolve }),
      poll: () => new Promise<'completed'>(resolve => { complete = () => resolve('completed') }),
      cancel: async () => {},
    })

    const attempt = login.begin()
    expect(login.state.value).toBe('starting')

    start()
    await Promise.resolve()
    expect(login.state.value).toBe('waiting')

    complete()
    await expect(attempt).resolves.toBe(true)
  })
})
