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



if __name__ == "__main__":
    unittest.main()
