// Bun uses JavaScriptCore. Check the parser family used by WKWebView as well
// as the Node/V8 Worker gate; this still does not replace testing on iOS.
import fs from 'node:fs';
import assert from 'node:assert/strict';

assert.equal(typeof Bun, 'object', 'run this check with Bun / JavaScriptCore');
for (const file of process.argv.slice(2)) {
  const bytes = fs.readFileSync(file);
  new WebAssembly.Module(bytes);
  assert.throws(() => new WebAssembly.Module(bytes.subarray(0, -1)), /overflow|size|end|length|bytes/i);
  console.log(`PASS JavaScriptCore parses complete WASM and rejects truncation (${bytes.length} bytes)`);
}
assert(process.argv.length > 2, 'usage: bun check-wasm-jsc.mjs SIBOOT_WASM [PROBE_WASM]');
