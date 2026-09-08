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

  it.each([
    ['codex', 'Weekly / all models', 'role="progressbar"', 2],
    ['claude', 'Current session across all supported models', 'role="progressbar"', 1],
    ['loading', 'Updating usage', 'role="status"', 0],
    ['unavailable', 'Usage is unavailable right now.', 'role="status"', 0],
    ['error', 'Could not update usage. Try again.', 'role="alert"', 0],
    ['stale', 'Last updated 18 minutes ago', 'role="progressbar"', 2],
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
    expect(html).toContain('--ring-color:#123abc')
    expect(html).toContain('>37%</strong>')
  })

  it('falls back to Codex for an unknown fixture', () => {
    expect(getUsageFixture('unknown')).toBe(usageFixtures.codex)
  })
})
