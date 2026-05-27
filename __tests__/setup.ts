/**
 * Global vitest setup.
 *
 * Previously this file used `Module._load` interception to mock the native
 * addon. That worked when the loader used a runtime `require()` (which
 * `_load` can intercept). Now that we use `import * as native from '../index.js'`
 * (static ESM import), Vitest can't intercept anymore — but we don't need
 * to, because the real `.node` binary is loaded directly.
 *
 * For tests that need to mock specific behavior, use `vi.mock()` per test.
 */

// No-op setup. Real native addon is loaded by src/native.ts at module
// evaluation time.
