<script setup lang="ts">
import { computed } from 'vue'
import { clampPercentage } from './usage'
import codexIcon from '../../assets/gpt-icon-black.png'

interface Props { percentage: number | null; color: string; glyph: string; label: string }
const props = defineProps<Props>()

const normalizedPercentage = computed(() => props.percentage === null ? null : clampPercentage(props.percentage))
const ringStyle = computed(() => ({
  '--ring-color': props.color,
  '--ring-value': `${normalizedPercentage.value}%`,
  '--ring-value-number': normalizedPercentage.value ?? 0,
}))
</script>

<template>
  <div class="progress-ring" :class="{ 'progress-ring--unknown': normalizedPercentage === null }" :style="ringStyle" role="progressbar" :aria-label="label" aria-valuemin="0" aria-valuemax="100" :aria-valuenow="normalizedPercentage ?? undefined">
    <svg class="progress-ring__svg" viewBox="0 0 62 62" aria-hidden="true">
      <circle class="progress-ring__track" cx="31" cy="31" r="26" />
      <circle class="progress-ring__progress" cx="31" cy="31" r="26" />
    </svg>
    <img v-if="glyph === '✦'" class="progress-ring__icon" :src="codexIcon" alt="" aria-hidden="true" />
    <span v-else class="progress-ring__glyph" aria-hidden="true">{{ glyph }}</span>
  </div>
</template>

<style scoped>
.progress-ring {
  --ring-track: color-mix(in srgb, var(--app-blue) 28%, var(--app-ink));
  display: grid; position: relative; width: 62px; aspect-ratio: 1; margin-inline: auto; flex: 0 0 auto; place-items: center;
  border-radius: 50%;
  animation: ring-in 420ms cubic-bezier(0.16, 1, 0.3, 1) both;
}
.progress-ring::before {
  z-index: 1; grid-area: 1 / 1; width: 48px; aspect-ratio: 1; border: 1px solid color-mix(in srgb, var(--app-purple) 32%, var(--app-ink));
  border-radius: 50%; background: var(--app-ink); content: '';
}
.progress-ring__svg { z-index: 0; grid-area: 1 / 1; width: 100%; height: 100%; overflow: visible; transform: rotate(-90deg); }
.progress-ring__track, .progress-ring__progress { fill: none; stroke-width: 5; }
.progress-ring__track { stroke: var(--ring-track); }
.progress-ring__progress {
  stroke: var(--ring-color); stroke-dasharray: 163.36; stroke-dashoffset: calc(163.36 * (1 - var(--ring-value-number) / 100));
  stroke-linecap: round; transition: stroke-dashoffset 260ms ease, stroke 160ms ease;
}
.progress-ring--unknown .progress-ring__progress { display: none; }
.progress-ring__icon { z-index: 2; grid-area: 1 / 1; width: 30px; height: 30px; object-fit: contain; }
.progress-ring__glyph { z-index: 2; grid-area: 1 / 1; color: var(--app-white); font-size: 1.25rem; font-weight: 600; line-height: 1; }
@keyframes ring-in { from { opacity: 0; transform: scale(0.92) rotate(-8deg); } }
@media (prefers-reduced-motion: reduce) { .progress-ring, .progress-ring__progress { animation: none; transition: none; } }
</style>
