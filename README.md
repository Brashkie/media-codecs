<div align="center">

# ⚠️ @brashkie/media-codecs — DEPRECATED ⚠️

</div>

> [!WARNING]
> ### This package has been moved to [`@kryxjs/codecs`](https://www.npmjs.com/package/@kryxjs/codecs)
>
> Development continues at **[`github.com/Brashkie/kryx-codecs`](https://github.com/Brashkie/kryx-codecs)**.
>
> No new features will be added here. Only critical security fixes (if any) until end of 2026.

---

## 🚀 Migrate now

```bash
npm uninstall @brashkie/media-codecs @brashkie/media-core
npm install @kryxjs/codecs @kryxjs/core
```

Update your imports:

```diff
- import { createDecoder, CodecRegistry } from '@brashkie/media-codecs'
- import { MediaError } from '@brashkie/media-core'
+ import { createDecoder, CodecRegistry } from '@kryxjs/codecs'
+ import { MediaError } from '@kryxjs/core'
```

That's it. The public TypeScript API is **identical** — only the package names change.

📖 **[Full migration guide](https://github.com/Brashkie/kryx-codecs/blob/main/docs/MIGRATION.md)**

---

## What's the difference?

| Aspect | `@brashkie/media-codecs@0.1.0` | `@kryxjs/codecs@0.1.0` |
|--------|-------------------------------|-----------------------|
| Status | 🔴 Deprecated | 🟢 Active development |
| Repo | `Brashkie/media-codecs` | [`Brashkie/kryx-codecs`](https://github.com/Brashkie/kryx-codecs) |
| Depends on | `@brashkie/media-core@^0.1.4` | `@kryxjs/core@^0.1.0` |
| TypeScript API | identical | identical |
| Future features | ❌ none | ✅ all new work happens here |

The successor package is part of the broader **[Kryx](https://github.com/Brashkie/kryx-core)** ecosystem — a modular alternative to FFmpeg for Node.js, organized under the `@kryxjs/*` scope.

---

## Why the rename?

The `@kryxjs/*` scope groups all packages of the Kryx ecosystem together (`@kryxjs/core`, `@kryxjs/codecs`, `@kryxjs/codecs-opus`, etc.), making the ecosystem easier to discover and maintain.

`@brashkie/media-codecs` was the prototype name. `@kryxjs/codecs` is the production name.

---

## What did NOT change

- Public TypeScript API (every class, function, type, signature)
- `Codec` / `Decoder` / `Encoder` traits
- `CodecRegistry` global singleton API
- Built-in PCM codecs (`pcm_s16le`, `pcm_s32le`, `pcm_f32le`, `pcm_f64le`)
- `CodecError` hierarchy
- ESM + CJS dual format
- Per-platform native binaries (7 platforms)
- Node.js ≥18 requirement

---

## Status

- `@brashkie/media-codecs@0.1.0` is the **last functional version** (now deprecated).
- No new features will land here. Only critical security fixes (if any) until end of 2026.

---

## Need help migrating?

[Open a discussion on the new repo](https://github.com/Brashkie/kryx-codecs/discussions) or [file an issue](https://github.com/Brashkie/kryx-codecs/issues).

---

<div align="center">

**👉 Go to [`@kryxjs/codecs`](https://www.npmjs.com/package/@kryxjs/codecs) 👈**

[English](README.md) · [Español](README.es.md)

</div>