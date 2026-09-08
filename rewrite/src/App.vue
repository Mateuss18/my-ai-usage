<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'

import { getUsage } from './bridge'
import UsagePanel from './components/usage/UsagePanel.vue'
import { createUsageRefresh } from './usageRefresh'

const { provider, refresh, start, stop } = createUsageRefresh(getUsage)

onMounted(start)
onUnmounted(stop)
</script>

<template>
  <main class="app-shell">
    <UsagePanel :provider="provider" @refresh="refresh" />
  </main>
</template>

<style scoped>
:global(*) { box-sizing: border-box; }
:global(html), :global(body), :global(#app) { min-width: 0; min-height: 100%; margin: 0; }
:global(body) {
  overflow-x: hidden;
  background: #171717;
  color: #f5f5f5;
  font-family: "Segoe UI Variable Text", "Segoe UI", system-ui, sans-serif;
  -webkit-font-smoothing: antialiased;
}
:global(button), :global(summary) { font: inherit; }
.app-shell { display: grid; min-height: 100vh; place-items: start center; padding: 2px; }
</style>
