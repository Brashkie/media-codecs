# Roadmap

Directional plan for `@brashkie/media-codecs`. Dates are intentions, not promises.

---

## Current: v0.1.x — Framework + PCM

✅ **Released** — May 2026

- Codec registry, `Codec`/`Encoder`/`Decoder` traits
- 4 built-in PCM codecs (s16le, s32le, f32le, f64le)
- Full TypeScript layer + napi-rs bindings
- Cross-platform CI on 7 targets

---

## v0.2.0 — First Zig codec (Opus)

🎯 **Target: Q3 2026**

The first non-trivial codec. Opus is mature, has clean APIs, covers both
speech and music, and is ubiquitous in WebRTC.

- [ ] `crates/zig-bridge` for FFI to libopus
- [ ] `OpusEncoder` / `OpusDecoder` implementing the traits
- [ ] Configurable bitrate, complexity, application (`voip`/`audio`/`lowdelay`)
- [ ] FEC + DTX support
- [ ] Comprehensive tests + opus-tools reference vector comparison

---

## v0.3.0 — AAC + FLAC

🎯 **Target: Q4 2026**

Two more audio codecs that close most of the practical audio matrix.

- [ ] AAC-LC encoder/decoder via libavcodec or fdk-aac
- [ ] FLAC encode/decode via libFLAC
- [ ] Audio resampling helpers (optional sub-crate)

---

## v0.4.0 — First video codec (H264 decoder)

🎯 **Target: Q1 2027**

H264 decoding via OpenH264 or libavcodec. Encoding is harder (licensing) and
ships in v0.5+.

- [ ] H264 decoder with B-frame support
- [ ] HEVC/H265 decoder (same backend if possible)
- [ ] Pixel format conversion helpers

---

## v0.5.0 — Hardware acceleration scaffolding

🎯 **Target: Q2 2027**

Expose `CodecCapabilities::HARDWARE` and make registered codecs that use
GPU/dedicated hardware (VideoToolbox on macOS, NVDEC on NVIDIA, AMF on AMD).

- [ ] Hardware capability detection
- [ ] VideoToolbox H264/HEVC on Apple Silicon
- [ ] NVDEC H264 on NVIDIA
- [ ] Generic fallback path

---

## v0.6.0 — AV1 + VP9

🎯 **Target: Q3 2027**

Modern video codecs.

- [ ] AV1 decode via dav1d (Rust port: `rav1d`?)
- [ ] AV1 encode via rav1e
- [ ] VP9 decode via libvpx

---

## v1.0.0 — Stable API

🎯 **Target: Q4 2027**

- API frozen, semver enforced
- All major codecs supported
- All audio codecs ported to pure Zig (where feasible)
- Plugin system for third-party codecs

---

## Beyond 1.0

- VVC/H266, EVC
- Image codecs (JPEG XL, AVIF, WebP)
- WebAssembly target
- Embedded (no_std) target

---

## How we prioritize

1. **What downstream packages need.** If `@brashkie/media-stream` needs Opus, Opus jumps the queue.
2. **Ecosystem coverage over polish.** Better to have 5 codecs working at 80% than 1 at 100%.
3. **Stability of the API.** Adding codecs should NEVER break the public API.

---

## Want to influence?

- [Open an issue](https://github.com/Brashkie/media-codecs/issues)
- [Start a discussion](https://github.com/Brashkie/media-codecs/discussions)
