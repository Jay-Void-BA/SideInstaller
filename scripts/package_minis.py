#!/usr/bin/env python3
"""Package an already-built Minis executable with its matching install script."""

import hashlib
import os
import re
import shutil
import sys
from pathlib import Path


def package(source, binary, output, repository, tag):
    if not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repository):
        raise ValueError("Invalid repository")
    if not re.fullmatch(r"v\d+\.\d+\.\d+(?:-[0-9A-Za-z.]+)?", tag):
        raise ValueError("Invalid release tag")
    output.mkdir(parents=True, exist_ok=True)
    dest = output / "siboot-aarch64"
    shutil.copyfile(binary, dest)
    dest.chmod(0o755)
    with dest.open("rb") as executable:
        sha = hashlib.file_digest(executable, "sha256").hexdigest()
    text = (source / "install.sh").read_text()
    replacements = {
        r'^BASE=.*$': f'BASE="${{SIBOOT_BASE:-https://github.com/{repository}/releases/download/{tag}}}"',
        r'^SHA256="[0-9a-f]+"$': f'SHA256="{sha}"',
        r'^SIZE=\d+$': f'SIZE={dest.stat().st_size}',
    }
    for pattern, value in replacements.items():
        text, count = re.subn(pattern, lambda _: value, text, flags=re.MULTILINE)
        if count != 1:
            raise ValueError(f"Install script no longer matches {pattern}")
    old = '$BASE/bin/siboot-aarch64'
    if text.count(old) != 2:
        raise ValueError("Unexpected install script download commands")
    text = text.replace(old, '$BASE/siboot-aarch64')
    script = output / "install-minis.sh"
    script.write_text(text)
    script.chmod(0o755)


if __name__ == "__main__":
    package(Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3]),
            os.environ["GITHUB_REPOSITORY"], os.environ["RELEASE_TAG"])
