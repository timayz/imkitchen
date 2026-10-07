#!/usr/bin/env node
// Rasterises the bottom tab bar icons from the web app's navigation
// (`templates/_user.html`, Heroicons outline, 24 dp, 2 px stroke) so the native
// tab bar shows the same glyphs. Lynx `<image>` has no SVG decoder on Android
// (Fresco), so each icon is a transparent PNG recoloured at runtime through
// `tint-color`; the stroke colour below is irrelevant.
//
//   src/assets/icons/<name>.png   24 dp at 4x (xxxhdpi), inlined by rspeedy
//
// Needs ImageMagick 7 (`magick`, with librsvg) — provided by the nix shell.
// Run with `npm run icons` from mobile/.
import { execFileSync } from 'node:child_process'
import { writeFileSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const mobile = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const OUT = join(mobile, 'src/assets/icons')
const DP = 24
const SCALE = 4

// Path data copied verbatim from the `<nav>` in templates/_user.html.
const ICONS = {
  kitchen: [
    'M6 18h12v3a1 1 0 01-1 1H7a1 1 0 01-1-1v-3z',
    'M6 18C4 18 2 15 2 12a6 6 0 016-6',
    'M18 18c2 0 4-3 4-6a6 6 0 00-6-6',
    'M8 6C8 2 16 2 16 6',
  ],
  recipes: [
    'M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253',
  ],
  groceries: [
    'M3 3h2l.4 2M7 13h10l4-8H5.4M7 13L5.4 5M7 13l-2.293 2.293c-.63.63-.184 1.707.707 1.707H17m0 0a2 2 0 100 4 2 2 0 000-4zm-8 2a2 2 0 11-4 0 2 2 0 014 0z',
  ],
  settings: [
    'M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z',
    'M15 12a3 3 0 11-6 0 3 3 0 016 0z',
  ],
}

function svg(paths) {
  const size = DP * SCALE
  const body = paths.map((d) => `<path d="${d}"/>`).join('')
  return (
    `<svg xmlns="http://www.w3.org/2000/svg" width="${size}" height="${size}" viewBox="0 0 24 24" ` +
    `fill="none" stroke="#000" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">${body}</svg>`
  )
}

for (const [name, paths] of Object.entries(ICONS)) {
  const png = execFileSync('magick', ['-background', 'none', 'svg:-', '-strip', 'png32:-'], {
    input: svg(paths),
  })
  writeFileSync(join(OUT, `${name}.png`), png)
  console.log(`${name}.png ${png.length} bytes`)
}
