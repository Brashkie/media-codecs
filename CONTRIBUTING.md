# Contributing to `@brashkie/media-codecs`

Thanks for considering a contribution. This document explains expectations and workflow.

---

## Code of Conduct

See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

---

## Quick start

```bash
git clone https://github.com/Brashkie/media-codecs.git
cd media-codecs

# Install Rust + Node 18+, then:
npm install
npm run build:debug
npm test
```

---

## What to work on

**In scope:**
- New built-in codecs (Opus, AAC, FLAC, ...) — but coordinate first via an issue
- Bug fixes, perf improvements
- More test coverage (target ≥95%)
- Documentation
- New examples

**Out of scope:**
- Container parsing → `@brashkie/media-containers`
- Streaming protocols → `@brashkie/media-stream`
- GPU compute → `@brashkie/media-gpu`
- AI models → `@brashkie/media-ai`

When in doubt: **open an issue first**.

---

## Adding a new codec

1. Add a module under `crates/codecs-core/src/` (e.g. `opus.rs`)
2. Implement `Codec` + `Decoder` and/or `Encoder` traits
3. Create a `CodecDescriptor` and a `register_all(reg)` helper
4. Wire registration in `crates/codecs-core/src/lib.rs` (or `registry.rs`)
5. Add unit tests inline (`#[cfg(test)] mod tests { ... }`)
6. Add the codec name to `PcmCodecName`-style enum in `src/types.ts` if relevant
7. Update `CHANGELOG.md` under `[Unreleased]`

---

## Coding standards

### Rust
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace --exclude codecs-node`
- Public items have `///` doc comments
- `unsafe` blocks have a `// SAFETY:` comment

### TypeScript
- `prettier` + `eslint`
- Strict mode (already configured)
- Every public symbol has JSDoc
- Avoid `any` — use `unknown`

### Tests
- Every bug fix → regression test
- Every new feature → unit + integration test
- Target: **≥95% coverage on changed files**

---

## Commit conventions

[Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>
```

Types: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `chore`, `ci`.

---

## PR checklist

- [ ] `npm test` passes
- [ ] `npm run typecheck` passes
- [ ] `npm run lint` passes
- [ ] `npm run clippy` passes
- [ ] New APIs have doc comments
- [ ] New APIs have tests
- [ ] `CHANGELOG.md` updated

---

## License

By contributing you agree your contributions are licensed under [Apache-2.0](LICENSE).
