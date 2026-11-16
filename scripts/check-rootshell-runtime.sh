#!/usr/bin/env bash
# Gate on the unmodified rootshell 1.0.16 (178) host, pinned by commit and hashes.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WASM="${1:?Usage: check-rootshell-runtime.sh SIBOOT_WASM RUNTIME_CHECK_WASM}"
PROBE="${2:?Runtime probe is required}"
REV=192eab95001aef3f2ea4b282c0d3f38a3539fb3f
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

while read -r expected relative; do
  destination="$WORK/$relative"
  mkdir -p "$(dirname "$destination")"
  if [[ -n "${ROOTSHELL_178_SOURCE_DIR:-}" ]]; then
    cp "$ROOTSHELL_178_SOURCE_DIR/$relative" "$destination"
  else
    curl --fail --silent --show-error --location --retry 3 --connect-timeout 20 \
      --output "$destination" "https://raw.githubusercontent.com/kitknox/rootshell/$REV/$relative"
  fi
  python3 - "$destination" "$expected" <<'PYHASH'
import hashlib, pathlib, sys
actual = hashlib.sha256(pathlib.Path(sys.argv[1]).read_bytes()).hexdigest()
if actual != sys.argv[2]:
    raise SystemExit(f'rootshell 178 source checksum mismatch: {sys.argv[1]} ({actual})')
PYHASH
done <<'SOURCES'
dde022bfba42a6f71019b988b48d49997e92c5d088b201db99ff05ec46c1b2ba Resources/Wasm/wasm-worker.js
4644f873a3d23600d8ebcd9f546b847d0f5503e26c5c07fdd2ec035401335d51 Resources/Wasm/wasm-host.js
d988c23bb034cf41f0b40c21d258cf40193e91269fa9ee27ea9f3f2ab1c4c9e9 rootshell/Features/Wasm/WasmRuntime.swift
a335bb61cf36c0c11e376423f948167e996921c0afaee2fa403fb385d10d1419 rootshell/Features/Wasm/WasmSocketBroker.swift
2067fc2aa0cc85c452a43f6a69e597bced43ac4e5e8182af3413b7ea35c026a4 rootshell/Features/Wasm/WasmFilesystemBroker.swift
489e3f442924b9a500bf6376c426994fa834720c2128bcd4a31651af13766fab rootshell/Features/Wasm/WasmProcess.swift
SOURCES

node "$ROOT/scripts/rootshell-host-test.mjs" "$WORK/Resources/Wasm/wasm-worker.js" "$WASM" "$PROBE" rootshell-178
"${BUN:-bun}" "$ROOT/scripts/check-wasm-jsc.mjs" "$WASM" "$PROBE"
