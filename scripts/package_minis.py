#!/usr/bin/env python3
"""Package a complete, static ARM64 ELF and its release-pinned installer."""

import hashlib
import os
import re
import shutil
import struct
import sys
from pathlib import Path


def check_elf(data):
    """Reject the wrong target, dynamic dependencies and truncated ELF ranges."""
    if len(data) < 64 or data[:7] != b"\x7fELF\x02\x01\x01":
        raise ValueError("Expected a complete ELF64 little-endian executable")
    kind, machine, version = struct.unpack_from("<HHI", data, 16)
    phoff, shoff = struct.unpack_from("<QQ", data, 32)
    ehsize, phsize, phnum, shsize, shnum = struct.unpack_from("<HHHHH", data, 52)
    if (kind, machine, version, ehsize, phsize) != (2, 183, 1, 64, 56) or not phnum:
        raise ValueError("Expected a static aarch64 Linux ET_EXEC executable")

    def bounded(offset, size):
        if offset > len(data) or size > len(data) - offset:
            raise ValueError("ELF range exceeds the file: truncated download/build")

    bounded(phoff, phsize * phnum)
    loads = 0
    for index in range(phnum):
        offset = phoff + index * phsize
        segment = struct.unpack_from("<I", data, offset)[0]
        start, size, memory = (struct.unpack_from("<Q", data, offset + n)[0] for n in (8, 32, 40))
        bounded(start, size)
        if segment in (2, 3):
            raise ValueError("Dynamic ELF is not supported in Minis")
        if segment == 1:
            if size > memory:
                raise ValueError("Invalid ELF load segment")
            loads += 1
    if not loads:
        raise ValueError("ELF has no loadable segment")
    if shnum:
        if shsize != 64:
            raise ValueError("Invalid ELF section table")
        bounded(shoff, shsize * shnum)
        for index in range(shnum):
            offset = shoff + index * shsize
            section = struct.unpack_from("<I", data, offset + 4)[0]
            if section != 8:  # SHT_NOBITS has no bytes in the file.
                start, size = struct.unpack_from("<QQ", data, offset + 24)
                bounded(start, size)


def package(source, binary, output, repository, tag):
    # Keep the source argument for the historical iOS release workflow.
    if not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repository):
        raise ValueError("Invalid repository")
    if not re.fullmatch(r"(?:siboot-)?v\d+\.\d+\.\d+(?:-[0-9A-Za-z.]+)?", tag):
        raise ValueError("Invalid release tag")
    data = binary.read_bytes()
    check_elf(data)
    sha = hashlib.sha256(data).hexdigest()
    text = Path(__file__).with_name("install-minis.sh.in").read_text()
    replacements = {
        "@VERSION@": tag.removeprefix("siboot-").removeprefix("v"),
        "@BASE@": f"https://github.com/{repository}/releases/download/{tag}",
        "@SHA256@": sha,
        "@SIZE@": str(len(data)),
    }
    for marker, value in replacements.items():
        if text.count(marker) != 1:
            raise ValueError(f"Unexpected installer template: {marker}")
        text = text.replace(marker, value)
    output.mkdir(parents=True, exist_ok=True)
    dest = output / "siboot-aarch64"
    shutil.copyfile(binary, dest)
    dest.chmod(0o755)
    script = output / "install-minis.sh"
    script.write_text(text)
    script.chmod(0o755)
    (output / "SHA256SUMS").write_text("".join(
        f"{hashlib.sha256(path.read_bytes()).hexdigest()}  {path.name}\n" for path in (dest, script)
    ))


if __name__ == "__main__":
    package(Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3]),
            os.environ["GITHUB_REPOSITORY"], os.environ["RELEASE_TAG"])
