/**
 * "Offline" as the app knows it: the last request failed to reach the
 * server. There is no connectivity API in the Lynx runtime, so the flag can
 * lag one request behind reality; a retry corrects it.
 */
let offline = false
const listeners = new Set<(offline: boolean) => void>()

export function isOfflineNow(): boolean {
  return offline
}

export function setOffline(next: boolean): void {
  if (offline === next) return
  offline = next
  listeners.forEach((listener) => listener(next))
}

export function subscribeOffline(listener: (offline: boolean) => void): () => void {
  listeners.add(listener)
  return () => {
    listeners.delete(listener)
  }
}
