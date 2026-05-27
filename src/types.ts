/**
 * @brashkie/media-codecs — shared types
 *
 * TypeScript-side mirrors of the Rust types exposed via napi-rs.
 */

// ─── Discriminants ───────────────────────────────────────────────────────────

export const CodecKind = {
  Audio: 'audio',
  Video: 'video',
  Subtitle: 'subtitle',
} as const
export type CodecKind = (typeof CodecKind)[keyof typeof CodecKind]

export const SampleFormat = {
  S16: 's16',
  S32: 's32',
  F32: 'f32',
  F64: 'f64',
  S16Planar: 's16p',
  F32Planar: 'f32p',
} as const
export type SampleFormat = (typeof SampleFormat)[keyof typeof SampleFormat]

// ─── Config / descriptors ───────────────────────────────────────────────────

/** Options passed when creating an encoder or decoder. */
export interface CodecConfig {
  readonly sampleRate?: number
  readonly channels?: number
  readonly sampleFormat?: SampleFormat
  readonly bitrate?: number
  readonly extraData?: Buffer | Uint8Array
}

/** Static metadata about a codec. */
export interface CodecDescriptor {
  readonly name: string
  readonly longName: string
  readonly kind: CodecKind
  readonly canDecode: boolean
  readonly canEncode: boolean
  readonly isLossless: boolean
  readonly isHardware: boolean
}

// ─── Frames / packets ───────────────────────────────────────────────────────

/** A decoded frame produced by `Decoder.decode`. */
export interface DecodedFrame {
  readonly payload: Buffer
  readonly pts: number
  readonly dts: number
  readonly isKeyframe: boolean
  readonly duration: number
}

/** An encoded packet produced by `Encoder.encode`. */
export interface EncodedPacket {
  readonly payload: Buffer
  readonly pts: number
  readonly dts: number
  readonly isKeyframe: boolean
  readonly duration: number
}

// ─── Built-in codec names (typed string union) ──────────────────────────────

/** All PCM variants shipped in the base package. */
export const PcmCodecName = {
  S16LE: 'pcm_s16le',
  S32LE: 'pcm_s32le',
  F32LE: 'pcm_f32le',
  F64LE: 'pcm_f64le',
} as const
export type PcmCodecName = (typeof PcmCodecName)[keyof typeof PcmCodecName]

/** Stable type-safe union of all built-in codecs. Future codecs widen this. */
export type BuiltinCodecName = PcmCodecName
