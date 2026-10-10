import * as sparklingMethod from 'sparkling-method'

interface Pipe {
  call(method: string, params: unknown, callback: (v: unknown) => void): void
}

// `sparkling-method` is typed as CommonJS, so its default export is reachable
// either as the namespace itself (bundler interop) or as `.default`.
const pipe: Pipe = (sparklingMethod as unknown as { default?: Pipe }).default ?? (sparklingMethod as unknown as Pipe)

export interface ScheduleRequest {
  /** Replaces any pending alarm with the same id. */
  id: string
  /** Epoch milliseconds. */
  at: number
  title: string
  body: string
}

export interface ScheduleResponse {
  /** 1 = ok, anything else failed (see `msg`). */
  code: number
  msg: string
  scheduled: boolean
  /** Android: `false` when the OS only allows an inexact (possibly late) alarm. */
  exact: boolean
  /** Whether the user allows notifications; the alarm is set regardless. */
  notifications: boolean
}

export interface CancelRequest {
  id: string
}

export interface CancelResponse {
  code: number
  msg: string
  ok: boolean
}

/** Raw bridge answer: `code` 1 = ok, anything else failed (`msg`); result fields come back under `data`. */
function unwrap<T>(v: unknown): { code: number; msg: string; data: Partial<T> } {
  const response = v as { code?: number; msg?: string; data?: Partial<T> } | null
  const code = response?.code ?? -1
  return { code, msg: response?.msg ?? (code === 1 ? 'ok' : 'Unknown error'), data: response?.data ?? {} }
}

/** Ring at `at` with `title`/`body`, through the OS so it survives sleep and process death. */
export function schedule(params: ScheduleRequest, callback: (result: ScheduleResponse) => void): void {
  const payload = {
    id: String(params.id),
    at: Math.round(params.at),
    title: String(params.title),
    body: String(params.body),
  }
  pipe.call('TimerAlarm.schedule', payload, (v: unknown) => {
    const { code, msg, data } = unwrap<ScheduleResponse>(v)
    callback({
      code,
      msg,
      scheduled: data.scheduled === true,
      exact: data.exact === true,
      notifications: data.notifications === true,
    })
  })
}

/** Drop the pending alarm `id` (no-op when there is none). */
export function cancel(params: CancelRequest, callback: (result: CancelResponse) => void): void {
  pipe.call('TimerAlarm.cancel', { id: String(params.id) }, (v: unknown) => {
    const { code, msg, data } = unwrap<CancelResponse>(v)
    callback({ code, msg, ok: data.ok === true })
  })
}
