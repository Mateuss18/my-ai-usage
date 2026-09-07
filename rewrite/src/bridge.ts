import { invoke } from '@tauri-apps/api/core'

export function invokeGreeting(): Promise<string> {
  return invoke<string>('greet')
}

export function describeBridgeError(error: unknown): string {
  if (error instanceof Error && error.message) {
    return error.message
  }

  return String(error)
}
