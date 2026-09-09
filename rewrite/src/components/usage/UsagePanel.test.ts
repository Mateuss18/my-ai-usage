import { renderToString } from '@vue/server-renderer'
import { createSSRApp } from 'vue'
import { describe, expect, it } from 'vitest'

import ProgressRing from './ProgressRing.vue'
import UsagePanel from './UsagePanel.vue'
import { getUsageFixture, usageFixtures } from './usageFixtures'

describe('compact usage panel', () => {
  it.each([[-10, 0], [37, 37], [120, 100]])('renders a dynamic accessible ring for %s', async (percentage, expected) => {
    const html = await renderToString(createSSRApp(ProgressRing, { percentage, color: '#123abc', glyph: 'C', label: 'Session usage' }))

    expect(html).toContain('role="progressbar"')
    expect(html).toContain(`aria-valuenow="${expected}"`)
    expect(html).toContain('--ring-color:#123abc')
    expect(html).toContain(`--ring-value:${expected}%`)
  })

  it('renders the Codex icon inside the ring as decorative content', async () => {
    const html = await renderToString(createSSRApp(ProgressRing, { percentage: 37, color: '#123abc', glyph: '✦', label: 'Session usage' }))

    expect(html).toContain('class="progress-ring__icon"')
    expect(html).toContain('aria-hidden="true"')
  })

  it.each([
    ['codex', 'Weekly / all models', 'role="progressbar"', 2],
    ['claude', 'Current session across all supported models', 'role="progressbar"', 1],
    ['opencode', 'Current session', 'role="progressbar"', 1],
    ['loading', 'Updating usage', 'role="status"', 0],
    ['unavailable', 'Usage is unavailable right now.', 'role="status"', 0],
    ['error', 'Could not update usage. Try again.', 'role="alert"', 0],
    ['stale', 'Weekly / all models', 'role="progressbar"', 2],
  ])('renders the %s fixture and its state', async (fixtureName, expectedText, expectedRole, ringCount) => {
    const html = await renderToString(createSSRApp(UsagePanel, { provider: usageFixtures[fixtureName] }))

    expect(html).toContain(expectedText)
    expect(html).toContain(expectedRole)
    expect(html.match(/role="progressbar"/g) ?? []).toHaveLength(ringCount)
  })

  it('passes quota values and colors through the complete panel', async () => {
    const provider = {
      ...usageFixtures.codex,
      quotas: [{ ...usageFixtures.codex.quotas[0], percentage: 37, color: '#123abc' }],
    }
    const html = await renderToString(createSSRApp(UsagePanel, { provider }))

    expect(html).toContain('aria-valuenow="37"')
    expect(html).toContain('--ring-color:#eab308')
    expect(html).toContain('>63%</strong>')
    expect(html).toContain('37% used')
  })

  it('falls back to Codex for an unknown fixture', () => {
    expect(getUsageFixture('unknown')).toBe(usageFixtures.codex)
  })

  it('does not turn an unknown percentage into zero', async () => {
    const html = await renderToString(createSSRApp(UsagePanel, { provider: usageFixtures.opencode }))
    expect(html).toContain('>—</strong>')
    expect(html).not.toContain('aria-valuenow="0"')
  })
  it('renders the switch action or cancellation action for Codex sign-in', async () => {
    const ready = await renderToString(createSSRApp(UsagePanel, { provider: usageFixtures.codex, authenticating: false }))
    const authenticating = await renderToString(createSSRApp(UsagePanel, { provider: usageFixtures.codex, authenticating: true }))

    expect(ready).toContain('aria-label="Switch Codex account"')
    expect(authenticating).toContain('aria-label="Cancel Codex sign-in"')
    expect(authenticating).toContain('Signing in')
  })
})
