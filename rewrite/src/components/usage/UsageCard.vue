<script setup lang="ts">
import { computed } from 'vue'
import ProgressRing from './ProgressRing.vue'
import { clampPercentage } from './usage'
import type { UsageQuota } from './usageTypes'

const props = defineProps<{ quota: UsageQuota }>()
const percentage = computed(() => clampPercentage(props.quota.percentage))
</script>

<template>
  <article class="usage-card" :aria-labelledby="`${quota.id}-title`">
    <ProgressRing :percentage="quota.percentage" :color="quota.color" :glyph="quota.glyph" :label="`${quota.title}: ${percentage}% used`" />
    <strong class="usage-card__percentage">{{ percentage }}%</strong>
    <h2 :id="`${quota.id}-title`" class="usage-card__title">{{ quota.title }}</h2>
    <p class="usage-card__reset">{{ quota.resetLabel }}</p>
  </article>
</template>

<style scoped>
.usage-card {
  min-width: 0; padding: 18px 10px 16px; border: 1px solid #3b3b42; border-radius: 12px;
  background: #29292e; box-shadow: inset 0 1px #ffffff0a; text-align: center;
}
.usage-card__percentage {
  display: block; margin-top: 12px; color: #fff; font-size: 1.45rem;
  font-variant-numeric: tabular-nums; letter-spacing: -0.035em; line-height: 1;
}
.usage-card__title, .usage-card__reset { overflow-wrap: anywhere; }
.usage-card__title { margin: 9px 0 0; color: #f0f0f2; font-size: 0.81rem; font-weight: 600; line-height: 1.3; }
.usage-card__reset { margin: 6px 0 0; color: #aaaab2; font-size: 0.7rem; line-height: 1.35; }
</style>
