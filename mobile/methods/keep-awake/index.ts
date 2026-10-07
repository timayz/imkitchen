import * as sparklingMethod from 'sparkling-method'

interface Pipe {
  call(method: string, params: unknown, callback: (v: unknown) => void): void
}

// `sparkling-method` is typed as CommonJS, so its default export is reachable
// either as the namespace itself (bundler interop) or as `.default`.
const pipe: Pipe =
  (sparklingMethod as unknown as { default?: Pipe }).default ?? (sparklingMethod as unknown as Pipe)

export interface SetEnabledRequest {
  enabled: boolean
}

export interface SetEnabledResponse {
  /** 1 = ok, anything else failed (see `msg`). */
  code: number
  msg: string
}

/** Keep the screen on (`true`) or let it sleep again (`false`). */
export function setEnabled(
  params: SetEnabledRequest,
  callback: (result: SetEnabledResponse) => void,
): void {
  pipe.call('KeepAwake.setEnabled', { enabled: params.enabled === true }, (v: unknown) => {
    const response = v as Partial<SetEnabledResponse> | null
    const code = response?.code ?? -1
    callback({ code, msg: response?.msg ?? (code === 1 ? 'ok' : 'Unknown error') })
  })
}
