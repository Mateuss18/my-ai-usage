<script setup lang="ts">
import { computed, onMounted, onUnmounted, shallowRef } from 'vue'
import { isTauri } from '@tauri-apps/api/core'

import { cancelCodexLogin, deleteAccount, getUsage, logoutCodex, pollCodexLogin, resizeUsagePanel, startCodexLogin } from './bridge'
import { createCodexLogin } from './codexLogin'
import UsagePanel from './components/usage/UsagePanel.vue'
import { createUsageRefresh } from './usageRefresh'

const { accounts, loading, error, refresh, reset, start, stop } = createUsageRefresh(getUsage)
const loginProfileId = shallowRef<string>()
const { state: loginState, begin, cancel } = createCodexLogin({
  start: () => startCodexLogin(loginProfileId.value),
  poll: pollCodexLogin,
  cancel: cancelCodexLogin,
})
const logoutFailed = shallowRef(false)
const logoutting = shallowRef(false)
const deleting = shallowRef(false)
const deleteFailed = shallowRef(false)
const authenticating = computed(() => ['starting', 'waiting', 'cancelling'].includes(loginState.value))
const authenticated = computed(() => accounts.value.some(account => account.accountStatus === 'Active'))
const loginFailed = computed(() => loginState.value === 'error')

async function switchAccount(profileId?: string): Promise<void> {
  loginProfileId.value = profileId
  stop()
  logoutFailed.value = false
  try {
    await begin()
  } finally {
    start()
  }
}

async function logout(profileId?: string): Promise<void> {
  logoutting.value = true
  stop()
  logoutFailed.value = false
  try {
    await logoutCodex(profileId)
    reset()
  } catch {
    logoutFailed.value = true
    start()
    await refresh()
  } finally {
    logoutting.value = false
    start()
  }
}

async function removeAccount(key: string): Promise<void> {
  deleting.value = true
  deleteFailed.value = false
  stop()
  try {
    const account = accounts.value.find(account => account.account.key === key)
    await deleteAccount(key, account?.accountStatus === 'Active', account?.profileId ?? undefined)
    reset()
  } catch {
    deleteFailed.value = true
  } finally {
    deleting.value = false
    start()
  }
}

let panelObserver: ResizeObserver | undefined

function fitPanel(): void {
  const panel = document.querySelector<HTMLElement>('.usage-panel')
  if (isTauri() && panel) void resizeUsagePanel(Math.ceil(Number.parseFloat(getComputedStyle(panel).height))).catch(() => console.warn('Could not resize usage panel.'))
}

onMounted(() => {
  start()
  if (!isTauri()) return
  const panel = document.querySelector<HTMLElement>('.usage-panel')
  if (panel) {
    panelObserver = new ResizeObserver(entries => {
      const height = entries[0]?.borderBoxSize[0]?.blockSize
      if (height) void resizeUsagePanel(Math.ceil(height)).catch(() => console.warn('Could not resize usage panel.'))
    })
    panelObserver.observe(panel)
  }
  window.addEventListener('focus', fitPanel)
})
onUnmounted(() => {
  panelObserver?.disconnect()
  window.removeEventListener('focus', fitPanel)
  stop()
  void cancel()
})
</script>

<template>
  <main class="app-shell">
    <UsagePanel
      :accounts="accounts"
      :loading="loading"
      :error="error"
      :authenticating="authenticating"
      :authenticated="authenticated"
      :login-failed="loginFailed"
      :logout-failed="logoutFailed"
      :logoutting="logoutting"
      :deleting="deleting"
      :delete-failed="deleteFailed"
      @refresh="refresh"
      @switch-account="switchAccount"
      @cancel-login="cancel"
      @logout="logout"
      @delete-account="removeAccount"
    />
  </main>
</template>

<style scoped>
:global(:root) {
  --app-ink: #05081A;
  --app-surface-1: #0d1120;
  --app-surface-2: #141a33;
  --app-surface-3: #1b2244;
  --app-border: rgba(255, 255, 255, 0.07);
  --app-border-strong: rgba(255, 255, 255, 0.14);
  --app-text-secondary: #b2badc;
  --app-text-subtle: #9099bd;
  --app-blue: #2678FD;
  --app-purple: #8974FD;
  --app-white: #FAFAFA;
}
:global(*) { box-sizing: border-box; }
:global(html), :global(body), :global(#app) { width: 100%; height: 100%; min-width: 0; min-height: 100%; margin: 0; overflow: hidden; }
:global(body) {
  background: var(--app-ink);
  color: var(--app-white);
  font-family: "Segoe UI Variable Text", "Segoe UI", system-ui, sans-serif;
  -webkit-font-smoothing: antialiased;
}
:global(button), :global(summary) { font: inherit; }
.app-shell { display: grid; width: 100%; height: 100vh; overflow: hidden; place-items: start center; }
</style>
