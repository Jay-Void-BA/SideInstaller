#!/usr/bin/env python3
"""Resolve version and notes for a manual or checks-triggered release."""

import json
import os
import re
import subprocess
import sys
from pathlib import Path


def configuration(root, env, source_sha):
    manifest = json.loads((root / ".github/release.json").read_text())
    version = env.get("VERSION_INPUT", "").strip().removeprefix("v").removeprefix("V") or manifest["version"]
    if not re.fullmatch(r"\d+\.\d+\.\d+(?:-[0-9A-Za-z]+(?:\.[0-9A-Za-z]+)*)?", version):
        raise ValueError("Version must be X.Y.Z or X.Y.Z-beta.N")
    channel = env.get("CHANNEL_INPUT") or manifest["channel"]
    if channel not in {"stable", "beta"}:
        raise ValueError("Channel must be stable or beta")
    if channel == "stable" and "-" in version:
        raise ValueError("A stable release cannot have a prerelease version")
    if not re.fullmatch(r"[0-9a-f]{40}", source_sha):
        raise ValueError("The source must be a full commit SHA")
    notes = env.get("CHANGELOG_INPUT", "")
    if notes:
        # Match the existing workflow's support for typed literal newlines.
        notes = notes.replace("\\n", "\n")
    else:
        if version != manifest["version"]:
            raise ValueError("A manual version different from the manifest needs release notes")
        notes_path = (root / manifest["notes"]).resolve()
        if not notes_path.is_relative_to(root.resolve()):
            raise ValueError("Release notes must be inside the repository")
        notes = notes_path.read_text()
    if not notes.strip():
        raise ValueError("Release notes cannot be empty")
    dry_run = env.get("DRY_RUN_INPUT", "false") or "false"
    if dry_run not in {"true", "false"}:
        raise ValueError("dry_run must be true or false")
    return {
        "version": version, "marketing_version": version.split("-")[0],
        "tag": "v" + version, "channel": channel,
        "dry_run": dry_run == "true", "source_sha": source_sha,
    }, notes.strip() + "\n"


def main():
    root = Path(__file__).resolve().parent.parent
    source_sha = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip()
    metadata, notes = configuration(root, os.environ, source_sha)
    dest = Path(sys.argv[1])
    dest.mkdir(parents=True, exist_ok=True)
    (dest / "release.json").write_text(json.dumps(metadata, indent=2) + "\n")
    (dest / "release-notes.md").write_text(notes)
    if output := os.environ.get("GITHUB_OUTPUT"):
        with open(output, "a") as out:
            for key, value in metadata.items():
                if isinstance(value, bool):
                    value = str(value).lower()
                out.write(f"{key}={value}\n")
    print(json.dumps(metadata))


if __name__ == "__main__":
    main()
