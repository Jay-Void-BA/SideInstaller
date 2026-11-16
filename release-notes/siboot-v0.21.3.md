# siboot 0.21.3 — first launch, reliable storage, and bounded I/O

Maintained by Jay-Void-BA. Original code and design: FrizzleM/siboot and SideInstaller by FrizzleM.

- Add a Chinese/English first-launch guide for copying an IPA through Files into the rootshell folder. Confirmed guides appear once; cancellation keeps the guide available. Explicit `--ipa` skips it.
- Reuse existing storage and temporary directories even when rootshell reports `EIO` for an existing directory. Preserve conflicting regular files and report real failures.
- Fit file and socket reads into the actual 64 KiB JSON/Base64 reply mailbox: read at most 32 KiB per host call, buffer extraction writes, and aggregate 1 MiB AFC upload packets.
- Cache file-picker entry types, batch terminal output, and validate one renewal snapshot before network access and sign-in.
- Bound pairing-file wrappers and plist container depth, vault reads, and upload paths. Escape terminal controls in displayed names and messages. Clear revealed password input on EOF and restore terminal echo.
- Give each upload its own staging parent and clean up that parent after success or failure. Retain complete recursive signing and per-file upload verification.
- Exercise legacy, modern, and backported Workers with the real mailbox size and mkdir error behavior, large executable/resource files, directory pagination, malformed archives, repeated launches, password EOF, and picker host-call bounds.

Download `siboot.wasm` and verify it against `SHA256SUMS`. Keep the existing data and pairing files beside the program when upgrading.

The 0.21.2 compatibility boundary remains: unmodified older rootshell hosts can start, pair, and check TCP reachability; full installation/renewal requires receive-timeout support. Update rootshell or rebuild build 175 with the optional included host patch. Real Apple Account-to-iPhone installation and a full Swift build of that backport have not been validated here; no device speedup is claimed without measurement.
