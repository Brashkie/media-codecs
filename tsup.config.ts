import { defineConfig } from 'tsup'

/**
 * tsup config for @brashkie/media-codecs
 *
 * Pattern proven by @brashkie/signalis-core and @brashkie/media-core@0.1.4:
 *   - No `shims: true` (it generates a broken __require helper).
 *   - The native addon loader (src/native.ts) uses a STATIC
 *     `import * as native from '../index.js'`. tsup leaves that as a literal
 *     `require()`/`import` because `'../index.js'` is external.
 *   - Per-format DTS files (.d.cts / .d.mts) come from scripts/fix-dts.js.
 *   - `noExternal` FORCES our internal modules to be bundled. Without this,
 *     tsup sometimes auto-externalizes modules imported from multiple entry
 *     points (we don't have multiple entries, but tsup's heuristics still
 *     trigger). The symptom: dist/index.cjs contains `require('./native')`
 *     that fails at runtime because no dist/native.cjs exists.
 */
export default defineConfig({
  entry: ['src/index.ts'],
  format: ['cjs', 'esm'],
  dts: true,
  splitting: false,
  sourcemap: true,
  clean: true,
  minify: false,
  target: 'node18',
  outDir: 'dist',
  bundle: true,
  // External = "do NOT bundle these, leave the import statement as-is".
  external: [
    // napi-rs loader — must stay outside the bundle to find the .node binary
    '../index.js',
    '../index.cjs',
    // Peer package — don't duplicate it into our tarball
    '@brashkie/media-core',
  ],
  // FORCE-bundle all internal sources (override tsup's auto-externalization).
  // This list intentionally matches everything under src/ except the entry.
  noExternal: [
    /^\.\/native$/,
    /^\.\/registry$/,
    /^\.\/codec$/,
    /^\.\/error$/,
    /^\.\/types$/,
  ],
  // Explicit extensions: .cjs → CommonJS, .mjs → ESM
  outExtension({ format }) {
    return {
      js: format === 'cjs' ? '.cjs' : '.mjs',
    }
  },
})