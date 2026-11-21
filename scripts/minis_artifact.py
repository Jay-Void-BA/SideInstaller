#!/usr/bin/env python3
"""Bind an already-tested ARM64 artifact to its source and release version."""
import hashlib
import json
import os
from pathlib import Path
import re
import sys
import tomllib

from package_minis import check_elf


def inspect(folder):
    data = (folder / "siboot-aarch64").read_bytes()
    check_elf(data)
    expected = {path.name: hashlib.sha256(path.read_bytes()).hexdigest()
                for path in (folder / "siboot-aarch64", folder / "install-minis.sh")}
    sums = dict((name, digest) for digest, name in
                (line.split() for line in (folder / "SHA256SUMS").read_text().splitlines()))
    if sums != expected:
        raise ValueError("Artifact digests do not match SHA256SUMS")
    return expected


def main():
    mode, directory, *args = sys.argv[1:]
    folder = Path(directory)
    version = json.loads(Path('.github/siboot-release.json').read_text())["version"]
    source = os.environ["SOURCE_SHA"]
    if not re.fullmatch(r"[0-9a-f]{40}", source):
        raise ValueError("Expected the full tested source commit")
    expected = dict(source_sha=source, version=version,
                    target="aarch64-unknown-linux-musl", sha256=inspect(folder))
    path = folder / "build-info.json"
    if mode == "record":
        if tomllib.loads(Path(args[0]).read_text())["package"]["version"] != version:
            raise ValueError("Minis Cargo version and release manifest differ")
        path.write_text(json.dumps(expected, indent=2) + "\n")
    elif mode == "verify":
        if json.loads(path.read_text()) != expected:
            raise ValueError("Artifact is from a different source, version or target")
    else:
        raise ValueError("Expected record or verify")
    print(json.dumps(expected))


if __name__ == "__main__":
    main()
