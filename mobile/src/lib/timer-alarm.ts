import { cancel, schedule } from 'sparkling-timer-alarm'

/**
 * The cooking timer's ring, through the OS (host method in
 * `methods/timer-alarm`): an exact alarm + notification on Android, a local
 * notification on iOS. Either survives the screen turning off, the app going
 * to the background and the process being killed, which the JS countdown in
 * the cooking screen does not. Failures are ignored: the countdown display
 * still works and the user is looking at it most of the time anyway.
 */

export interface TimerAlarm {
  /** Stable per recipe + step; scheduling it again moves the alarm. */
  id: string
  /** Epoch milliseconds. */
  at: number
  title: string
  body: string
}

export function scheduleTimerAlarm(alarm: TimerAlarm): void {
  try {
    schedule(alarm, () => {})
  } catch {
    // Host without the method (e.g. tests): nothing to do.
  }
}

export function cancelTimerAlarm(id: string): void {
  try {
    cancel({ id }, () => {})
  } catch {
    // Host without the method (e.g. tests): nothing to do.
  }
}
