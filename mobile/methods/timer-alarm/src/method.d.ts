/** Ring a cooking timer even when the app is asleep (AlarmManager / UNUserNotificationCenter). */
export interface ScheduleRequest {
  /** Replaces any pending alarm with the same id. */
  id: string;
  /** Epoch milliseconds. */
  at: number;
  title: string;
  body: string;
}

export interface ScheduleResponse {
  scheduled: boolean;
  /** Android: `false` when the OS only allows an inexact (possibly late) alarm. */
  exact: boolean;
  /** Whether the user allows notifications; the alarm is set regardless. */
  notifications: boolean;
}

export interface CancelRequest {
  id: string;
}

export interface CancelResponse {
  ok: boolean;
}

declare function schedule(params: ScheduleRequest, callback: (result: ScheduleResponse) => void): void;
declare function cancel(params: CancelRequest, callback: (result: CancelResponse) => void): void;
