"""Exercise the real generated script with curl and a local HTTP server."""
import hashlib
import http.server
import importlib.util
import os
from pathlib import Path
import subprocess
import tempfile
import threading
import unittest

SCRIPTS = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("package_rootshell", SCRIPTS / "package_rootshell.py")
package = importlib.util.module_from_spec(spec)
spec.loader.exec_module(package)


class PackageTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.binary = self.root / "input.wasm"
        # A valid empty module with a custom section: enough bytes to resume.
        payload = b"\0" + bytes(range(251)) * 500
        size = len(payload)
        leb = bytearray()
        while size > 127:
            leb.append((size & 127) | 128)
            size >>= 7
        leb.append(size)
        self.data = b"\0asm\x01\0\0\0\0" + leb + payload
        self.binary.write_bytes(self.data)
        self.assets = self.root / "assets"
        package.package(self.binary, self.assets, "Jay-Void-BA/SideInstaller", "siboot-v0.21.4")
        self.work = self.root / "work"
        self.work.mkdir()
        self.script = self.assets / "install-rootshell.sh"
        self.requests = []
        self.corrupt = False
        self.interrupt = False
        owner = self

        class Handler(http.server.BaseHTTPRequestHandler):
            def do_GET(self):
                start = int(self.headers.get("Range", "bytes=0-")[6:-1])
                owner.requests.append(start)
                body = (b"x" * len(owner.data) if owner.corrupt else owner.data)[start:]
                self.send_response(206 if start else 200)
                if start:
                    self.send_header("Content-Range", f"bytes {start}-{len(owner.data)-1}/{len(owner.data)}")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                if owner.interrupt:
                    self.wfile.write(body[:len(body)//2])
                    self.close_connection = True
                else:
                    self.wfile.write(body)

            def log_message(self, *args):
                pass

        self.server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()
        self.addCleanup(self.close_server)
        self.env = {**os.environ, "SIBOOT_BASE": f"http://127.0.0.1:{self.server.server_port}"}

    def close_server(self):
        self.server.shutdown()
        self.server.server_close()
        self.thread.join()

    def run_script(self, *args):
        return subprocess.run(["sh", str(self.script), *args], cwd=self.work,
                              env=self.env, capture_output=True, text=True, timeout=30)

    def test_interrupted_download_preserves_program_then_resumes_and_verifies(self):
        installed = self.work / "siboot.wasm"
        installed.write_bytes(b"previous working program")
        self.interrupt = True
        result = self.run_script()
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(installed.read_bytes(), b"previous working program")
        partial = self.work / "siboot-v0.21.4.wasm.part"
        offset = partial.stat().st_size
        self.assertGreater(offset, 0)
        self.interrupt = False
        result = self.run_script()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.requests[-1], offset)
        self.assertEqual(installed.read_bytes(), self.data)
        self.assertFalse(partial.exists())
        self.assertFalse((self.work / ".siboot-download.lock").exists())
        self.assertEqual(self.run_script("--verify").returncode, 0)
        requests = len(self.requests)
        self.assertEqual(self.run_script().returncode, 0)
        self.assertEqual(len(self.requests), requests, "valid installation must not download again")

    def test_corruption_requires_restart_and_never_replaces_the_old_program(self):
        installed = self.work / "siboot.wasm"
        installed.write_bytes(b"keep me")
        self.corrupt = True
        self.assertNotEqual(self.run_script().returncode, 0)
        self.assertEqual(installed.read_bytes(), b"keep me")
        self.assertNotEqual(self.run_script("--verify").returncode, 0)
        self.corrupt = False
        self.assertEqual(self.run_script("--restart").returncode, 0)
        self.assertEqual(installed.read_bytes(), self.data)

    def test_links_and_concurrent_downloads_are_refused(self):
        secret = self.work / "private"
        secret.write_bytes(b"must not change")
        partial = self.work / "siboot-v0.21.4.wasm.part"
        partial.symlink_to(secret)
        self.assertNotEqual(self.run_script().returncode, 0)
        self.assertEqual(secret.read_bytes(), b"must not change")
        partial.unlink()
        (self.work / ".siboot-download.lock").mkdir()
        self.assertNotEqual(self.run_script().returncode, 0)
        self.assertEqual(self.requests, [])

    def test_package_rejects_section_overflow_and_records_both_digests(self):
        for suffix in [b"\x0b\x81", b"\x0b\x10abc", b"\0\xff\xff\xff\xff\x7f"]:
            with self.assertRaises(ValueError):
                package.check_sections(b"\0asm\x01\0\0\0" + suffix)
        self.binary.write_bytes(self.data[:-1])
        with self.assertRaises(ValueError):
            package.package(self.binary, self.assets, "Jay-Void-BA/SideInstaller", "siboot-v0.21.4")
        self.assertEqual((self.assets / "siboot.wasm").read_bytes(), self.data)
        for line in (self.assets / "SHA256SUMS").read_text().splitlines():
            digest, name = line.split()
            self.assertEqual(hashlib.sha256((self.assets / name).read_bytes()).hexdigest(), digest)
        self.assertIn(str(len(self.data)), self.script.read_text())


if __name__ == "__main__":
    unittest.main()
