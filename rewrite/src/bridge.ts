import { invoke } from '@tauri-apps/api/core'
import type { UsageSnapshot } from './domain/usage'

export function invokeGreeting(): Promise<string> {
  return invoke<string>('greet')
}

export function getUsage(): Promise<UsageSnapshot> {
  return invoke<UsageSnapshot>('get_usage')
}

export function describeBridgeError(error: unknown): string {
  if (error instanceof Error && error.message) {
    return error.message
  }

  return String(error)
}
