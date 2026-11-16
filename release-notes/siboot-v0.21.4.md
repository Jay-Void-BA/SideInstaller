# siboot 0.21.4 — rootshell 1.0.16 (178), verified downloads

Maintained by Jay-Void-BA. Original code and design: FrizzleM/siboot and SideInstaller by FrizzleM.

- Support only **rootshell 1.0.16 (178)**, using its original host. Remove the old optional ABI fallback and build-175 backport asset. Pin the exact upstream tag commit and verify six host-source hashes.
- Add `install-rootshell.sh`: resume a version-specific partial download, verify its exact size and SHA-256, then replace `siboot.wasm`. Interruptions and corrupt bytes preserve the installed program and its data. `--verify` checks the local binary; `--restart` starts a fresh partial download.
- Reject malformed/truncated WASM sections during packaging. Test real interrupted HTTP transfers, resumed ranges, hash mismatches, repeated installation, symlinks, and concurrent-download exclusion.
- Validate complete and truncated modules in JavaScriptCore through Bun as well as the actual Worker in Node/V8.
- Resolve WASI paths using the host virtual `PWD`, including relative IPA, pairing, state and Wi-Fi files; Rust current_dir remains `/` on Preview 1.
- Model build 178's actual single root preopen, test launches from a subdirectory, and retain the real 64 KiB mailbox and bounded reads. Recheck first-launch Files guidance, directory reuse, full CMS signing and resource seals, pairing/vault limits, password cleanup, renewal, HTTP/WebSocket, and cooperative TCP.
- Reuse verified parent directories within each private extraction tree to avoid repeated WebKit metadata calls; report `directory_checks_saved` in performance logs. Keep complete resource sealing and upload verification.
- Rewrite English/Chinese setup and recovery instructions around 178 and the verified installer. Fix remaining rootshell UI wording.

Download `install-rootshell.sh` and run `sh install-rootshell.sh` in your working directory. Keep rootshell in the foreground. The matching binary and script hashes are in `SHA256SUMS`; the script also embeds the binary's expected byte count and hash. Re-running it does not contact the network when the installed binary already verifies.

The reported 0.21.3 error (`Data section ... would overflow Module's size`) points at its final data section: offset 4,600,000 plus 717,374 bytes requires a complete 5,317,374-byte module. That failure occurs before siboot code runs and means the loader received incomplete module bytes. Validate the local file first; if a verified binary still fails on 178, the host-loading path needs device evidence.

The original Files import guide, existing directories, encrypted credentials, complete recursive signing, private upload staging, and performance logs are retained. Real Apple Account-to-iPhone end-to-end installation and device performance still require real-device validation.
