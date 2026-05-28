<div align="center">

<img src="media/logo.png" alt="Kryx media-codecs" width="200" />

**Framework de codecs del ecosistema multimedia [Kryx](https://kryx.dev)**

Encoders y decoders enchufables · PCM nativo · Construido sobre `@brashkie/media-core`

[![CI](https://github.com/Brashkie/media-codecs/actions/workflows/ci.yml/badge.svg)](https://github.com/Brashkie/media-codecs/actions)
[![npm version](https://img.shields.io/npm/v/@brashkie/media-codecs?color=cb3837&logo=npm)](https://npmjs.com/package/@brashkie/media-codecs)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)
[![node ≥18](https://img.shields.io/badge/node-%E2%89%A518-3c873a?logo=node.js)](https://nodejs.org)

[English](README.md) · **Español** · [Arquitectura](docs/ARCHITECTURE.md) · [Roadmap](docs/ROADMAP.md) · [Changelog](CHANGELOG.md)

</div>

---

## ¿Qué es esto?

`@brashkie/media-codecs` es la **capa de codecs** de [Kryx](https://kryx.dev). Provee:

- Un **registry** enchufable para lookup de codecs en runtime
- Clases async `Decoder` / `Encoder` con API uniforme
- Codecs **PCM** nativos (s16le, s32le, f32le, f64le)
- Base para las próximas implementaciones de Opus, AAC, H264, AV1

**No incluye** codecs pesados aún — esos llegan en v0.2+ como módulos separados respaldados por Zig. Los codecs PCM son la implementación de referencia del protocolo.

```bash
npm install @brashkie/media-codecs
```

```ts
import { createDecoder, createEncoder, registry } from '@brashkie/media-codecs'

// Ver qué hay disponible
console.log(registry().names())
// → ['pcm_f32le', 'pcm_f64le', 'pcm_s16le', 'pcm_s32le']

// Round-trip PCM
const enc = createEncoder('pcm_s16le', { channels: 2, sampleRate: 48_000 })
const dec = createDecoder('pcm_s16le', { channels: 2, sampleRate: 48_000 })

const pkt = await enc.encode({
  payload: Buffer.alloc(8),
  pts: 0, dts: 0, isKeyframe: true, duration: 0,
})

const frame = await dec.decode(pkt.payload, pkt.pts)
console.log(frame.pts, frame.duration) // → 0, 2
```

---

## ¿Por qué?

| | |
|---|---|
| 🔌 **Enchufable** | Los codecs se registran al inicio — la API pública nunca cambia |
| ⚡ **Async-first** | Todas las llamadas devuelven Promises — backpressure-friendly |
| 🎯 **Zero-cost para PCM** | Valida + etiqueta metadatos, sin copia real |
| 🧩 **Construido sobre media-core** | Reutiliza `MediaError`, mismo ecosistema |
| 🔒 **Type-safe** | TypeScript 6.0 estricto + `.d.ts` autogenerados |
| 📦 **Dual-package** | ESM + CJS con exports map adecuado |
| 🌐 **Multiplataforma** | Windows, macOS, Linux en x64 y arm64 |

---

## Codecs nativos (v0.1)

| Nombre | Nombre largo | Tipo | Encode | Decode |
|--------|--------------|------|--------|--------|
| `pcm_s16le` | PCM signed 16-bit little-endian | audio | ✅ | ✅ |
| `pcm_s32le` | PCM signed 32-bit little-endian | audio | ✅ | ✅ |
| `pcm_f32le` | PCM 32-bit float little-endian   | audio | ✅ | ✅ |
| `pcm_f64le` | PCM 64-bit float little-endian   | audio | ✅ | ✅ |

Todos son lossless, stateless (excepto el contador de PTS) y validados contra alineación de frames.

---

## Roadmap

Ver [docs/ROADMAP.md](docs/ROADMAP.md). Próximos pasos:

- **v0.2** — Opus (primer codec con backend Zig)
- **v0.3** — AAC + FLAC
- **v0.4** — Decoder H264
- **v0.5** — Aceleración hardware
- **v0.6** — AV1 + VP9
- **v1.0** — API estable

---

## Licencia

[Apache-2.0](LICENSE) © 2026 [Brashkie](https://github.com/Brashkie)

---

<div align="center">

Hecho con 🦀 + ⚡ para la web multimedia moderna.

[Sitio web](https://kryx.dev) · [Issues](https://github.com/Brashkie/media-codecs/issues)

</div>
