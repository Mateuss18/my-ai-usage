import { shallowRef } from 'vue'

import type { ProviderUsage as PanelProviderUsage } from './components/usage/usageTypes'
import type { ProviderUsage, UsageSnapshot } from './domain/usage'

type VisibilityDocument = Pick<typeof globalThis.document, 'hidden' | 'addEventListener' | 'removeEventListener'>

export interface UsageRefreshOptions {
  document?: VisibilityDocument
  intervalMs?: number
  now?: () => Date
}

const colors: Record<string, string> = { session: '#7dd3fc', weekly: '#c4b5fd' }
const defaultProvider: PanelProviderUsage = {
  id: 'codex', name: 'Codex', eyebrow: 'OpenAI', glyph: '✦', state: 'loading', statusLabel: 'Updating usage…', quotas: [],
}

export function createUsageRefresh(load: () => Promise<UsageSnapshot>, options: UsageRefreshOptions = {}) {
  const provider = shallowRef<PanelProviderUsage>(defaultProvider)
  const now = options.now ?? (() => new Date())
  const document = options.document ?? (typeof globalThis.document === 'undefined' ? undefined : globalThis.document)
  const intervalMs = options.intervalMs ?? 60_000
  let lastValid: { provider: ProviderUsage; fetchedAt: string | null } | undefined
  let refreshing: Promise<void> | undefined
  let timer: ReturnType<typeof globalThis.setInterval> | undefined
  let started = false

  function refresh(): Promise<void> {
    if (refreshing) return refreshing

    refreshing = load()
      .then(snapshot => {
        const next = snapshot.providers.find(item => item.id === 'codex')
        if (!next) throw new Error('Codex usage is unavailable.')

        if (!isValid(next) && lastValid) {
          provider.value = toPanelProvider(lastValid.provider, lastValid.fetchedAt, now(), true)
          return
        }

        if (isValid(next)) lastValid = { provider: next, fetchedAt: snapshot.fetchedAt }
        provider.value = toPanelProvider(next, snapshot.fetchedAt, now())
      })
      .catch(() => {
        provider.value = lastValid
          ? toPanelProvider(lastValid.provider, lastValid.fetchedAt, now(), true)
          : { ...defaultProvider, state: 'error', statusLabel: 'Could not update usage. Try again.' }
      })
      .finally(() => { refreshing = undefined })

    return refreshing
  }

  function updateTimer(): void {
    if (document?.hidden || timer) return
    timer = globalThis.setInterval(() => { void refresh() }, intervalMs)
  }

  function clearTimer(): void {
    if (timer === undefined) return
    globalThis.clearInterval(timer)
    timer = undefined
  }

  function onVisibilityChange(): void {
    if (document?.hidden) clearTimer()
    else updateTimer()
  }

  function start(): void {
    if (started) return
    started = true
    document?.addEventListener('visibilitychange', onVisibilityChange)
    onVisibilityChange()
    if (!document?.hidden) void refresh()
  }

  function stop(): void {
    if (!started) return
    started = false
    clearTimer()
    document?.removeEventListener('visibilitychange', onVisibilityChange)
  }

  return { provider, refresh, start, stop }
}

function isValid(provider: ProviderUsage): boolean {
  return provider.state === 'available' || provider.state === 'partial' || provider.state === 'stale'
}

function toPanelProvider(source: ProviderUsage, fetchedAt: string | null, now: Date, stale = false): PanelProviderUsage {
  const state = stale ? 'stale' : source.state
  const timestamp = fetchedAt ?? source.capturedAt
  return {
    id: source.id,
    name: source.name,
    eyebrow: source.vendor,
    glyph: '✦',
    state,
    statusLabel: statusLabel(state, timestamp, source.error?.message, now),
    quotas: source.quotas.map(quota => ({
      id: quota.id,
      title: quota.label,
      percentage: quota.percentage,
      resetLabel: resetLabel(quota.resetAt, now),
      color: colors[quota.id] ?? '#7dd3fc',
      glyph: '✦',
    })),
  }
}

function statusLabel(state: PanelProviderUsage['state'], timestamp: string | null, error: string | undefined, now: Date): string {
  if (state === 'loading') return 'Updating usage…'
  if (state === 'stale') return `Last updated ${timeAgo(timestamp, now)}`
  if (state === 'available') return `Updated ${timeAgo(timestamp, now)}`
  if (state === 'partial') return `Updated ${timeAgo(timestamp, now)} — some usage data is unavailable.`
  return error ? `${error} Try again.` : 'Could not update usage. Try again.'
}

function timeAgo(timestamp: string | null, now: Date): string {
  const time = timestamp ? Date.parse(timestamp) : Number.NaN
  if (Number.isNaN(time)) return 'recently'
  const minutes = Math.max(0, Math.floor((now.getTime() - time) / 60_000))
  if (minutes < 1) return 'just now'
  if (minutes < 60) return `${minutes} minute${minutes === 1 ? '' : 's'} ago`
  const hours = Math.floor(minutes / 60)
  if (hours < 24) return `${hours} hour${hours === 1 ? '' : 's'} ago`
  const days = Math.floor(hours / 24)
  return `${days} day${days === 1 ? '' : 's'} ago`
}

function resetLabel(timestamp: string | null, now: Date): string {
  const time = timestamp ? Date.parse(timestamp) : Number.NaN
  if (Number.isNaN(time)) return 'Reset time unavailable'
  let minutes = Math.max(0, Math.ceil((time - now.getTime()) / 60_000))
  if (!minutes) return 'Resets now'
  const days = Math.floor(minutes / 1_440)
  minutes -= days * 1_440
  const hours = Math.floor(minutes / 60)
  minutes -= hours * 60
  return `Resets in ${[days && `${days} d`, hours && `${hours} h`, minutes && `${minutes} min`].filter(Boolean).join(' ')}`
}
