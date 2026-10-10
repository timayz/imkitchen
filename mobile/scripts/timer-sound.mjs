// Renders the cooking timer's ring (`npm run sound`): a small bell struck
// three times, twice over, like a mechanical kitchen timer. Written once as
// 16-bit mono WAV for both platforms (Android `res/raw`, iOS bundle sound;
// Android loops it while the notification is insistent, iOS plays it once).
import { mkdirSync, writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = join(dirname(fileURLToPath(import.meta.url)), '..', 'methods', 'timer-alarm')
const outputs = [
  join(root, 'android', 'src', 'main', 'res', 'raw', 'cooking_timer.wav'),
  join(root, 'ios', 'Sounds', 'cooking_timer.wav'),
]

const rate = 44100
const length = 3.6
const strikes = [0, 0.28, 0.56, 1.7, 1.98, 2.26]
// A bright little bell: inharmonic partials over a 1760 Hz fundamental.
const partials = [
  { ratio: 1, gain: 1, decay: 0.7 },
  { ratio: 2.0, gain: 0.55, decay: 0.45 },
  { ratio: 2.74, gain: 0.3, decay: 0.3 },
  { ratio: 3.76, gain: 0.18, decay: 0.2 },
  { ratio: 5.4, gain: 0.08, decay: 0.12 },
]
const f0 = 1760

const samples = new Float64Array(Math.round(rate * length))
for (const at of strikes) {
  const start = Math.round(at * rate)
  for (let i = start; i < samples.length; i++) {
    const t = (i - start) / rate
    const attack = Math.min(1, t / 0.003)
    let v = 0
    for (const p of partials) v += p.gain * Math.exp(-t / p.decay) * Math.sin(2 * Math.PI * f0 * p.ratio * t)
    samples[i] += attack * v
  }
}
let peak = 0
for (const v of samples) peak = Math.max(peak, Math.abs(v))
const scale = 0.85 / peak

const data = Buffer.alloc(samples.length * 2)
for (let i = 0; i < samples.length; i++) data.writeInt16LE(Math.round(samples[i] * scale * 32767), i * 2)
const header = Buffer.alloc(44)
header.write('RIFF', 0)
header.writeUInt32LE(36 + data.length, 4)
header.write('WAVE', 8)
header.write('fmt ', 12)
header.writeUInt32LE(16, 16)
header.writeUInt16LE(1, 20) // PCM
header.writeUInt16LE(1, 22) // mono
header.writeUInt32LE(rate, 24)
header.writeUInt32LE(rate * 2, 28)
header.writeUInt16LE(2, 32)
header.writeUInt16LE(16, 34)
header.write('data', 36)
header.writeUInt32LE(data.length, 40)

for (const out of outputs) {
  mkdirSync(dirname(out), { recursive: true })
  writeFileSync(out, Buffer.concat([header, data]))
  console.log(`${out} (${((header.length + data.length) / 1024).toFixed(0)} KB, ${length}s)`)
}
