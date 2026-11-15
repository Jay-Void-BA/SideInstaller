#!/usr/bin/env bash
# Gate the release using rootshell's actual, revision-pinned WASI Worker.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WASM="${1:?Usage: check-rootshell-runtime.sh SIBOOT_WASM RUNTIME_CHECK_WASM}"
PROBE="${2:?Runtime probe is required}"
REV=f5adabe57ddb2959922e135e622fb9776641735f
SHA=dde022bfba42a6f71019b988b48d49997e92c5d088b201db99ff05ec46c1b2ba
WORKER="$(mktemp)"
trap 'rm -f "$WORKER"' EXIT
if [[ -n "${ROOTSHELL_WORKER_JS:-}" ]]; then
  cp "$ROOTSHELL_WORKER_JS" "$WORKER"
else
  curl --fail --location --retry 3 --output "$WORKER" \
    "https://raw.githubusercontent.com/kitknox/rootshell/$REV/Resources/Wasm/wasm-worker.js"
fi
actual="$(python3 - "$WORKER" <<'PY'
import hashlib, pathlib, sys
print(hashlib.sha256(pathlib.Path(sys.argv[1]).read_bytes()).hexdigest())
PY
)"
[[ "$actual" == "$SHA" ]] || { echo 'rootshell Worker checksum mismatch' >&2; exit 1; }
node "$ROOT/scripts/rootshell-host-test.mjs" "$WORKER" "$WASM" "$PROBE"
