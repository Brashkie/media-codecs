/**
 * @brashkie/media-codecs — native addon loader
 *
 * Uses a STATIC `import * as native from '../index.js'` so that tsup leaves
 * it as a clean `require()` in CJS output and a literal `import` in ESM
 * output — both work natively in Node and Vitest, without the broken
 * `__require` shim that `shims: true` would generate.
 *
 * Pattern proven by @brashkie/signalis-core and @brashkie/media-core@0.1.4.
 */

// eslint-disable-next-line @typescript-eslint/no-var-requires
import * as native from '../index.js'

// ─── Native addon types ─────────────────────────────────────────────────────
//
// We declare these locally rather than relying on `../index.d.ts` because
// napi-rs codegen output can change between versions / be missing during
// initial development. The actual runtime values come from `native`.

export interface JsCodecDescriptorNative {
  name: string
  longName: string
  kind: string
  canDecode: boolean
  canEncode: boolean
  isLossless: boolean
  isHardware: boolean
}

export interface JsCodecConfigNative {
  sampleRate?: number | null
  channels?: number | null
  sampleFormat?: string | null
  bitrate?: number | null
  extraData?: Buffer | null
}

export interface JsDecodedFrameNative {
  payload: Buffer
  pts: number
  dts: number
  isKeyframe: boolean
  duration: number
}

export interface JsEncodedPacketNative {
  payload: Buffer
  pts: number
  dts: number
  isKeyframe: boolean
  duration: number
}

export interface JsCodecRegistry {
  names(): string[]
  list(kind: string | null): JsCodecDescriptorNative[]
  find(name: string): JsCodecDescriptorNative | null
}

export interface JsCodecRegistryConstructor {
  new (): JsCodecRegistry
}

export interface JsDecoder {
  readonly name: string
  decode(data: Buffer, pts: number | null): Promise<JsDecodedFrameNative>
  flush(): Promise<JsDecodedFrameNative[]>
  reset(): Promise<void>
}

export interface JsEncoder {
  readonly name: string
  encode(frame: JsDecodedFrameNative): Promise<JsEncodedPacketNative>
  flush(): Promise<JsEncodedPacketNative[]>
  reset(): Promise<void>
}

/** Runtime shape of the loaded native addon. */
interface NativeAddon {
  JsCodecRegistry: JsCodecRegistryConstructor
  createDecoder: (name: string, config: JsCodecConfigNative | null) => JsDecoder
  createEncoder: (name: string, config: JsCodecConfigNative | null) => JsEncoder
  version: () => string
}

// ─── Re-export native values ────────────────────────────────────────────────
//
// Cast through `unknown` because `import * as native` types as a module
// namespace, not as our `NativeAddon` interface — but the runtime shape
// matches exactly.

const addon = native as unknown as NativeAddon

export const JsCodecRegistry = addon.JsCodecRegistry
export const createDecoder = addon.createDecoder
export const createEncoder = addon.createEncoder

/** Returns the version reported by the linked native addon. */
export function nativeAddonVersion(): string {
  return addon.version()
}
