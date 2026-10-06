import { getItem, setItem } from 'sparkling-storage'

/** Namespace inside the native key/value store (SharedPreferences / UserDefaults). */
const BIZ = 'imkitchen'

export function storageGet<T extends string | number | boolean>(key: string): Promise<T | null> {
  return new Promise((resolve) => {
    getItem({ key, biz: BIZ }, (res) => {
      const value = res.code === 1 ? (res.data?.data as T | undefined) : undefined
      // '' is how `storageRemove` clears a key (no remove in the TS API).
      resolve(value === undefined || value === null || value === '' ? null : value)
    })
  })
}

export function storageSet(key: string, data: string | number | boolean): Promise<void> {
  return new Promise((resolve, reject) => {
    setItem({ key, data, biz: BIZ }, (res) => {
      if (res.code === 1) resolve()
      else reject(new Error(`storage.setItem ${key}: ${res.msg}`))
    })
  })
}

export function storageRemove(key: string): Promise<void> {
  return storageSet(key, '')
}
