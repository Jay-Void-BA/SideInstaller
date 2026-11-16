#!/usr/bin/env bash
# Gate releases on modern, unmodified legacy, and backported rootshell Workers.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WASM="${1:?Usage: check-rootshell-runtime.sh SIBOOT_WASM RUNTIME_CHECK_WASM}"
PROBE="${2:?Runtime probe is required}"
REV=f5adabe57ddb2959922e135e622fb9776641735f
SHA=dde022bfba42a6f71019b988b48d49997e92c5d088b201db99ff05ec46c1b2ba
LEGACY_REV=2428f29990ab1e1e18dd8bf06c40f9f8a5b11273
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

fetch_verified() {
  local revision="$1" relative="$2" expected="$3" destination="$4" local_source="${5:-}"
  mkdir -p "$(dirname "$destination")"
  if [[ -n "$local_source" ]]; then
    cp "$local_source" "$destination"
  else
    curl --fail --silent --show-error --location --retry 3 --connect-timeout 20 \
      --output "$destination" "https://raw.githubusercontent.com/kitknox/rootshell/$revision/$relative"
  fi
  python3 - "$destination" "$expected" <<'PY'
import hashlib, pathlib, sys
actual = hashlib.sha256(pathlib.Path(sys.argv[1]).read_bytes()).hexdigest()
if actual != sys.argv[2]:
    raise SystemExit(f'rootshell source checksum mismatch: {sys.argv[1]} ({actual})')
PY
}

fetch_verified "$REV" Resources/Wasm/wasm-worker.js "$SHA" "$WORK/modern-worker.js" "${ROOTSHELL_WORKER_JS:-}"

while read -r expected relative; do
  local_source=""
  if [[ -n "${ROOTSHELL_LEGACY_SOURCE_DIR:-}" ]]; then
    local_source="$ROOTSHELL_LEGACY_SOURCE_DIR/$relative"
  fi
  fetch_verified "$LEGACY_REV" "$relative" "$expected" "$WORK/legacy/$relative" "$local_source"
done <<'SOURCES'
1170da50213c080ec30de7c3f64df43b16e8bc5a0852e4d8744a67f4d8945281 Resources/Wasm/wasm-worker.js
6c29491734c6872aed4f1f36f4804378a4d4311be0e78bc8234af9e2615ef00c rootshell/Features/Wasm/WasmSocketBroker.swift
1154173e8d84cfec53d5bd5e3ed763b1395120ce9e1486678f086d61478a7bdf rootshell/Features/Wasm/WasmProcess.swift
af0642133d83f6b15f6bc16447840b6e9809d6001efd007ff4680cf75019440f rootshell/Features/Wasm/WasmFilesystemBroker.swift
SOURCES

node "$ROOT/scripts/rootshell-host-test.mjs" "$WORK/legacy/Resources/Wasm/wasm-worker.js" "$WASM" "$PROBE" legacy
node "$ROOT/scripts/rootshell-host-test.mjs" "$WORK/modern-worker.js" "$WASM" "$PROBE" modern

cp -R "$WORK/legacy" "$WORK/backport"
git -C "$WORK/backport" apply --check "$ROOT/patches/rootshell/recv-timeout-175.patch"
git -C "$WORK/backport" apply "$ROOT/patches/rootshell/recv-timeout-175.patch"
node "$ROOT/scripts/rootshell-host-test.mjs" "$WORK/backport/Resources/Wasm/wasm-worker.js" "$WASM" "$PROBE" backport
