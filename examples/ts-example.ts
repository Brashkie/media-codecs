/**
 * @brashkie/media-codecs — TypeScript example
 */

import {
  registry,
  createDecoder,
  createEncoder,
  nativeAddonVersion,
  VERSION,
  type CodecConfig,
  type DecodedFrame,
} from '@brashkie/media-codecs'

console.log('media-codecs version:', VERSION)
console.log('native addon version:', nativeAddonVersion())

const reg = registry()
console.log('\nAll registered codecs:', reg.names())

const config: CodecConfig = {
  channels: 2,
  sampleRate: 48_000,
}

const enc = createEncoder('pcm_s16le', config)
const dec = createDecoder('pcm_s16le', config)

const frame: DecodedFrame = {
  payload: Buffer.alloc(8),
  pts: 0,
  dts: 0,
  isKeyframe: true,
  duration: 0,
}

const pkt = await enc.encode(frame)
const out = await dec.decode(pkt.payload, pkt.pts)

console.log('\nFrame decoded:')
console.log('  pts:', out.pts)
console.log('  duration:', out.duration)
console.log('  isKeyframe:', out.isKeyframe)
