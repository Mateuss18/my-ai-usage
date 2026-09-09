import { shallowRef } from 'vue'

export type CodexLoginResult = 'pending' | 'completed' | 'failed'
export type CodexLoginState = 'idle' | 'starting' | 'waiting' | 'cancelling' | 'error'

export interface CodexLoginBridge {
  start: () => Promise<void>
  poll: () => Promise<CodexLoginResult>
  cancel: () => Promise<void>
}

export interface CodexLoginOptions {
  intervalMs?: number
  wait?: () => Promise<void>
}

export function createCodexLogin(bridge: CodexLoginBridge, options: CodexLoginOptions = {}) {
  const state = shallowRef<CodexLoginState>('idle')
  const intervalMs = options.intervalMs ?? 500
  const wait = options.wait ?? (() => new Promise<void>(resolve => globalThis.setTimeout(resolve, intervalMs)))
  let activeLogin: Promise<boolean> | undefined
  let cancellation: Promise<void> | undefined

  function begin(): Promise<boolean> {
    if (activeLogin) return activeLogin

    state.value = 'starting'
    const login = (async () => {
      try {
        await bridge.start()
        if (state.value === 'starting') state.value = 'waiting'
        while (state.value === 'waiting') {
          const result = await bridge.poll()
          if (result === 'completed') {
            state.value = 'idle'
            return true
          }
          if (result === 'failed') {
            state.value = 'error'
            return false
          }
          await wait()
        }
        if (cancellation) await cancellation
      } catch {
        state.value = 'error'
      }
      return false
    })()

    activeLogin = login
    void login.finally(() => { activeLogin = undefined })
    return login
  }

  async function cancel(): Promise<void> {
    if (state.value !== 'starting' && state.value !== 'waiting') return
    state.value = 'cancelling'
    cancellation = bridge.cancel()
    try {
      await cancellation
      state.value = 'idle'
    } catch {
      state.value = 'error'
    } finally {
      cancellation = undefined
    }
  }

  return { state, begin, cancel }
}
