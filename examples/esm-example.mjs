/**
 * @brashkie/media-codecs — ESM example
 */

import {
  registry,
  createDecoder,
  createEncoder,
  nativeAddonVersion,
  VERSION,
} from '@brashkie/media-codecs'

console.log('media-codecs version:', VERSION)
console.log('native addon version:', nativeAddonVersion())

const reg = registry()
console.log('\nRegistered codecs:')
for (const desc of reg.list('audio')) {
  console.log(`  ${desc.name.padEnd(12)} — ${desc.longName}`)
}

// PCM f32le round-trip
console.log('\nPCM f32le round-trip:')
const enc = createEncoder('pcm_f32le', { channels: 2, sampleRate: 48_000 })
const dec = createDecoder('pcm_f32le', { channels: 2, sampleRate: 48_000 })

const original = Buffer.alloc(32)
for (let i = 0; i < 32; i++) original[i] = i * 2

const pkt = await enc.encode({
  payload: original,
  pts: 1_000,
  dts: 1_000,
  isKeyframe: true,
  duration: 0,
})
console.log(`  encoded: pts=${pkt.pts}, ${pkt.payload.length} bytes`)

const frame = await dec.decode(pkt.payload, pkt.pts)
console.log(`  decoded: pts=${frame.pts}, duration=${frame.duration} samples`)
console.log(`  identity: ${frame.payload.equals(original)}`)
