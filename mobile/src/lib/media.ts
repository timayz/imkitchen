import { chooseMedia } from 'sparkling-media'

export interface PickedImage {
  mimeType: string
  base64: string
}

/**
 * Lets the user pick one photo (album) and returns it as base64, downsized
 * so the JSON upload stays small. `null` when the user cancelled.
 */
export function pickImage(): Promise<PickedImage | null> {
  return new Promise((resolve, reject) => {
    chooseMedia(
      {
        mediaTypes: ['image'],
        sourceType: 'album',
        maxCount: 1,
        needBase64Data: true,
        compressImage: true,
        compressWidth: 1600,
        compressHeight: 1600,
        compressQuality: 85,
      },
      (res) => {
        if (res.code !== 0 && res.code !== 1) {
          // Negative codes are failures; a cancel usually comes back without files.
          if (res.data?.tempFiles?.length) {
            /* fall through */
          } else if (res.msg && /cancel/i.test(res.msg)) {
            resolve(null)
            return
          } else {
            reject(new Error(res.msg || `chooseMedia failed (${res.code})`))
            return
          }
        }
        const file = res.data?.tempFiles?.[0]
        if (!file) {
          resolve(null)
          return
        }
        if (!file.base64Data) {
          reject(new Error('No image data returned'))
          return
        }
        // Some hosts return a data URL; keep only the payload.
        const base64 = file.base64Data.replace(/^data:[^;]+;base64,/, '')
        const mimeType = file.mimeType && file.mimeType !== '' ? file.mimeType : 'image/jpeg'
        resolve({ mimeType, base64 })
      },
    )
  })
}
