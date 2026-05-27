/**
 * @brashkie/media-codecs — Decoder + Encoder façades
 *
 * Thin TypeScript wrappers over the native `JsDecoder` / `JsEncoder` classes
 * that add error normalization and richer typing.
 */

import type {
  CodecConfig,
  DecodedFrame,
  EncodedPacket,
} from './types'
import { CodecError, wrapCodecCall } from './error'
import type {
  JsDecoder as NativeDecoder,
  JsEncoder as NativeEncoder,
} from './native'

// ─── Decoder ────────────────────────────────────────────────────────────────

/**
 * Stateful audio/video decoder.
 *
 * Instances are obtained via {@link createDecoder} or
 * {@link CodecRegistry.createDecoder}. Always call {@link Decoder.flush}
 * at end-of-stream to drain any buffered frames.
 *
 * @example
 * ```ts
 * const decoder = createDecoder('pcm_s16le', { sampleRate: 48000, channels: 2 })
 * const frame = await decoder.decode(rawBytes)
 * console.log(frame.pts, frame.duration)
 * ```
 */
export class Decoder {
  /** @internal */
  constructor(private readonly inner: NativeDecoder) {}

  /** The codec name this decoder was created for. */
  get name(): string {
    return this.inner.name
  }

  /** Decode one packet of encoded data into a frame. */
  async decode(data: Buffer | Uint8Array, pts?: number): Promise<DecodedFrame> {
    return wrapCodecCall(`decode(${this.inner.name})`, async () => {
      const buf = data instanceof Buffer ? data : Buffer.from(data)
      const frame = await this.inner.decode(buf, pts ?? null)
      return frame as DecodedFrame
    })
  }

  /** Drain any buffered frames at end of stream. */
  async flush(): Promise<readonly DecodedFrame[]> {
    return wrapCodecCall(`flush(${this.inner.name})`, async () => {
      const frames = await this.inner.flush()
      return frames as DecodedFrame[]
    })
  }

  /** Reset internal state — useful after a seek. */
  async reset(): Promise<void> {
    return wrapCodecCall(`reset(${this.inner.name})`, () => this.inner.reset())
  }
}

// ─── Encoder ────────────────────────────────────────────────────────────────

/**
 * Stateful audio/video encoder.
 *
 * @example
 * ```ts
 * const encoder = createEncoder('pcm_f32le', { sampleRate: 48000, channels: 2 })
 * const packet = await encoder.encode({
 *   payload: rawSamples,
 *   pts: 0, dts: 0, isKeyframe: true, duration: 0,
 * })
 * ```
 */
export class Encoder {
  /** @internal */
  constructor(private readonly inner: NativeEncoder) {}

  /** The codec name this encoder was created for. */
  get name(): string {
    return this.inner.name
  }

  /** Encode one frame into a packet. */
  async encode(frame: DecodedFrame): Promise<EncodedPacket> {
    return wrapCodecCall(`encode(${this.inner.name})`, async () => {
      const payload =
        frame.payload instanceof Buffer ? frame.payload : Buffer.from(frame.payload as Uint8Array)

      const pkt = await this.inner.encode({
        payload,
        pts: frame.pts,
        dts: frame.dts,
        isKeyframe: frame.isKeyframe,
        duration: frame.duration,
      })
      return pkt as EncodedPacket
    })
  }

  /** Drain any buffered packets at end of stream. */
  async flush(): Promise<readonly EncodedPacket[]> {
    return wrapCodecCall(`flush(${this.inner.name})`, async () => {
      const packets = await this.inner.flush()
      return packets as EncodedPacket[]
    })
  }

  /** Reset internal state. */
  async reset(): Promise<void> {
    return wrapCodecCall(`reset(${this.inner.name})`, () => this.inner.reset())
  }
}

// ─── Re-export error for convenience ───────────────────────────────────────
export { CodecError, type CodecConfig }
