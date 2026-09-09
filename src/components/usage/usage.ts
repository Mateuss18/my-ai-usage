export function clampPercentage(value: number): number {
  return Number.isFinite(value) ? Math.min(100, Math.max(0, value)) : 0
}
