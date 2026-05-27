# Architecture

`@brashkie/media-codecs` — design rationale and structure.

---

## Design goals

1. **Pluggable codecs.** New codecs register at startup; the public API doesn't change when codecs are added.
2. **Async-first.** All decode/encode calls are `async` because real codecs (Opus, dav1d, hardware) need it.
3. **Zero-cost for stateless codecs.** PCM "encoding" should be a memcpy + metadata attachment, nothing more.
4. **Built on `@brashkie/media-core`.** No duplication of `MediaBuffer`, `MediaError`, etc.
5. **Codec-agnostic public API.** The same `Decoder`/`Encoder` classes work for PCM today and Opus/AV1 tomorrow.

---

## Module layout

```
crates/
├── codecs-core/                Pure Rust — no Node deps
│   └── src/
│       ├── lib.rs              Public re-exports + smoke tests
│       ├── codec.rs            Codec / Encoder / Decoder traits
│       ├── registry.rs         Global registry + singleton
│       ├── pcm.rs              Built-in PCM codecs
│       ├── error.rs            CodecError + CodecErrorKind
│       └── utils.rs            Helpers (align_up, human_size)
│
└── codecs-node/                napi-rs bindings (cdylib)
    └── src/lib.rs              JS-facing wrappers

src/                            TypeScript layer
├── index.ts                    Public entry
├── registry.ts                 CodecRegistry façade
├── codec.ts                    Decoder / Encoder classes
├── error.ts                    CodecError (extends MediaError)
├── types.ts                    Public types/enums
└── native.ts                   Native addon loader + inline types
```

---

## The registry pattern

A single global `CodecRegistry` maps `&str` names to `RegisteredCodec` entries. Each entry has:

- A `&'static CodecDescriptor` (name, long_name, kind, capabilities)
- Optional `decoder_factory: Arc<dyn Fn(&CodecConfig) -> Result<Box<dyn Decoder>>>`
- Optional `encoder_factory: Arc<dyn Fn(&CodecConfig) -> Result<Box<dyn Encoder>>>`

Built-in codecs (PCM) register themselves via `pcm::register_all()` invoked from
`registry::global_registry()`'s `Lazy` initializer.

Future packages can register more codecs by calling `global_registry().register(...)`
when their crate is loaded.

---

## Why traits + dyn?

We use `Box<dyn Decoder>` rather than monomorphization because:
- Codecs are chosen by **string name** at runtime
- Different codecs have wildly different state sizes
- Performance hit is negligible vs codec compute cost itself
- Plays well with the napi-rs layer (`Box<dyn>` becomes a JS handle trivially)

---

## Why a separate `codecs-node` crate?

Same reason as `media-core`:
- `codecs-core` can be consumed by any Rust project, not just Node-backed ones
- Tests, clippy, miri run on pure Rust without napi noise
- napi-rs codegen quirks are isolated to one crate

---

## Error propagation across FFI

`CodecError` carries a `kind` discriminant (`"not_found"`, `"invalid_data"`, ...). When the napi layer throws into JS, the message is formatted as `"[kind] message (context)"`. The TS layer parses that prefix in `parseNativeCodecError` and constructs a typed `CodecError` that extends `MediaError`.

This keeps the JS-side error hierarchy unified with `@brashkie/media-core` while preserving codec-specific information in the `codecKind` field.

---

## PCM as the reference implementation

PCM doesn't need much logic, but it's a complete worked example of the pattern:

- Validates frame alignment (sample size × channels)
- Tracks PTS automatically
- Supports both encode and decode
- All four variants (s16, s32, f32, f64) share one `PcmCodec` struct parameterized by `SampleFormat`
- Stateless except for the PTS counter (resettable via `reset()`)

When we add Opus in v0.2, it follows the same pattern but with real Zig-backed encode/decode internals.

---

## Future: Zig integration (v0.2+)

The plan for the first non-trivial codec (Opus):

1. Use `libopus` via Zig FFI in a new crate `crates/zig-bridge`
2. Wrap it in a `OpusDecoder` struct that implements `Decoder`
3. Register via `opus::register_all()` in `lib.rs`
4. No public API changes — `createDecoder('opus', config)` just works

This is the same `link-zig` feature-gate pattern from `media-core`.

---

## What's NOT in scope here

- Container demuxing (MP4, MKV, WebM) → `media-containers`
- Streaming protocols (RTP, WebRTC) → `media-stream`
- GPU shaders → `media-gpu`
- Subtitle rendering → `media-subtitles`
- High-level transcode helpers → `kryx`

`media-codecs` is **just** codec dispatch + the codec implementations themselves.
