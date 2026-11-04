import { invoke } from '@tauri-apps/api/core'
import type { CodexLoginResult } from './codexLogin'
import type { UsageSnapshot } from './domain/usage'

export function invokeGreeting(): Promise<string> {
  return invoke<string>('greet')
}

export function getUsage(): Promise<UsageSnapshot> {
  return invoke<UsageSnapshot>('get_usage')
}

export function resizeUsagePanel(height: number): Promise<void> {
  return invoke<void>('resize_usage_panel', { height })
}

export function startCodexLogin(profileId?: string): Promise<void> {
  return profileId ? invoke<void>('start_codex_login', { profileId }) : invoke<void>('start_codex_login')
}

export function pollCodexLogin(): Promise<CodexLoginResult> {
  return invoke<CodexLoginResult>('poll_codex_login')
}

export function cancelCodexLogin(): Promise<void> {
  return invoke<void>('cancel_codex_login')
}

export function logoutCodex(profileId?: string): Promise<void> {
  return profileId ? invoke<void>('logout_codex', { profileId }) : invoke<void>('logout_codex')
}

export function deleteAccount(key: string, active: boolean, profileId?: string): Promise<void> {
  return invoke<void>('delete_account', { key, active, profileId })
}

export function describeBridgeError(error: unknown): string {
  if (error instanceof Error && error.message) {
    return error.message
  }

  return String(error)
}
