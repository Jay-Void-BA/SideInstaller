import hashlib
import importlib.util
import json
import tempfile
import unittest
from pathlib import Path


SCRIPTS = Path(__file__).resolve().parents[1]


def load(name):
    spec = importlib.util.spec_from_file_location(name, SCRIPTS / (name + ".py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


release = load("release_config")
installer = load("package_minis")


class ReleaseConfigTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        (self.root / ".github").mkdir()
        (self.root / ".github/release.json").write_text(json.dumps({
            "version": "2.1.0", "channel": "stable", "notes": "notes.md",
        }))
        (self.root / "notes.md").write_text("Local IPA installs and retries.\n")
        self.sha = "a" * 40

    def test_automatic_release_uses_manifest_and_exact_tested_commit(self):
        meta, notes = release.configuration(self.root, {}, self.sha)
        self.assertEqual(meta["tag"], "v2.1.0")
        self.assertEqual(meta["source_sha"], self.sha)
        self.assertFalse(meta["dry_run"])
        self.assertIn("Local IPA", notes)

    def test_manual_beta_preserves_notes_without_shell_interpretation(self):
        meta, notes = release.configuration(self.root, {
            "VERSION_INPUT": "v2.2.0-beta.1", "CHANNEL_INPUT": "beta",
            "CHANGELOG_INPUT": r"## Notes\n`$(not-a-command)` stays literal",
            "DRY_RUN_INPUT": "true",
        }, self.sha)
        self.assertEqual(meta["marketing_version"], "2.2.0")
        self.assertTrue(meta["dry_run"])
        self.assertEqual(notes, "## Notes\n`$(not-a-command)` stays literal\n")

    def test_invalid_versions_channels_and_unreviewed_commits_fail(self):
        for env, sha in [
            ({"VERSION_INPUT": "2.1.0; touch file"}, self.sha),
            ({"VERSION_INPUT": "2.2.0"}, self.sha),
            ({"VERSION_INPUT": "2.1.0-beta.1", "CHANNEL_INPUT": "stable"}, self.sha),
            ({"CHANNEL_INPUT": "unknown"}, self.sha),
            ({"DRY_RUN_INPUT": "yes"}, self.sha),
            ({}, "main"),
        ]:
            with self.subTest(env=env, sha=sha), self.assertRaises(ValueError):
                release.configuration(self.root, env, sha)

    def test_installer_downloads_and_verifies_the_release_binary(self):
        source = self.root / "source"
        source.mkdir()
        (source / "install.sh").write_text('''#!/bin/sh
BASE="original"
SHA256="0000"
SIZE=1
fetch "$BASE/bin/siboot-aarch64" "$BIN.part"
fetch "$BASE/bin/siboot-aarch64" "$BIN.part"
''')
        binary = self.root / "binary"
        binary.write_bytes(b"release executable")
        out = self.root / "assets"
        installer.package(source, binary, out, "Jay-Void-BA/SideInstaller", "v2.1.0")
        script = (out / "install-minis.sh").read_text()
        self.assertIn("https://github.com/Jay-Void-BA/SideInstaller/releases/download/v2.1.0", script)
        self.assertIn(hashlib.sha256(binary.read_bytes()).hexdigest(), script)
        self.assertIn(f"SIZE={binary.stat().st_size}", script)
        self.assertEqual(script.count('$BASE/siboot-aarch64'), 2)
        self.assertEqual((out / "siboot-aarch64").read_bytes(), binary.read_bytes())


if __name__ == "__main__":
    unittest.main()
