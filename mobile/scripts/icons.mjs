#!/usr/bin/env node
// Renders every app icon from the web app's `static/icons/icon-maskable.svg` so
// the store listing, the launcher and the splash all show the same pot:
//
//   android/app/src/main/res/mipmap-*/ic_launcher_foreground.webp  adaptive foreground (108 dp)
//   android/app/src/main/res/mipmap-*/ic_launcher{,_round}.webp    legacy icons (48 dp, API < 26)
//   android/app/src/main/res/mipmap-anydpi-v26/*.xml               adaptive icon definitions
//   android/app/src/main/res/values/ic_launcher_background.xml     adaptive background colour
//   ios/.../AppIcon.appiconset/<1024 entry>                        iOS app icon (unverified, no Mac)
//   resource/app_icon.png, resource/splash_icon.png                Sparkling shared resources
//
// Needs ImageMagick 7 (`magick`, with librsvg) — provided by the nix shell.
// Run with `npm run icons` from mobile/.
import { execFileSync } from 'node:child_process'
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const mobile = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const SOURCE = resolve(mobile, '../static/icons/icon-maskable.svg')
const RES = join(mobile, 'android/app/src/main/res')
const IOS_ICONSET = join(mobile, 'ios/SparklingGo/SparklingGo/Assets.xcassets/AppIcon.appiconset')

// `--color-cream` in tailwind.css: the adaptive background and the tile behind the pot.
const BACKGROUND = '#fbf5e9'
// The SVG draws a 1024 content box inside a 1280 canvas (20% maskable safe margin).
const SVG_CANVAS = 1280
const SVG_CONTENT = 1024
const SVG_CORNER = 230
const DENSITIES = { mdpi: 1, hdpi: 1.5, xhdpi: 2, xxhdpi: 3, xxxhdpi: 4 }
const ADAPTIVE_DP = 108
const LEGACY_DP = 48

const source = readFileSync(SOURCE, 'utf8')
const foregroundSvg = source
  // Full-canvas and rounded-square backgrounds: the adaptive background layer draws those.
  .replace(/<rect x="0" y="0"[^>]*\/>\s*/g, '')
  // The wordmark is painted in the background colour, i.e. invisible on the web; drop it.
  .replace(/<text[\s\S]*?<\/text>\s*/g, '')
  // The grater face was the SVG's background colour; keep it on the app's cream.
  .replaceAll('#F5F0E8', BACKGROUND)

function magick(args, input) {
  return execFileSync('magick', args, { input, maxBuffer: 64 << 20 })
}

// The pot with a transparent background, the SVG's 1024 content box scaled to `size`.
// The adaptive foreground layer reserves its own safe zone (66 dp of 108 dp), so the
// SVG's extra 20% margin is cropped away instead of being applied twice.
function renderForeground(size, format) {
  const full = Math.round((size * SVG_CANVAS) / SVG_CONTENT)
  return magick(
    [
      '-background',
      'none',
      'svg:-',
      '-resize',
      `${full}x${full}`,
      '-gravity',
      'center',
      '-crop',
      `${size}x${size}+0+0`,
      '+repage',
      ...output(format),
    ],
    foregroundSvg,
  )
}

// The pot on a cream tile: 'square' is full-bleed (stores mask it themselves),
// 'rounded' and 'round' are the legacy launcher shapes.
function renderTile(size, shape, format) {
  const radius = Math.round((SVG_CORNER / SVG_CONTENT) * size)
  const half = size / 2
  const mask =
    shape === 'round'
      ? `circle ${half},${half} ${half},0`
      : shape === 'rounded'
        ? `roundrectangle 0,0,${size - 1},${size - 1},${radius},${radius}`
        : null
  return magick(
    [
      '-size',
      `${size}x${size}`,
      `xc:${BACKGROUND}`,
      'png:-',
      '-gravity',
      'center',
      '-composite',
      ...(mask
        ? ['(', '-size', `${size}x${size}`, 'xc:none', '-draw', mask, ')', '-compose', 'DstIn', '-composite']
        : []),
      ...output(format),
    ],
    renderForeground(size, 'png'),
  )
}

function output(format) {
  return format === 'webp' ? ['-define', 'webp:lossless=true', 'webp:-'] : ['png:-']
}

function write(path, data) {
  mkdirSync(dirname(path), { recursive: true })
  writeFileSync(path, data)
  console.log(`wrote ${path.replace(`${mobile}/`, '')}`)
}

const adaptiveIcon = `<?xml version="1.0" encoding="utf-8"?>
<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">
    <background android:drawable="@color/ic_launcher_background" />
    <foreground android:drawable="@mipmap/ic_launcher_foreground" />
    <monochrome android:drawable="@mipmap/ic_launcher_foreground" />
</adaptive-icon>
`

for (const [density, scale] of Object.entries(DENSITIES)) {
  const dir = join(RES, `mipmap-${density}`)
  write(join(dir, 'ic_launcher_foreground.webp'), renderForeground(ADAPTIVE_DP * scale, 'webp'))
  write(join(dir, 'ic_launcher.webp'), renderTile(LEGACY_DP * scale, 'rounded', 'webp'))
  write(join(dir, 'ic_launcher_round.webp'), renderTile(LEGACY_DP * scale, 'round', 'webp'))
}
write(join(RES, 'mipmap-anydpi-v26/ic_launcher.xml'), adaptiveIcon)
write(join(RES, 'mipmap-anydpi-v26/ic_launcher_round.xml'), adaptiveIcon)
write(
  join(RES, 'values/ic_launcher_background.xml'),
  `<?xml version="1.0" encoding="utf-8"?>
<resources>
    <color name="ic_launcher_background">${BACKGROUND}</color>
</resources>
`,
)
// The template's vector robot, superseded by the mipmaps above.
for (const name of ['ic_launcher_background.xml', 'ic_launcher_foreground.xml']) {
  const path = join(RES, 'drawable', name)
  if (existsSync(path)) {
    rmSync(path)
    console.log(`removed ${path.replace(`${mobile}/`, '')}`)
  }
}

const square1024 = renderTile(1024, 'square', 'png')
write(join(mobile, 'resource/app_icon.png'), square1024)
write(join(mobile, 'resource/splash_icon.png'), renderTile(512, 'rounded', 'png'))

const contentsPath = join(IOS_ICONSET, 'Contents.json')
if (existsSync(contentsPath)) {
  const contents = JSON.parse(readFileSync(contentsPath, 'utf8'))
  const entry = contents.images.find((image) => image.size === '1024x1024')
  if (entry) {
    entry.filename ??= 'AppIcon.png'
    write(join(IOS_ICONSET, entry.filename), square1024)
    write(contentsPath, `${JSON.stringify(contents, null, 2)}\n`)
  }
}
