#!/usr/bin/env bash
# Run the release artifact in the iSH revision pinned by OpenMinis.
set -euo pipefail

BINARY="${1:?Usage: check-minis-runtime.sh SIBOOT_BINARY VERSION}"
VERSION="${2:?Expected version required}"
[[ "$(uname -s)" == Darwin && "$(uname -m)" == arm64 ]] || {
  echo 'This compatibility check requires an Apple ARM64 runner.' >&2
  exit 1
}
BINARY="$(python3 -c 'import pathlib,sys; print(pathlib.Path(sys.argv[1]).resolve(strict=True))' "$BINARY")"
for package in meson ninja pkgconf llvm lld; do
  brew list --versions "$package" >/dev/null 2>&1 || brew install "$package"
done
export PATH="$(brew --prefix lld)/bin:$PATH"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/siboot-runtime.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT
git init -q "$WORK/ish"
git -C "$WORK/ish" remote add origin https://github.com/OpenMinis/ish-arm64.git
git -C "$WORK/ish" fetch --depth 1 origin 0fd17a135e78b7a0cf08a4f5831217bbb76ed22f
git -C "$WORK/ish" checkout --detach FETCH_HEAD
meson setup "$WORK/build" "$WORK/ish" -Dguest_arch=arm64 --buildtype=release
ninja -C "$WORK/build" -j4 ish
mkdir -p "$WORK/root/bin" "$WORK/root/dev" "$WORK/root/proc" "$WORK/root/tmp" "$WORK/root/root"
cp "$BINARY" "$WORK/root/bin/siboot"
chmod 700 "$WORK/root/bin/siboot"
python3 - "$WORK/build/ish" "$WORK/root" "$VERSION" <<'PY'
import subprocess, sys
engine, root, version = sys.argv[1:]
for option in ['--version', '--help']:
    result = subprocess.run([engine, '-r', root, '/bin/siboot', option],
                            capture_output=True, text=True, timeout=60)
    print(result.stdout)
    if result.returncode:
        print(result.stderr[-20000:], file=sys.stderr)
        raise SystemExit(f'iSH {option} exited {result.returncode}')
    assert 'Jay-Void-BA' in result.stdout
    assert version in result.stdout
    if option == '--help':
        assert '--ipa PATH' in result.stdout and '本地 IPA' in result.stdout
    print(f'iSH ARM64 {option}: PASS')
PY
