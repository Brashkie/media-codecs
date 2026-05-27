/**
 * @brashkie/media-codecs — Registry façade
 *
 * Thin wrapper around the native registry that adds typed list-filtering,
 * presence checks, and the factory functions consumers use.
 */

import type { CodecConfig, CodecDescriptor, CodecKind } from './types'
import { Decoder, Encoder } from './codec'
import { wrapCodecCall, parseNativeCodecError } from './error'
import {
  JsCodecRegistry,
  createDecoder as nativeCreateDecoder,
  createEncoder as nativeCreateEncoder,
} from './native'

// ─── Registry class ─────────────────────────────────────────────────────────

/**
 * Read-only handle to the process-wide codec registry.
 *
 * The registry is populated at native-addon load time with all built-in
 * codecs (PCM). Future packages register more codecs by being imported.
 *
 * @example
 * ```ts
 * const registry = new CodecRegistry()
 * console.log(registry.names())
 * // → ['pcm_f32le', 'pcm_f64le', 'pcm_s16le', 'pcm_s32le']
 *
 * const audio = registry.list('audio')
 * for (const codec of audio) {
 *   console.log(codec.name, codec.longName)
 * }
 * ```
 */
export class CodecRegistry {
  private readonly inner = new JsCodecRegistry()

  /** All registered codec names, sorted. */
  names(): readonly string[] {
    return this.inner.names()
  }

  /** Filter descriptors by media kind. Pass `undefined` for all kinds. */
  list(kind?: CodecKind): readonly CodecDescriptor[] {
    return this.inner.list(kind ?? null) as CodecDescriptor[]
  }

  /** Find a codec by name. Returns `null` if not registered. */
  find(name: string): CodecDescriptor | null {
    const found = this.inner.find(name)
    return (found as CodecDescriptor | null) ?? null
  }

  /** Whether `name` is registered. */
  has(name: string): boolean {
    return this.find(name) !== null
  }

  /** Convenience: create a decoder via the registry. */
  createDecoder(name: string, config?: CodecConfig): Decoder {
    return createDecoder(name, config)
  }

  /** Convenience: create an encoder via the registry. */
  createEncoder(name: string, config?: CodecConfig): Encoder {
    return createEncoder(name, config)
  }
}

// ─── Singleton accessor ─────────────────────────────────────────────────────

let _singleton: CodecRegistry | undefined

/** Access the process-wide registry (lazy singleton). */
export function registry(): CodecRegistry {
  if (!_singleton) _singleton = new CodecRegistry()
  return _singleton
}

// ─── Factory functions ──────────────────────────────────────────────────────

function configToNative(c: CodecConfig | undefined) {
  if (!c) return null
  return {
    sampleRate: c.sampleRate,
    channels: c.channels,
    sampleFormat: c.sampleFormat,
    bitrate: c.bitrate,
    extraData:
      c.extraData instanceof Buffer
        ? c.extraData
        : c.extraData
          ? Buffer.from(c.extraData)
          : undefined,
  }
}

/**
 * Build a {@link Decoder} for the given codec name.
 *
 * Throws {@link CodecError} (kind `not_found`) if the codec is not registered,
 * or `unsupported` if the codec exists but can't be used as a decoder.
 */
export function createDecoder(name: string, config?: CodecConfig): Decoder {
  try {
    const native = nativeCreateDecoder(name, configToNative(config))
    return new Decoder(native)
  } catch (err) {
    throw parseNativeCodecError(err)
  }
}

/**
 * Build an {@link Encoder} for the given codec name.
 *
 * Throws {@link CodecError} (kind `not_found`) if the codec is not registered,
 * or `unsupported` if the codec can't be used as an encoder.
 */
export function createEncoder(name: string, config?: CodecConfig): Encoder {
  try {
    const native = nativeCreateEncoder(name, configToNative(config))
    return new Encoder(native)
  } catch (err) {
    throw parseNativeCodecError(err)
  }
}
