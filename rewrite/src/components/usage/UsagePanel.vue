<script setup lang="ts">
import ProviderHeader from './ProviderHeader.vue'
import UsageCard from './UsageCard.vue'
import type { ProviderUsage } from './usageTypes'

defineProps<{ provider: ProviderUsage }>()
const emit = defineEmits<{ refresh: [] }>()
</script>

<template>
  <section class="usage-panel" :aria-busy="provider.state === 'loading'">
    <ProviderHeader :name="provider.name" :eyebrow="provider.eyebrow" :glyph="provider.glyph" @refresh="emit('refresh')" />

    <div v-if="provider.state === 'loading'" class="usage-panel__loading" role="status">
      <span class="sr-only">{{ provider.statusLabel }}</span>
      <div v-for="index in 2" :key="index" class="usage-panel__skeleton" aria-hidden="true">
        <span class="usage-panel__skeleton-ring"></span>
        <span class="usage-panel__skeleton-line"></span>
        <span class="usage-panel__skeleton-line usage-panel__skeleton-line--short"></span>
      </div>
    </div>

    <div
      v-else-if="provider.state === 'error' || provider.state === 'unavailable'"
      class="usage-panel__empty"
      :class="{ 'usage-panel__empty--error': provider.state === 'error' }"
      :role="provider.state === 'error' ? 'alert' : 'status'"
    >
      <span class="usage-panel__empty-icon" aria-hidden="true">{{ provider.state === 'error' ? '!' : '—' }}</span>
      <p>{{ provider.statusLabel }}</p>
      <button v-if="provider.state === 'error'" type="button" @click="emit('refresh')">Try again</button>
    </div>

    <div v-else class="usage-panel__grid">
      <UsageCard v-for="quota in provider.quotas" :key="quota.id" :quota="quota" />
    </div>

    <footer class="usage-panel__footer">
      <span class="usage-panel__status">
        <span class="usage-panel__status-dot" :class="`is-${provider.state}`" aria-hidden="true"></span>
        {{ provider.statusLabel }}
      </span>
      <span class="usage-panel__provider">{{ provider.name }}</span>
    </footer>
  </section>
</template>

<style scoped>
.usage-panel {
  width: min(100%, 396px); max-width: 100%; overflow: hidden; border: 1px solid #3d3d43;
  border-radius: 16px; background: #202024; box-shadow: 0 20px 54px #0008, inset 0 1px #ffffff10;
  animation: panel-in 180ms cubic-bezier(0.16, 1, 0.3, 1) both;
}
.usage-panel__grid, .usage-panel__loading { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 10px; padding: 14px; }
.usage-panel__empty {
  display: grid; min-height: 268px; padding: 32px; place-items: center; align-content: center;
  gap: 12px; color: #c5c5cc; text-align: center;
}
.usage-panel__empty--error { color: #fecaca; }
.usage-panel__empty-icon { display: grid; width: 42px; height: 42px; place-items: center; border: 1px solid currentColor; border-radius: 50%; font-size: 1.15rem; opacity: 0.8; }
.usage-panel__empty p { max-width: 240px; margin: 0; overflow-wrap: anywhere; font-size: 0.84rem; line-height: 1.5; }
.usage-panel__empty button { padding: 7px 12px; border: 1px solid #56565f; border-radius: 7px; background: #35353b; color: #fff; cursor: pointer; }
.usage-panel__empty button:focus-visible { outline: 2px solid #7dd3fc; outline-offset: 2px; }
.usage-panel__skeleton {
  display: grid; min-width: 0; min-height: 244px; padding: 22px 12px; place-items: center;
  align-content: center; gap: 14px; border: 1px solid #36363c; border-radius: 12px; background: #29292e;
}
.usage-panel__skeleton-ring, .usage-panel__skeleton-line {
  background: linear-gradient(90deg, #34343a 25%, #414148 50%, #34343a 75%);
  background-size: 200% 100%; animation: shimmer 1.3s linear infinite;
}
.usage-panel__skeleton-ring { width: 104px; aspect-ratio: 1; border-radius: 50%; }
.usage-panel__skeleton-line { width: 82%; height: 9px; border-radius: 999px; }
.usage-panel__skeleton-line--short { width: 56%; }
.usage-panel__footer {
  display: flex; min-width: 0; align-items: center; justify-content: space-between; gap: 12px;
  padding: 11px 15px 12px; border-top: 1px solid #34343a; color: #92929b; font-size: 0.68rem;
}
.usage-panel__status { display: flex; min-width: 0; flex: 1 1 auto; align-items: center; gap: 7px; overflow-wrap: anywhere; }
.usage-panel__status-dot { width: 6px; height: 6px; flex: 0 0 auto; border-radius: 50%; background: #6ee7b7; box-shadow: 0 0 0 3px #6ee7b719; }
.usage-panel__status-dot.is-loading, .usage-panel__status-dot.is-stale { background: #fbbf24; box-shadow: 0 0 0 3px #fbbf2419; }
.usage-panel__status-dot.is-unavailable { background: #a1a1aa; box-shadow: 0 0 0 3px #a1a1aa19; }
.usage-panel__status-dot.is-error { background: #f87171; box-shadow: 0 0 0 3px #f8717119; }
.usage-panel__provider { min-width: 0; flex: 0 1 auto; color: #b6b6bd; font-weight: 600; overflow-wrap: anywhere; text-align: right; }
.sr-only { position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0, 0, 0, 0); clip-path: inset(50%); white-space: nowrap; }
@keyframes panel-in { from { opacity: 0; transform: translateY(5px) scale(0.985); } }
@keyframes shimmer { to { background-position: -200% 0; } }
@media (max-width: 330px) { .usage-panel__grid, .usage-panel__loading { grid-template-columns: 1fr; } }
@media (prefers-reduced-motion: reduce) { .usage-panel, .usage-panel__skeleton-ring, .usage-panel__skeleton-line { animation: none; } }
</style>
