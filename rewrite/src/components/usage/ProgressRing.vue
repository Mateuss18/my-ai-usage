<script setup lang="ts">
import { computed } from 'vue'
import { clampPercentage } from './usage'

interface Props { percentage: number | null; color: string; glyph: string; label: string }
const props = defineProps<Props>()

const normalizedPercentage = computed(() => props.percentage === null ? null : clampPercentage(props.percentage))
const ringStyle = computed(() => ({ '--ring-color': props.color, '--ring-value': `${normalizedPercentage.value}%` }))
</script>

<template>
  <div class="progress-ring" :class="{ 'progress-ring--unknown': normalizedPercentage === null }" :style="ringStyle" role="progressbar" :aria-label="label" aria-valuemin="0" aria-valuemax="100" :aria-valuenow="normalizedPercentage ?? undefined">
    <span class="progress-ring__glyph" aria-hidden="true">{{ glyph }}</span>
  </div>
</template>

<style scoped>
.progress-ring {
  --ring-track: #35353b;
  display: grid; width: 104px; aspect-ratio: 1; flex: 0 0 auto; place-items: center;
  border-radius: 50%;
  background: conic-gradient(var(--ring-color) var(--ring-value), var(--ring-track) 0);
  box-shadow: inset 0 0 0 1px #ffffff0a;
  animation: ring-in 420ms cubic-bezier(0.16, 1, 0.3, 1) both;
}
.progress-ring::before {
  grid-area: 1 / 1; width: 82px; aspect-ratio: 1; border: 1px solid #ffffff0d;
  border-radius: 50%; background: #252529; content: '';
}
.progress-ring__glyph { z-index: 1; grid-area: 1 / 1; color: #f7f7f8; font-size: 2rem; font-weight: 600; line-height: 1; }
@keyframes ring-in { from { opacity: 0; transform: scale(0.92) rotate(-8deg); } }
@media (prefers-reduced-motion: reduce) { .progress-ring { animation: none; } }
</style>
