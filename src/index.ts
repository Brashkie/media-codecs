/**
 * @brashkie/media-codecs
 *
 * Codec framework for the Kryx multimedia ecosystem.
 * Built on top of `@brashkie/media-core`.
 *
 * Provides:
 *   - A global codec **registry** (lookup by name)
 *   - `Decoder` / `Encoder` classes with async API
 *   - Built-in **PCM** codecs (s16le, s32le, f32le, f64le)
 *   - `CodecError` extending `MediaError` from `@brashkie/media-core`
 */

// ─── Native addon version ───────────────────────────────────────────────────

export { nativeAddonVersion } from './native'

// ─── Public API ─────────────────────────────────────────────────────────────

export {
  CodecRegistry,
  registry,
  createDecoder,
  createEncoder,
} from './registry'

export { Decoder, Encoder } from './codec'

export { CodecError, parseNativeCodecError, wrapCodecCall } from './error'

export {
  CodecKind,
  SampleFormat,
  PcmCodecName,
  type CodecConfig,
  type CodecDescriptor,
  type DecodedFrame,
  type EncodedPacket,
  type BuiltinCodecName,
} from './types'

// ─── Package version ────────────────────────────────────────────────────────

/** npm package version of `@brashkie/media-codecs`. */
export const VERSION = '0.1.0'
