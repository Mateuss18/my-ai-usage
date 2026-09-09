<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue'

import { cancelCodexLogin, getUsage, pollCodexLogin, startCodexLogin } from './bridge'
import { createCodexLogin } from './codexLogin'
import UsagePanel from './components/usage/UsagePanel.vue'
import { createUsageRefresh } from './usageRefresh'

const { provider, refresh, start, stop } = createUsageRefresh(getUsage)
const { state: loginState, begin, cancel } = createCodexLogin({
  start: startCodexLogin,
  poll: pollCodexLogin,
  cancel: cancelCodexLogin,
})
const authenticating = computed(() => loginState.value === 'authenticating' || loginState.value === 'cancelling')

async function switchAccount(): Promise<void> {
  stop()
  try {
    await begin()
  } finally {
    start()
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
      @refresh="refresh"
      @switch-account="switchAccount"
      @cancel-login="cancel"
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
