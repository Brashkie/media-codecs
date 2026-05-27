import { describe, it, expect, beforeEach } from 'vitest'
import {
  CodecRegistry,
  registry,
  createDecoder,
  createEncoder,
  Decoder,
  Encoder,
  CodecError,
  CodecKind,
  SampleFormat,
  PcmCodecName,
  parseNativeCodecError,
  wrapCodecCall,
  nativeAddonVersion,
  VERSION,
} from '../src'

// ═════════════════════════════════════════════════════════════════════════════
// Constants & type re-exports
// ═════════════════════════════════════════════════════════════════════════════

describe('Constants', () => {
  it('VERSION is defined', () => {
    expect(typeof VERSION).toBe('string')
    expect(VERSION.length).toBeGreaterThan(0)
  })

  it('CodecKind has 3 entries', () => {
    expect(CodecKind.Audio).toBe('audio')
    expect(CodecKind.Video).toBe('video')
    expect(CodecKind.Subtitle).toBe('subtitle')
  })

  it('SampleFormat covers all variants', () => {
    expect(SampleFormat.S16).toBe('s16')
    expect(SampleFormat.S32).toBe('s32')
    expect(SampleFormat.F32).toBe('f32')
    expect(SampleFormat.F64).toBe('f64')
    expect(SampleFormat.S16Planar).toBe('s16p')
    expect(SampleFormat.F32Planar).toBe('f32p')
  })

  it('PcmCodecName has 4 entries', () => {
    expect(Object.values(PcmCodecName).sort()).toEqual([
      'pcm_f32le',
      'pcm_f64le',
      'pcm_s16le',
      'pcm_s32le',
    ])
  })

  it('nativeAddonVersion returns a non-empty string', () => {
    const v = nativeAddonVersion()
    expect(typeof v).toBe('string')
    expect(v.length).toBeGreaterThan(0)
  })
})

// ═════════════════════════════════════════════════════════════════════════════
// Registry
// ═════════════════════════════════════════════════════════════════════════════

describe('CodecRegistry', () => {
  let reg: CodecRegistry

  beforeEach(() => {
    reg = new CodecRegistry()
  })

  it('names() returns all built-in PCM codecs sorted', () => {
    const names = reg.names()
    expect(names).toContain('pcm_s16le')
    expect(names).toContain('pcm_s32le')
    expect(names).toContain('pcm_f32le')
    expect(names).toContain('pcm_f64le')
    expect([...names]).toEqual([...names].sort())
  })

  it('list() with no filter returns all', () => {
    const all = reg.list()
    expect(all.length).toBeGreaterThanOrEqual(4)
  })

  it('list("audio") returns audio codecs only', () => {
    const audio = reg.list('audio')
    expect(audio.length).toBeGreaterThanOrEqual(4)
    expect(audio.every((d) => d.kind === 'audio')).toBe(true)
  })

  it('list("video") returns empty for now', () => {
    const video = reg.list('video')
    expect(video).toHaveLength(0)
  })

  it('find() returns descriptor for known codec', () => {
    const d = reg.find('pcm_s16le')
    expect(d).not.toBeNull()
    expect(d!.name).toBe('pcm_s16le')
    expect(d!.kind).toBe('audio')
    expect(d!.canDecode).toBe(true)
    expect(d!.canEncode).toBe(true)
    expect(d!.isLossless).toBe(true)
    expect(d!.isHardware).toBe(false)
  })

  it('find() returns null for unknown codec', () => {
    expect(reg.find('does-not-exist')).toBeNull()
  })

  it('has() works for known + unknown', () => {
    expect(reg.has('pcm_s16le')).toBe(true)
    expect(reg.has('not-a-codec')).toBe(false)
  })

  it('registry() singleton returns the same instance', () => {
    const a = registry()
    const b = registry()
    expect(a).toBe(b)
  })

  it('CodecRegistry exposes createDecoder/createEncoder', () => {
    const dec = reg.createDecoder('pcm_s16le', { channels: 2, sampleRate: 48_000 })
    expect(dec).toBeInstanceOf(Decoder)
    const enc = reg.createEncoder('pcm_s16le', { channels: 2, sampleRate: 48_000 })
    expect(enc).toBeInstanceOf(Encoder)
  })
})

// ═════════════════════════════════════════════════════════════════════════════
// Decoder
// ═════════════════════════════════════════════════════════════════════════════

describe('Decoder', () => {
  it('decode() returns a frame with pts and duration', async () => {
    const dec = createDecoder('pcm_s16le', { channels: 2, sampleRate: 48_000 })
    const data = Buffer.from([0, 0, 0, 0, 0, 0, 0, 0])
    const frame = await dec.decode(data)
    expect(frame.pts).toBe(0)
    expect(frame.duration).toBe(2)
    expect(frame.isKeyframe).toBe(true)
    expect(frame.payload.length).toBe(8)
  })

  it('decode() with explicit pts uses it', async () => {
    const dec = createDecoder('pcm_s16le', { channels: 2 })
    const frame = await dec.decode(Buffer.alloc(8), 9_000_000)
    expect(frame.pts).toBe(9_000_000)
  })

  it('decode() PTS increments across calls', async () => {
    const dec = createDecoder('pcm_s16le', { channels: 2 })
    const data = Buffer.alloc(8)
    const f1 = await dec.decode(data)
    const f2 = await dec.decode(data)
    expect(f1.pts).toBe(0)
    expect(f2.pts).toBe(2)
  })

  it('decode() accepts Uint8Array', async () => {
    const dec = createDecoder('pcm_s16le', { channels: 2 })
    const data = new Uint8Array(8)
    const frame = await dec.decode(data)
    expect(frame.payload.length).toBe(8)
  })

  it('decode() throws CodecError on bad alignment', async () => {
    const dec = createDecoder('pcm_s16le', { channels: 2 })
    await expect(dec.decode(Buffer.from([0, 0, 0]))).rejects.toBeInstanceOf(CodecError)
  })

  it('flush() returns empty array for PCM (stateless)', async () => {
    const dec = createDecoder('pcm_s16le', { channels: 2 })
    const frames = await dec.flush()
    expect(frames).toEqual([])
  })

  it('reset() resets PTS counter', async () => {
    const dec = createDecoder('pcm_s16le', { channels: 2 })
    await dec.decode(Buffer.alloc(8))
    await dec.reset()
    const f = await dec.decode(Buffer.alloc(8))
    expect(f.pts).toBe(0)
  })

  it('name getter returns codec name', () => {
    const dec = createDecoder('pcm_f32le', { channels: 2 })
    expect(dec.name).toBe('pcm_f32le')
  })
})

// ═════════════════════════════════════════════════════════════════════════════
// Encoder
// ═════════════════════════════════════════════════════════════════════════════

describe('Encoder', () => {
  it('encode() returns a packet', async () => {
    const enc = createEncoder('pcm_s16le', { channels: 2 })
    const pkt = await enc.encode({
      payload: Buffer.alloc(8),
      pts: 0,
      dts: 0,
      isKeyframe: true,
      duration: 2,
    })
    expect(pkt.payload.length).toBe(8)
    expect(pkt.duration).toBe(2)
    expect(pkt.isKeyframe).toBe(true)
  })

  it('encode() accepts Uint8Array payload', async () => {
    const enc = createEncoder('pcm_s16le', { channels: 2 })
    const arr = new Uint8Array(8)
    const pkt = await enc.encode({
      payload: arr as unknown as Buffer,
      pts: 100,
      dts: 100,
      isKeyframe: true,
      duration: 0,
    })
    expect(pkt.pts).toBe(100)
  })

  it('flush() returns empty array for PCM', async () => {
    const enc = createEncoder('pcm_s16le', { channels: 2 })
    const packets = await enc.flush()
    expect(packets).toEqual([])
  })

  it('reset() works', async () => {
    const enc = createEncoder('pcm_s16le', { channels: 2 })
    await expect(enc.reset()).resolves.toBeUndefined()
  })

  it('name getter returns codec name', () => {
    const enc = createEncoder('pcm_s32le', { channels: 2 })
    expect(enc.name).toBe('pcm_s32le')
  })
})

// ═════════════════════════════════════════════════════════════════════════════
// Roundtrip
// ═════════════════════════════════════════════════════════════════════════════

describe('Encode/Decode roundtrip', () => {
  it('PCM f32le roundtrip is identity', async () => {
    const enc = createEncoder('pcm_f32le', { channels: 2 })
    const dec = createDecoder('pcm_f32le', { channels: 2 })

    const original = Buffer.alloc(16)
    for (let i = 0; i < 16; i++) original[i] = i

    const pkt = await enc.encode({
      payload: original,
      pts: 1000,
      dts: 1000,
      isKeyframe: true,
      duration: 0,
    })
    const frame = await dec.decode(pkt.payload, pkt.pts)

    expect(frame.payload).toEqual(original)
    expect(frame.pts).toBe(1000)
  })
})

// ═════════════════════════════════════════════════════════════════════════════
// Factories
// ═════════════════════════════════════════════════════════════════════════════

describe('createDecoder / createEncoder', () => {
  it('createDecoder works without config', () => {
    const dec = createDecoder('pcm_s16le')
    expect(dec).toBeInstanceOf(Decoder)
  })

  it('createEncoder works without config', () => {
    const enc = createEncoder('pcm_s16le')
    expect(enc).toBeInstanceOf(Encoder)
  })

  it('createDecoder throws CodecError on unknown codec', () => {
    expect(() => createDecoder('not-a-codec')).toThrow(CodecError)
  })

  it('createEncoder throws CodecError on unknown codec', () => {
    expect(() => createEncoder('not-a-codec')).toThrow(CodecError)
  })

  it('createDecoder forwards extraData buffer', () => {
    const dec = createDecoder('pcm_s16le', {
      channels: 2,
      extraData: Buffer.from([0xde, 0xad]),
    })
    expect(dec.name).toBe('pcm_s16le')
  })

  it('createDecoder accepts Uint8Array extraData', () => {
    const dec = createDecoder('pcm_s16le', {
      channels: 2,
      extraData: new Uint8Array([0xbe, 0xef]),
    })
    expect(dec.name).toBe('pcm_s16le')
  })
})

// ═════════════════════════════════════════════════════════════════════════════
// CodecError
// ═════════════════════════════════════════════════════════════════════════════

describe('CodecError', () => {
  it('is an Error instance', () => {
    const e = new CodecError('not_found', 'oops')
    expect(e).toBeInstanceOf(Error)
    expect(e).toBeInstanceOf(CodecError)
  })

  it('exposes codecKind', () => {
    const e = new CodecError('invalid_data', 'bad input')
    expect(e.codecKind).toBe('invalid_data')
  })

  it('exposes message from Error parent', () => {
    const e = new CodecError('not_found', 'codec missing: opus')
    expect((e as Error).message).toContain('codec missing')
  })

  it('parseNativeCodecError parses bracket-prefix format', () => {
    const e = parseNativeCodecError(new Error('[not_found] codec missing: opus'))
    expect(e).toBeInstanceOf(CodecError)
    expect(e.codecKind).toBe('not_found')
    expect((e as Error).message).toContain('codec missing')
  })

  it('parseNativeCodecError handles non-error throwables', () => {
    const e = parseNativeCodecError('plain string')
    expect(e).toBeInstanceOf(CodecError)
    expect(e.codecKind).toBe('internal')
  })

  it('parseNativeCodecError preserves CodecError pass-through', () => {
    const original = new CodecError('cancelled', 'aborted')
    const out = parseNativeCodecError(original)
    expect(out).toBe(original)
  })

  it('parseNativeCodecError handles malformed prefix', () => {
    const e = parseNativeCodecError(new Error('no brackets here'))
    expect(e.codecKind).toBe('internal')
    expect((e as Error).message).toBe('no brackets here')
  })

  it('wrapCodecCall passes through resolved values', async () => {
    const v = await wrapCodecCall('label', async () => 42)
    expect(v).toBe(42)
  })

  it('wrapCodecCall normalizes thrown errors', async () => {
    await expect(
      wrapCodecCall('decode-test', async () => {
        throw new Error('[invalid_data] bad')
      }),
    ).rejects.toBeInstanceOf(CodecError)
  })

  it('wrapCodecCall passes through CodecError without re-wrapping', async () => {
    const original = new CodecError('cancelled', 'aborted by user')
    await expect(
      wrapCodecCall('decode-test', async () => {
        throw original
      }),
    ).rejects.toBe(original) // .toBe checks identity — must be the same instance
  })

  it('parseNativeCodecError maps buffer_too_small kind', () => {
    const e = parseNativeCodecError(new Error('[buffer_too_small] need 100 got 50'))
    expect(e.codecKind).toBe('buffer_too_small')
    // Internally mapped to MediaErrorKind.Internal
    expect(e.kind).toBe('internal')
  })

  it('parseNativeCodecError maps invalid_state kind', () => {
    const e = parseNativeCodecError(new Error('[invalid_state] decoder closed'))
    expect(e.codecKind).toBe('invalid_state')
    // Internally mapped to MediaErrorKind.Closed
    expect(e.kind).toBe('closed')
  })

  it('parseNativeCodecError maps cancelled kind', () => {
    const e = parseNativeCodecError(new Error('[cancelled] aborted'))
    expect(e.codecKind).toBe('cancelled')
    expect(e.kind).toBe('timeout')
  })

  it('parseNativeCodecError maps unsupported kind', () => {
    const e = parseNativeCodecError(new Error('[unsupported] not implemented'))
    expect(e.codecKind).toBe('unsupported')
    expect(e.kind).toBe('unsupported')
  })
})