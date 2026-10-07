import { useCallback, useEffect, useRef, useState } from '@lynx-js/react'

import { errorMessage, isOffline } from './api/client.js'
import { peekDoc, readDoc, revalidate, subscribeDoc } from './cache.js'
import { installQueue } from './offline/queue.js'

// Every bundle that reads the cache must also fold the queued changes into
// what the server returns.
installQueue()

export interface Resource<T> {
  data: T | null
  /** Nothing to show yet and a fetch is in flight. */
  loading: boolean
  /** `data` comes from the local store; the refresh has not landed. */
  stale: boolean
  /** The last refresh failed to reach the server. */
  offline: boolean
  /** The last refresh failed; screens show it when there is no `data`. */
  error: string | null
  refresh: () => void
}

interface State<T> {
  data: T | null
  loading: boolean
  stale: boolean
  offline: boolean
  error: string | null
}

/**
 * Stale-while-revalidate over the local store: the cached document (if any)
 * renders at once, the fetch runs in the background and replaces it. With a
 * `null` key the resource is network-only (searches, uncached screens).
 * Re-runs when `key`, `deps` or `refresh()` change.
 */
export function useResource<T>(key: string | null, fetcher: () => Promise<T>, deps: unknown[] = []): Resource<T> {
  // A document this runtime has already seen renders on the first frame.
  const [state, setState] = useState<State<T>>(() => {
    const seen = key !== null ? peekDoc<T>(key) : null
    return { data: seen, loading: seen === null, stale: seen !== null, offline: false, error: null }
  })
  const [tick, setTick] = useState(0)
  const generation = useRef(0)

  const refresh = useCallback(() => setTick((n) => n + 1), [])

  useEffect(() => {
    const current = ++generation.current
    let fresh = false
    setState((s) => ({ ...s, loading: s.data === null, error: null }))

    if (key !== null) {
      readDoc<T>(key).then((doc) => {
        if (generation.current !== current || fresh || doc === null) return
        setState((s) => ({ ...s, data: doc, loading: false, stale: true }))
      })
    }

    const run = key !== null ? revalidate(key, fetcher) : fetcher()
    run
      .then((data) => {
        if (generation.current !== current) return
        fresh = true
        setState({ data, loading: false, stale: false, offline: false, error: null })
      })
      .catch((err: unknown) => {
        if (generation.current !== current) return
        setState((s) => ({ ...s, loading: false, offline: isOffline(err), error: errorMessage(err) }))
      })
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [key, tick, ...deps])

  // Optimistic patches from the offline queue land here.
  useEffect(() => {
    if (key === null) return
    return subscribeDoc<T>(key, (doc) => setState((s) => ({ ...s, data: doc, loading: false })))
  }, [key])

  return { ...state, refresh }
}
