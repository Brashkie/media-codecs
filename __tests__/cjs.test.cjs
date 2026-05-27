'use strict'

// Verifies that the published CJS bundle resolves and exposes the public API.

const assert = require('node:assert')
const m = require('../dist/index.cjs')

assert.ok(typeof m.VERSION === 'string', 'VERSION should be string')
assert.ok(typeof m.createDecoder === 'function', 'createDecoder export')
assert.ok(typeof m.createEncoder === 'function', 'createEncoder export')
assert.ok(typeof m.CodecRegistry === 'function', 'CodecRegistry export')
assert.ok(m.SampleFormat.S16 === 's16', 'SampleFormat.S16')
assert.ok(m.CodecKind.Audio === 'audio', 'CodecKind.Audio')

console.log('✓ CJS smoke test passed (version=' + m.VERSION + ')')
