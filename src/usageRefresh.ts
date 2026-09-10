import { shallowRef } from 'vue'

import type { AccountUsage as PanelAccountUsage, ProviderUsage as PanelProviderUsage } from './components/usage/usageTypes'
import type { AccountUsageSnapshot, ProviderUsage, UsageError, UsageSnapshot } from './domain/usage'

type VisibilityDocument = Pick<typeof globalThis.document, 'hidden' | 'addEventListener' | 'removeEventListener'>

export interface UsageRefreshOptions {
  document?: VisibilityDocument
  intervalMs?: number
  now?: () => Date
}

const colors: Record<string, string> = { session: '#2678FD', weekly: '#8974FD' }

export function createUsageRefresh(load: () => Promise<UsageSnapshot>, options: UsageRefreshOptions = {}) {
  const accounts = shallowRef<PanelAccountUsage[]>([])
  const loading = shallowRef(true)
  const error = shallowRef<UsageError | null>(null)
  const now = options.now ?? (() => new Date())
  const document = options.document ?? (typeof globalThis.document === 'undefined' ? undefined : globalThis.document)
  const intervalMs = options.intervalMs ?? 60_000
  const lastValidByAccount = new Map<string, AccountUsageSnapshot>()
  const accountRecords = new Map<string, AccountUsageSnapshot>()
  let activeAccountKey: string | null = null
  let refreshing: Promise<void> | undefined
  let refreshVersion = 0
  let timer: ReturnType<typeof globalThis.setInterval> | undefined
  let started = false

  function refresh(): Promise<void> {
    if (refreshing) return refreshing
    if (!accountRecords.size) loading.value = true

    const version = ++refreshVersion
    refreshing = load()
      .then(snapshot => {
        if (version !== refreshVersion) return
        error.value = snapshot.error ?? null
        for (const next of snapshot.accounts) {
          const previous = lastValidByAccount.get(next.account.key)
          if (isUsable(next.usage)) {
            lastValidByAccount.set(next.account.key, next)
            accountRecords.set(next.account.key, next)
          } else if (previous) {
            accountRecords.set(next.account.key, asStale(previous))
          } else {
            accountRecords.set(next.account.key, next)
          }
        }
        activeAccountKey = snapshot.activeAccountKey && accountRecords.has(snapshot.activeAccountKey)
          ? snapshot.activeAccountKey
          : null
        renderAccounts()
      })
      .catch(() => {
        if (version !== refreshVersion) return
        error.value = { code: 'bridge-error', message: 'Could not update usage. Try again.' }
        for (const [key, previous] of lastValidByAccount) accountRecords.set(key, asStale(previous))
        renderAccounts()
      })
      .finally(() => {
        if (version !== refreshVersion) return
        loading.value = false
        refreshing = undefined
      })

    return refreshing
  }

  function renderAccounts(): void {
    accounts.value = [...accountRecords.entries()].map(([key, snapshot]) => ({
      account: snapshot.account,
      usage: toPanelUsage(snapshot.usage, snapshot.fetchedAt, now()),
      accountStatus: key === activeAccountKey ? 'Active' : 'Cached',
    }))
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
    else {
      updateTimer()
      void refresh()
    }
  }

  function start(): void {
    if (started) return
    started = true
    document?.addEventListener('visibilitychange', onVisibilityChange)
    onVisibilityChange()
  }

  function stop(): void {
    if (!started) return
    started = false
    clearTimer()
    document?.removeEventListener('visibilitychange', onVisibilityChange)
    refreshVersion += 1
    refreshing = undefined
  }

  return { accounts, loading, error, refresh, start, stop }
}

function isUsable(provider: ProviderUsage): boolean {
  return (provider.state === 'available' || provider.state === 'partial') && provider.quotas.length > 0
}

function asStale(snapshot: AccountUsageSnapshot): AccountUsageSnapshot {
  return { ...snapshot, usage: { ...snapshot.usage, state: 'stale', error: null } }
}

function toPanelUsage(source: ProviderUsage, fetchedAt: string | null, now: Date): PanelProviderUsage {
  return {
    id: source.id,
    name: source.name,
    eyebrow: source.vendor,
    glyph: '✦',
    state: source.state,
    statusLabel: statusLabel(source.state, fetchedAt, source.error?.message, now),
    quotas: source.quotas.map(quota => ({
      id: quota.id,
      title: quota.label,
      percentage: quota.percentage,
      resetLabel: resetLabel(quota.resetAt, now, quota.id !== 'session'),
      color: colors[quota.id] ?? '#2678FD',
      glyph: '✦',
    })),
  }
}

function statusLabel(state: PanelProviderUsage['state'], timestamp: string | null, error: string | undefined, now: Date): string {
  if (state === 'loading') return 'Updating usage…'
  if (state === 'stale') return `Last updated ${timeAgo(timestamp, now)}`
  if (state === 'available') return `Updated ${timeAgo(timestamp, now)}`
  if (state === 'partial') return `Updated ${timeAgo(timestamp, now)} — some usage data is unavailable.`
  if (state === 'unauthenticated') return error ?? 'Sign in to read usage.'
  if (state === 'not-installed') return error ?? 'Provider is not installed.'
  return error ?? 'Could not update usage. Try again.'
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

function resetLabel(timestamp: string | null, now: Date, includeDate: boolean): string {
  const time = timestamp ? Date.parse(timestamp) : Number.NaN
  if (Number.isNaN(time)) return 'Reset time unavailable'
  let minutes = Math.max(0, Math.ceil((time - now.getTime()) / 60_000))
  if (!minutes) return `Resets now · ${resetTimeInBrasilia(time, includeDate)}`
  const days = Math.floor(minutes / 1_440)
  minutes -= days * 1_440
  const hours = Math.floor(minutes / 60)
  minutes -= hours * 60
  return `Resets in ${[days && `${days} d`, hours && `${hours} h`, minutes && `${minutes} min`].filter(Boolean).join(' ')} · ${resetTimeInBrasilia(time, includeDate)}`
}

function resetTimeInBrasilia(timestamp: number, includeDate: boolean): string {
  const parts = new Intl.DateTimeFormat('pt-BR', {
    timeZone: 'America/Sao_Paulo', day: '2-digit', month: '2-digit', hour: '2-digit', minute: '2-digit', hour12: false,
  }).formatToParts(new Date(timestamp))
  const value = (type: Intl.DateTimeFormatPartTypes) => parts.find(part => part.type === type)?.value ?? ''
  return `${includeDate ? `${value('day')}/${value('month')} ` : ''}${value('hour')}:${value('minute')}`
}
