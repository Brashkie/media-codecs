<div align="center">

# ⚠️ @brashkie/media-codecs — OBSOLETO ⚠️

</div>

> [!WARNING]
> ### Este paquete fue movido a [`@kryxjs/codecs`](https://www.npmjs.com/package/@kryxjs/codecs)
>
> El desarrollo continúa en **[`github.com/Brashkie/kryx-codecs`](https://github.com/Brashkie/kryx-codecs)**.
>
> Aquí no se agregarán nuevas funcionalidades. Solo correcciones críticas de seguridad (si surgen) hasta fines de 2026.

---

## 🚀 Migra ahora

```bash
npm uninstall @brashkie/media-codecs @brashkie/media-core
npm install @kryxjs/codecs @kryxjs/core
```

Actualiza tus imports:

```diff
- import { createDecoder, CodecRegistry } from '@brashkie/media-codecs'
- import { MediaError } from '@brashkie/media-core'
+ import { createDecoder, CodecRegistry } from '@kryxjs/codecs'
+ import { MediaError } from '@kryxjs/core'
```

Eso es todo. La API pública de TypeScript es **idéntica** — solo cambian los nombres de los paquetes.

📖 **[Guía completa de migración](https://github.com/Brashkie/kryx-codecs/blob/main/docs/MIGRATION.md)**

---

## ¿Cuál es la diferencia?

| Aspecto | `@brashkie/media-codecs@0.1.0` | `@kryxjs/codecs@0.1.0` |
|---------|-------------------------------|-----------------------|
| Estado | 🔴 Obsoleto | 🟢 Desarrollo activo |
| Repositorio | `Brashkie/media-codecs` | [`Brashkie/kryx-codecs`](https://github.com/Brashkie/kryx-codecs) |
| Depende de | `@brashkie/media-core@^0.1.4` | `@kryxjs/core@^0.1.0` |
| API TypeScript | idéntica | idéntica |
| Funcionalidades futuras | ❌ ninguna | ✅ todo el trabajo nuevo está aquí |

El paquete sucesor es parte del ecosistema más amplio **[Kryx](https://github.com/Brashkie/kryx-core)** — una alternativa modular a FFmpeg para Node.js, organizada bajo el scope `@kryxjs/*`.

---

## ¿Por qué el cambio de nombre?

El scope `@kryxjs/*` agrupa todos los paquetes del ecosistema Kryx juntos (`@kryxjs/core`, `@kryxjs/codecs`, `@kryxjs/codecs-opus`, etc.), haciendo el ecosistema más fácil de descubrir y mantener.

`@brashkie/media-codecs` era el nombre del prototipo. `@kryxjs/codecs` es el nombre de producción.

---

## Lo que NO cambió

- API pública de TypeScript (cada clase, función, tipo y firma)
- Traits `Codec` / `Decoder` / `Encoder`
- API del singleton global `CodecRegistry`
- Codecs PCM built-in (`pcm_s16le`, `pcm_s32le`, `pcm_f32le`, `pcm_f64le`)
- Jerarquía de `CodecError`
- Formato dual ESM + CJS
- Binarios nativos por plataforma (7 plataformas)
- Requerimiento de Node.js ≥18

---

## Estado

- `@brashkie/media-codecs@0.1.0` es la **última versión funcional** (ahora obsoleta).
- Aquí no se agregarán nuevas funcionalidades. Solo correcciones críticas de seguridad (si surgen) hasta fines de 2026.

---

## ¿Necesitas ayuda para migrar?

[Abre una discusión en el nuevo repositorio](https://github.com/Brashkie/kryx-codecs/discussions) o [reporta un issue](https://github.com/Brashkie/kryx-codecs/issues).

---

<div align="center">

**👉 Ve a [`@kryxjs/codecs`](https://www.npmjs.com/package/@kryxjs/codecs) 👈**

[English](README.md) · **Español**

</div>