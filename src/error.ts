/**
 * @brashkie/media-codecs — error types
 *
 * Codec errors derive from `MediaError` in `@brashkie/media-core` so that all
 * Kryx packages share a single error class hierarchy.
 *
 * The native addon throws plain JS `Error`s whose message starts with
 * `"[kind] ..."`. We parse that prefix and convert into the right
 * `MediaError` subclass.
 */

import { MediaError, MediaErrorKind } from '@brashkie/media-core'

/** Codec-specific error subclass. */
export class CodecError extends MediaError {
  /** Original codec error kind from Rust (e.g. `"not_found"`, `"invalid_data"`). */
  public readonly codecKind: string

  constructor(codecKind: string, message: string, options: { cause?: unknown } = {}) {
    super(mapCodecKind(codecKind), message, options)
    this.codecKind = codecKind
    this.name = 'CodecError'
  }
}

/** Parse a string of the form `"[kind] message"` thrown by the native addon. */
export function parseNativeCodecError(err: unknown): CodecError {
  if (err instanceof CodecError) return err

  const msg = err instanceof Error ? err.message : String(err)
  const match = /^\[([a-z_]+)]\s+(.+)$/.exec(msg)
  if (match) {
    const [, kind, body] = match
    return new CodecError(kind, body, { cause: err })
  }
  // Fallback — unrecognized format
  return new CodecError('internal', msg, { cause: err })
}

/** Wrap an arbitrary call so errors get normalized into `CodecError`. */
export async function wrapCodecCall<T>(label: string, fn: () => Promise<T>): Promise<T> {
  try {
    return await fn()
  } catch (err) {
    if (CodecError.is(err)) throw err
    const wrapped = parseNativeCodecError(err)
    // Attach context if missing
    if (!wrapped.context) {
      Object.defineProperty(wrapped, 'context', { value: label, enumerable: true })
    }
    throw wrapped
  }
}

// ─── Mapping ────────────────────────────────────────────────────────────────

function mapCodecKind(codecKind: string): MediaErrorKind {
  switch (codecKind) {
    case 'not_found':
    case 'unsupported':
      return MediaErrorKind.Unsupported
    case 'invalid_data':
      return MediaErrorKind.Internal
    case 'buffer_too_small':
      return MediaErrorKind.Internal
    case 'invalid_state':
      return MediaErrorKind.Closed
    case 'cancelled':
      return MediaErrorKind.Timeout
    case 'internal':
    default:
      return MediaErrorKind.Internal
  }
}
