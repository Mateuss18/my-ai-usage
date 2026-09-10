import { renderToString } from '@vue/server-renderer'
import { createSSRApp } from 'vue'
import { describe, expect, it } from 'vitest'

import appStyles from '../../App.vue?raw'
import panelStyles from './UsagePanel.vue?raw'
import ProgressRing from './ProgressRing.vue'
import UsagePanel from './UsagePanel.vue'
import { getUsageFixture, usageFixtures } from './usageFixtures'

describe('compact usage panel', () => {
  it('uses the high-contrast navy surface palette', () => {
    expect(appStyles).toContain('--app-surface-1: #0d1120')
    expect(appStyles).toContain('--app-text-secondary: #b2badc')
    expect(panelStyles).toContain('background: var(--app-surface-1)')
  })

  it.each([[-10, 0], [37, 37], [120, 100]])('renders a dynamic accessible ring for %s', async (percentage, expected) => {
    const html = await renderToString(createSSRApp(ProgressRing, { percentage, color: '#123abc', glyph: 'C', label: 'Session usage' }))

    expect(html).toContain('role="progressbar"')
    expect(html).toContain(`aria-valuenow="${expected}"`)
    expect(html).toContain('--ring-color:#123abc')
    expect(html).toContain(`--ring-value:${expected}%`)
  })

  it('renders every account with email, plan and an accessible Active or Cached label', async () => {
    const html = await renderToString(createSSRApp(UsagePanel, { accounts: usageFixtures.multi, loading: false, error: null }))

    expect(html).toContain('role="tablist"')
    expect(html).toContain('1 - owner')
    expect(html).toContain('2 - team@')
    expect(html.match(/role="tab"/g) ?? []).toHaveLength(2)
    expect(html).toContain('owner@example.com')
    expect(html).toContain('team@example.com')
    expect(html).toContain('Pro')
    expect(html).toContain('Active')
    expect(html).toContain('Cached')
    expect(html.match(/role="progressbar"/g) ?? []).toHaveLength(4)
  })

  it.each([
    ['partial', 'Updated just now — some usage data is unavailable.', 'role="progressbar"'],
    ['stale', 'Last updated 18 minutes ago', 'role="progressbar"'],
    ['unauthenticated', 'Sign in to Codex to read usage.', 'role="status"'],
    ['not-installed', 'Codex is not installed.', 'role="status"'],
  ])('renders the %s account state without hiding the account', async (fixtureName, expectedText, expectedRole) => {
    const html = await renderToString(createSSRApp(UsagePanel, { accounts: usageFixtures[fixtureName], loading: false, error: null }))

    expect(html).toContain(expectedText)
    expect(html).toContain(expectedRole)
  })

  it('uses a global loading or error state only when no accounts are available', async () => {
    const loading = await renderToString(createSSRApp(UsagePanel, { accounts: [], loading: true, error: null }))
    expect(loading).toContain('role="status"')
    expect(loading).toContain('Updating usage')

    const error = await renderToString(createSSRApp(UsagePanel, { accounts: [], loading: false, error: { code: 'bridge-error', message: 'Could not update usage. Try again.' } }))
    expect(error).toContain('role="alert"')
    expect(error).toContain('Could not update usage. Try again.')
    expect(error).toContain('Try again')
  })

  it('keeps loaded accounts visible beside a root error', async () => {
    const html = await renderToString(createSSRApp(UsagePanel, { accounts: usageFixtures.codex, loading: false, error: { code: 'partial', message: 'One provider failed.' } }))

    expect(html).toContain('owner@example.com')
    expect(html).toContain('One provider failed.')
    expect(html).not.toContain('Try again')
  })

  it('passes quota values and keeps unknown percentage unknown', async () => {
    const account = usageFixtures.codex[0]
    const usage = { ...account.usage, quotas: [{ ...account.usage.quotas[0], percentage: 37 }, { ...account.usage.quotas[0], id: 'unknown', percentage: null }] }
    const html = await renderToString(createSSRApp(UsagePanel, { accounts: [{ ...account, usage }], loading: false, error: null }))

    expect(html).toContain('aria-valuenow="37"')
    expect(html).toContain('>63%</strong>')
    expect(html).toContain('37% used')
    expect(html).toContain('>—</strong>')
    expect(html).not.toContain('aria-valuenow="0"')
  })

  it('falls back to Codex for an unknown fixture', () => {
    expect(getUsageFixture('unknown')).toBe(usageFixtures.codex)
  })

  it('renders sign-in, account switching and cancellation actions', async () => {
    const props = { accounts: usageFixtures.codex, loading: false, error: null }
    const signedOut = await renderToString(createSSRApp(UsagePanel, { ...props, authenticated: false }))
    const signedIn = await renderToString(createSSRApp(UsagePanel, { ...props, authenticated: true }))
    const authenticating = await renderToString(createSSRApp(UsagePanel, { ...props, authenticating: true }))
    const loginFailed = await renderToString(createSSRApp(UsagePanel, { ...props, loginFailed: true }))
    const loggingOut = await renderToString(createSSRApp(UsagePanel, { ...props, authenticated: true, logoutting: true }))

    expect(signedOut).toContain('aria-label="Sign in to Codex"')
    expect(signedIn).toContain('aria-label="Switch Codex account"')
    expect(signedIn).toContain('aria-label="Sign out of Codex"')
    expect(authenticating).toContain('aria-label="Cancel Codex sign-in"')
    expect(authenticating).toContain('Waiting for login in your browser')
    expect(loginFailed).toContain('Login failed. Try again.')
    expect(loggingOut).toContain('Signing out…')
    expect(loggingOut.match(/disabled/g) ?? []).toHaveLength(3)
  })
})
