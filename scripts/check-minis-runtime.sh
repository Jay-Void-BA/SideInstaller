#!/usr/bin/env bash
# Execute the actual release binary on an ARM64 Linux runner; no Apple account.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BINARY="$(realpath "${1:?Usage: check-minis-runtime.sh BINARY OFFLINE_PROBE}")"
PROBE="$(realpath "${2:?Offline probe is required}")"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
mkdir "$WORK/bin"
cp "$BINARY" "$WORK/bin/siboot"
chmod +x "$WORK/bin/siboot" "$PROBE"
python3 - "$ROOT" "$BINARY" <<'PY'
import pathlib, sys
sys.path.insert(0, str(pathlib.Path(sys.argv[1]) / 'scripts'))
from package_minis import check_elf
check_elf(pathlib.Path(sys.argv[2]).read_bytes())
PY
SIBOOT="$WORK/bin/siboot"
"$SIBOOT" --version | tee "$WORK/version.txt"
grep -q 'Jay-Void-BA' "$WORK/version.txt"
grep -q 'FrizzleM' "$WORK/version.txt"
"$SIBOOT" --lang en --help > "$WORK/help.txt"
grep -q -- '--perf-log' "$WORK/help.txt"
grep -q -- '--jobs' "$WORK/help.txt"
"$SIBOOT" --lang zh-CN --help > "$WORK/help-zh.txt"
if timeout 30 "$SIBOOT" --terminal --plain --lang en --home "$WORK/state" \
    --ipa "$WORK/missing.ipa" --perf-log < /dev/null > "$WORK/invalid.txt" 2>&1; then
    echo 'Invalid IPA unexpectedly succeeded' >&2
    exit 1
fi
grep -q 'IPA' "$WORK/invalid.txt"
if grep -q 'Waiting for Minis\|Starting pairing\|Signing in to Apple' "$WORK/invalid.txt"; then
    echo 'Invalid IPA reached a remote step' >&2
    exit 1
fi
python3 "$ROOT/scripts/make_signing_fixture.py" "$WORK/fixture" --large-directory
timeout 120 "$PROBE" "$WORK/fixture" | tee "$WORK/probe.txt"
grep -q 'NATIVE_OFFLINE_PROBE_OK' "$WORK/probe.txt"
python3 - "$WORK" <<'PY'
import json, pathlib, stat, sys
root = pathlib.Path(sys.argv[1])
logs = list(root.rglob('siboot-perf-*.jsonl'))
assert len(logs) == 2, logs
for path in logs:
    assert stat.S_IMODE(path.stat().st_mode) == 0o600
    text = path.read_text()
    assert not any(secret in text for secret in ('offline@example.invalid', 'test-only-password',
                  'different local passphrase', str(root), 'original-pairing', '-----BEGIN'))
    rows = [json.loads(line) for line in text.splitlines()]
    assert rows[0]['event'] == 'run_start'
    assert rows[-1]['event'] == 'summary'
    assert rows[-1]['outcome'] == ('ok' if path.parent.name == 'fixture' else 'error')
    for row in rows:
        assert all(isinstance(value, int) for value in row.get('data', {}).values())
    if path.parent.name == 'fixture':
        for operation in ('ipa.extract', 'sign.bundle', 'vault.scrypt', 'renew.remember'):
            assert any(row.get('operation') == operation or operation in row.get('metrics', {})
                       for row in rows), operation
print('PASS static ARM64 startup, complete CMS/resource signing, links, vault, renewal and private logs')
PY
