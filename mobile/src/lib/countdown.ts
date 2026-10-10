/**
 * Wall-clock countdown arithmetic for the cooking timer. The timer keeps a
 * deadline (epoch ms) rather than counting ticks, so a throttled or suspended
 * JS thread shows the right time as soon as it runs again.
 */

/** Whole seconds left until `deadline`, never negative; a partial second still counts. */
export function remainingSeconds(deadline: number, now: number): number {
  return Math.max(0, Math.ceil((deadline - now) / 1000))
}

/** `mm:ss`, minutes growing past 59 (a 90 minute braise reads 90:00). */
export function formatClock(seconds: number): string {
  const s = Math.max(0, Math.floor(seconds))
  const mm = String(Math.floor(s / 60)).padStart(2, '0')
  const ss = String(s % 60).padStart(2, '0')
  return `${mm}:${ss}`
}
