import { storageGet, storageSet } from './storage.js'

const KEY = 'client.install_id'
let cached: string | null = null

/**
 * The string this install sends as `User-Agent`. The server stores it on the
 * login record and matches it on every request (it is the device identity),
 * so it must never change for the lifetime of an install:
 *
 *   imkitchen-android (Android; pixel 8; 01J9…)
 *
 * No app version, no OS version: either would log the user out on update.
 * The app version travels separately in `X-Client-Version`.
 */
export async function clientIdentity(): Promise<string> {
  if (cached) return cached

  let installId = await storageGet<string>(KEY)
  if (!installId) {
    installId = newInstallId()
    await storageSet(KEY, installId)
  }

  const props = lynx.__globalProps
  const os = props.os === 'ios' ? 'iOS' : 'Android'
  const model = (props.deviceModel || 'unknown').replace(/[();]/g, ' ').trim()

  cached = `imkitchen-${os.toLowerCase()} (${os}; ${model}; ${installId})`
  return cached
}

/** 26 chars of Crockford base32: 10 of time, 16 of randomness (ULID shape). */
function newInstallId(): string {
  const alphabet = '0123456789ABCDEFGHJKMNPQRSTVWXYZ'
  let time = Date.now()
  let out = ''
  for (let i = 0; i < 10; i++) {
    out = alphabet[time % 32] + out
    time = Math.floor(time / 32)
  }
  for (let i = 0; i < 16; i++) {
    out += alphabet[Math.floor(Math.random() * 32)]
  }
  return out
}
