import { invoke } from '@tauri-apps/api/core'
import type { CodexLoginResult } from './codexLogin'
import type { UsageSnapshot } from './domain/usage'

export function invokeGreeting(): Promise<string> {
  return invoke<string>('greet')
}

export function getUsage(): Promise<UsageSnapshot> {
  return invoke<UsageSnapshot>('get_usage')
}

export function startCodexLogin(): Promise<void> {
  return invoke<void>('start_codex_login')
}

export function pollCodexLogin(): Promise<CodexLoginResult> {
  return invoke<CodexLoginResult>('poll_codex_login')
}

export function cancelCodexLogin(): Promise<void> {
  return invoke<void>('cancel_codex_login')
}

export function describeBridgeError(error: unknown): string {
  if (error instanceof Error && error.message) {
    return error.message
  }

  return String(error)
}
