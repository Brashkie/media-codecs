'use strict'

/**
 * @brashkie/media-codecs — CJS example
 *
 * Lists registered codecs and runs a PCM s16le round-trip.
 */

const {
  registry,
  createDecoder,
  createEncoder,
  nativeAddonVersion,
  VERSION,
} = require('@brashkie/media-codecs')

async function main() {
  console.log('media-codecs version:', VERSION)
  console.log('native addon version:', nativeAddonVersion())

  const reg = registry()
  console.log('\nRegistered codecs:')
  for (const desc of reg.list('audio')) {
    console.log(`  ${desc.name.padEnd(12)} — ${desc.longName}`)
  }

  // PCM round-trip
  console.log('\nPCM s16le round-trip:')
  const enc = createEncoder('pcm_s16le', { channels: 2, sampleRate: 48_000 })
  const dec = createDecoder('pcm_s16le', { channels: 2, sampleRate: 48_000 })

  const original = Buffer.alloc(16)
  for (let i = 0; i < 16; i++) original[i] = i

  const pkt = await enc.encode({
    payload: original,
    pts: 0,
    dts: 0,
    isKeyframe: true,
    duration: 0,
  })
  console.log(`  encoded: pts=${pkt.pts}, ${pkt.payload.length} bytes`)

  const frame = await dec.decode(pkt.payload, pkt.pts)
  console.log(`  decoded: pts=${frame.pts}, duration=${frame.duration} samples`)
  console.log(`  identity: ${frame.payload.equals(original)}`)
}

main().catch((err) => {
  console.error('Failed:', err)
  process.exit(1)
})
