<script setup lang="ts">
import UsagePanel from './components/usage/UsagePanel.vue'
import { getUsageFixture } from './components/usage/usageFixtures'

const fixture = getUsageFixture(new globalThis.URLSearchParams(globalThis.location.search).get('fixture'))

function refresh(): void {
  globalThis.location.reload()
}
</script>

<template>
  <main class="app-shell">
    <UsagePanel :provider="fixture" @refresh="refresh" />
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
