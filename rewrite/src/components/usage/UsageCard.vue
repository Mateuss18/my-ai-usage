<script setup lang="ts">
import { computed } from 'vue'
import ProgressRing from './ProgressRing.vue'
import { clampPercentage } from './usage'
import type { UsageQuota } from './usageTypes'

const props = defineProps<{ quota: UsageQuota }>()
const usedPercentage = computed(() => props.quota.percentage === null ? null : clampPercentage(props.quota.percentage))
const percentage = computed(() => usedPercentage.value === null ? null : 100 - usedPercentage.value)
const progressColor = computed(() => {
  if (usedPercentage.value === null) return props.quota.color
  if (usedPercentage.value <= 30) return '#22c55e'
  if (usedPercentage.value <= 70) return '#eab308'
  return '#ef4444'
})
</script>

<template>
  <article class="usage-card" :aria-labelledby="`${quota.id}-title`">
    <ProgressRing :percentage="usedPercentage" :color="progressColor" :glyph="quota.glyph" :label="`${quota.title}: ${usedPercentage === null ? 'unknown' : `${usedPercentage}% used`}`" />
    <strong class="usage-card__percentage">{{ percentage === null ? '—' : `${percentage}%` }}</strong>
    <h2 :id="`${quota.id}-title`" class="usage-card__title">{{ quota.title }}</h2>
    <p class="usage-card__reset">{{ quota.resetLabel }}</p>
  </article>
</template>

<style scoped>
.usage-card {
  min-width: 0; padding: 8px 10px 6px; text-align: center;
}
.usage-card__percentage {
  display: block; margin-top: 8px; color: #fff; font-size: 1.1rem;
  font-variant-numeric: tabular-nums; letter-spacing: -0.035em; line-height: 1;
}
.usage-card__title, .usage-card__reset { overflow-wrap: anywhere; }
.usage-card__title { margin: 6px 0 0; color: #f0f0f2; font-size: 0.81rem; font-weight: 600; line-height: 1.3; }
.usage-card__reset { margin: 4px 0 0; color: #aaaab2; font-size: 0.7rem; line-height: 1.35; }
</style>
