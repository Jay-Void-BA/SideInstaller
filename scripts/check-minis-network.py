#!/usr/bin/env python3
"""Exercise the real CLI with no system CAs, a private CA, and a local TLS server."""
import http.server
import json
import os
from pathlib import Path
import shutil
import ssl
import subprocess
import sys
import tempfile
import threading


def check(binary):
    with tempfile.TemporaryDirectory(prefix="siboot-tls-") as directory:
        root = Path(directory)
        (root / "empty-ca").mkdir()
        (root / "empty.pem").write_text("")
        program = root / "siboot"
        shutil.copyfile(binary, program)
        program.chmod(0o700)
        cert, key = root / "test-ca.pem", root / "test-ca-key.pem"
        subprocess.run([
            "openssl", "req", "-x509", "-newkey", "rsa:2048", "-nodes", "-days", "1",
            "-subj", "/CN=siboot local TLS test CA", "-addext", "basicConstraints=critical,CA:TRUE",
            "-keyout", str(key), "-out", str(cert),
        ], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        leaf, leaf_key, csr = root / "server.pem", root / "server-key.pem", root / "server.csr"
        subprocess.run([
            "openssl", "req", "-new", "-newkey", "rsa:2048", "-nodes", "-subj", "/CN=127.0.0.1",
            "-keyout", str(leaf_key), "-out", str(csr),
        ], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        extensions = root / "server.ext"
        extensions.write_text("basicConstraints=critical,CA:FALSE\nkeyUsage=critical,digitalSignature,keyEncipherment\nextendedKeyUsage=serverAuth\nsubjectAltName=IP:127.0.0.1\n")
        subprocess.run([
            "openssl", "x509", "-req", "-in", str(csr), "-CA", str(cert), "-CAkey", str(key),
            "-CAcreateserial", "-days", "1", "-out", str(leaf), "-extfile", str(extensions),
        ], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        requests = []

        class Handler(http.server.BaseHTTPRequestHandler):
            def do_GET(self):
                requests.append(self.path)
                body = json.dumps(dict(client_info="<MacBookPro18,3> <macOS;26.6;25F84>",
                                       user_agent="AuthKit/1")).encode()
                self.send_response(self.server.reply_status)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)

            def log_message(self, *args):
                pass

        server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        server.reply_status = 200
        context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        context.load_cert_chain(leaf, leaf_key)
        server.socket = context.wrap_socket(server.socket, server_side=True)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        env = {**os.environ, "SSL_CERT_FILE": str(root / "empty.pem"),
               "SSL_CERT_DIR": str(root / "empty-ca"), "SIBOOT_APPLE_ID": "unused@example.invalid",
               "SIBOOT_PASSWORD": "not-used-by-a-network-check"}

        def run(*extra):
            return subprocess.run([
                str(program), "--check-anisette", "--terminal", "--plain", "--lang", "en",
                "--home", str(root / "no-login-state"),
                "--anisette", f"https://127.0.0.1:{server.server_port}", *extra,
            ], env=env, input="", capture_output=True, text=True, timeout=30)

        try:
            rejected = run()
            details = (rejected.stdout + rejected.stderr).lower()
            assert rejected.returncode != 0, rejected.stdout
            assert "certificate" in details or "unknownissuer" in details, details
            assert "no ca certificates were loaded" not in details, details
            assert not requests, requests
            # System/private roots must keep working alongside the bundled public roots.
            env["SSL_CERT_FILE"] = str(cert)
            trusted = run("--perf-log")
            assert trusted.returncode == 0, trusted.stdout + trusted.stderr
            assert "reachable: 1/1" in trusted.stdout, trusted.stdout
            server.reply_status = 503
            unavailable = run()
            assert unavailable.returncode != 0
            assert "503" in unavailable.stdout + unavailable.stderr
            assert requests == ["/v3/client_info", "/v3/client_info"], requests
            assert not (root / "no-login-state").exists(), "a public probe must not open login storage"
            logs = list(root.glob("siboot-perf-*.jsonl"))
            assert len(logs) == 1, logs
            text = logs[0].read_text()
            assert not any(value in text for value in ("unused@example.invalid", env["SIBOOT_PASSWORD"], str(root)))
            rows = [json.loads(line) for line in text.splitlines()]
            assert rows[0]["command"] == "anisette_check"
            assert rows[-1]["outcome"] == "ok"
            assert "anisette.client_info" in rows[-1]["metrics"]
        finally:
            server.shutdown()
            server.server_close()
            thread.join()
    print("PASS missing system CAs, untrusted TLS rejection, private CA trust, HTTP errors and credential-free ani diagnostics")


if __name__ == "__main__":
    check(Path(sys.argv[1]).resolve())
