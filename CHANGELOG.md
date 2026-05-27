# Changelog

All notable changes to `@brashkie/media-codecs` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Added
- *(nothing yet — see [ROADMAP](docs/ROADMAP.md))*

---

## [0.1.0] — 2026-05-25

**First public release.** Codec framework + built-in PCM codecs.

### Added

#### Core framework
- `Codec` / `Decoder` / `Encoder` traits (Rust async-trait)
- `CodecRegistry` global singleton — `find`, `list`, `register`, `unregister`
- `CodecDescriptor` static metadata + `CodecCapabilities` bitflags
- `CodecConfig` for instantiation parameters (sample_rate, channels, bitrate, extra_data)
- `DecodeRequest` / `EncodeRequest` + `DecodedFrame` / `EncodedPacket` types
- `MediaKind` (audio/video/subtitle), `SampleFormat` (s16, s32, f32, f64, planar variants)
- `CodecError` discriminated error type with `CodecErrorKind`

#### Built-in codecs
- `pcm_s16le` — PCM signed 16-bit little-endian
- `pcm_s32le` — PCM signed 32-bit little-endian
- `pcm_f32le` — PCM 32-bit float little-endian
- `pcm_f64le` — PCM 64-bit float little-endian

All variants support both encode and decode, run synchronously (zero-copy),
validate frame alignment, and track PTS automatically.

#### TypeScript layer
- `CodecRegistry` class — wraps native registry
- `Decoder` / `Encoder` classes with async API
- `createDecoder` / `createEncoder` factory functions
- `CodecError` extending `MediaError` from `@brashkie/media-core`
- `parseNativeCodecError` + `wrapCodecCall` helpers
- Full typed re-exports: `CodecKind`, `SampleFormat`, `PcmCodecName`, `CodecConfig`, etc.

#### Tooling
- Dual ESM + CJS build via `tsup`
- TypeScript 6.0 strict mode
- napi-rs v2.18.4 bindings (Node 18+ compat)
- 7 platforms supported via per-platform npm sub-packages
- 60+ TypeScript tests via Vitest
- Rust tests via `cargo test`
- Dual-package smoke tests (CJS + ESM)
- Cross-platform CI

### Notes
- Pre-1.0 — API may change between minor versions.
- Depends on `@brashkie/media-core@^0.1.2`.
- Zig integration planned for v0.2 with the first real codec (Opus).

---

[Unreleased]: https://github.com/Brashkie/media-codecs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Brashkie/media-codecs/releases/tag/v0.1.0
