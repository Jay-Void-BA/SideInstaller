#!/usr/bin/env bash
# Prepare the pinned Minis source with our patch and this checkout's Rust core.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEST="${1:?Usage: prepare-minis.sh NEW_DIRECTORY}"
SIBOOT_REV=82b808de3a407619be7fbe54b3f313932bb80ade

if [ -e "$DEST" ]; then
  echo "error: $DEST already exists; choose a new directory" >&2
  exit 1
fi
git clone --no-checkout "${SIBOOT_REPOSITORY:-https://github.com/FrizzleM/siboot.git}" "$DEST"
git -C "$DEST" checkout --detach "$SIBOOT_REV"
git -C "$DEST" apply --check "$ROOT/patches/siboot/retry-local-ipa.patch"
git -C "$DEST" apply "$ROOT/patches/siboot/retry-local-ipa.patch"
# A submodule is an empty directory until initialized. Use the code being
# checked/released here, so its vendored dependencies have the same fixes.
rmdir "$DEST/SideInstaller"
ln -s "$ROOT" "$DEST/SideInstaller"
echo "Prepared $DEST/minis from $SIBOOT_REV"
