<script setup lang="ts">
import { computed, onMounted, onUnmounted, shallowRef } from 'vue'

import { cancelCodexLogin, getUsage, logoutCodex, pollCodexLogin, startCodexLogin } from './bridge'
import { createCodexLogin } from './codexLogin'
import UsagePanel from './components/usage/UsagePanel.vue'
import { createUsageRefresh } from './usageRefresh'

const { provider, refresh, start, stop } = createUsageRefresh(getUsage)
const { state: loginState, begin, cancel } = createCodexLogin({
  start: startCodexLogin,
  poll: pollCodexLogin,
  cancel: cancelCodexLogin,
})
const logoutFailed = shallowRef(false)
const logoutting = shallowRef(false)
const authenticating = computed(() => ['starting', 'waiting', 'cancelling'].includes(loginState.value))
const authenticated = computed(() => ['available', 'partial', 'stale', 'error'].includes(provider.value.state))
const loginFailed = computed(() => loginState.value === 'error')

async function switchAccount(): Promise<void> {
  stop()
  logoutFailed.value = false
  try {
    await begin()
  } finally {
    start()
  }
}

async function logout(): Promise<void> {
  logoutting.value = true
  stop()
  logoutFailed.value = false
  try {
    await logoutCodex()
  } catch {
    logoutFailed.value = true
  } finally {
    start()
    await refresh()
    logoutting.value = false
  }
}

onMounted(start)
onUnmounted(() => {
  stop()
  void cancel()
})
</script>

<template>
  <main class="app-shell">
    <UsagePanel
      :provider="provider"
      :authenticating="authenticating"
      :authenticated="authenticated"
      :login-failed="loginFailed"
      :logout-failed="logoutFailed"
      :logoutting="logoutting"
      @refresh="refresh"
      @switch-account="switchAccount"
      @cancel-login="cancel"
      @logout="logout"
    />
  </main>
</template>

<style scoped>
:global(*) { box-sizing: border-box; }
:global(html), :global(body), :global(#app) { width: 100%; height: 100%; min-width: 0; min-height: 100%; margin: 0; overflow: hidden; }
:global(body) {
  background: #090909;
  color: #f5f5f5;
  font-family: "Segoe UI Variable Text", "Segoe UI", system-ui, sans-serif;
  -webkit-font-smoothing: antialiased;
}
:global(button), :global(summary) { font: inherit; }
.app-shell { display: grid; width: 100%; height: 100vh; overflow: hidden; place-items: start center; }
</style>
