#!/usr/bin/env python3
"""Package a tested WASM with a version-pinned, integrity-checking downloader."""

import hashlib
import os
import re
import shutil
import sys
from pathlib import Path


def check_sections(data):
    """Reject a truncated module before creating a release or its installer.

    The actual Worker validates instructions/imports separately. Here every
    section's declared byte range must fit, including the final data section.
    """
    if data[:8] != b"\0asm\x01\0\0\0":
        raise ValueError("Not a WASM version-1 module")
    offset = 8
    while offset < len(data):
        offset += 1  # Section id, validated by WebAssembly.Module in the gate.
        length = 0
        for shift in range(0, 35, 7):
            if offset >= len(data):
                raise ValueError("Truncated WASM section length")
            byte = data[offset]
            offset += 1
            length |= (byte & 127) << shift
            if byte < 128:
                if shift == 28 and byte > 15:
                    raise ValueError("Invalid WASM section length")
                break
        else:
            raise ValueError("Invalid WASM section length")
        if length > len(data) - offset:
            raise ValueError(f"Truncated WASM: section at {offset} requires {length} bytes")
        offset += length


def package(binary, output, repository, tag):
    if not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repository):
        raise ValueError("Invalid repository")
    if not re.fullmatch(r"siboot-v[0-9]+\.[0-9]+\.[0-9]+", tag):
        raise ValueError("Invalid release tag")
    data = binary.read_bytes()
    check_sections(data)
    digest = hashlib.sha256(data).hexdigest()
    template = Path(__file__).with_name("install-rootshell.sh.in").read_text()
    replacements = {
        "@VERSION@": tag.removeprefix("siboot-v"),
        "@BASE@": f"https://github.com/{repository}/releases/download/{tag}",
        "@SHA256@": digest,
        "@SIZE@": str(len(data)),
    }
    for token, value in replacements.items():
        if template.count(token) != 1:
            raise ValueError(f"Unexpected installer template token: {token}")
        template = template.replace(token, value)
    output.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(binary, output / "siboot.wasm")
    installer = output / "install-rootshell.sh"
    installer.write_text(template)
    installer.chmod(0o755)
    manifest = "".join(
        f"{hashlib.sha256((output / name).read_bytes()).hexdigest()}  {name}\n"
        for name in ("siboot.wasm", "install-rootshell.sh")
    )
    (output / "SHA256SUMS").write_text(manifest)
    return digest


if __name__ == "__main__":
    package(Path(sys.argv[1]), Path(sys.argv[2]),
            os.environ["GITHUB_REPOSITORY"], os.environ["RELEASE_TAG"])
