// Verifies that the published ESM bundle resolves and exposes the public API.

import assert from 'node:assert'
import * as m from '../dist/index.mjs'

assert.ok(typeof m.VERSION === 'string', 'VERSION should be string')
assert.ok(typeof m.createDecoder === 'function', 'createDecoder export')
assert.ok(typeof m.createEncoder === 'function', 'createEncoder export')
assert.ok(typeof m.CodecRegistry === 'function', 'CodecRegistry export')
assert.ok(m.SampleFormat.S16 === 's16', 'SampleFormat.S16')
assert.ok(m.CodecKind.Audio === 'audio', 'CodecKind.Audio')

console.log('✓ ESM smoke test passed (version=' + m.VERSION + ')')
