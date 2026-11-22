<h1 align="center">siboot</h1>
<p align="center">Sign, install, and renew local IPA files in Minis.</p>
<p align="center"><strong>English</strong> · <a href="README.zh-CN.md">简体中文</a></p>
<p align="center">
  <a href="https://github.com/Jay-Void-BA/SideInstaller/releases/tag/siboot-v0.22.2"><img src="https://img.shields.io/badge/release-0.22.2-blue" alt="Release"></a>
  <a href="https://github.com/Jay-Void-BA/SideInstaller/actions/workflows/checks.yml"><img src="https://github.com/Jay-Void-BA/SideInstaller/actions/workflows/checks.yml/badge.svg?branch=main" alt="Checks"></a>
  <a href="LICENSE.md"><img src="https://img.shields.io/badge/license-Custom-lightgrey" alt="License"></a>
</p>
<p align="center">Maintained by <a href="https://github.com/Jay-Void-BA">Jay-Void-BA</a> · Original concept and code by <a href="https://github.com/FrizzleM">FrizzleM</a></p>

siboot runs the complete installation workflow on your iPhone or iPad: import a local IPA, pair the device, sign in to Apple, sign every executable, and install the app. The runtime is **Minis + `siboot-aarch64`**; device connectivity also requires LocalDevVPN.

> [!IMPORTANT]
> **Minis ARM64 is the supported platform. rootshell is deprecated.** Version 0.22.0 returns to a static Linux ARM64 executable. No rootshell app, WASM module, or pairing relay is required. Ordinary x86 iSH cannot run this executable.

## Features

- **Complete signing.** Recursively sign the app, extensions, frameworks, dylibs and other Mach-O files. Seal hidden resources, including `.config`, preserve valid symlinks, and verify resource seals before upload.
- **Local IPA installation.** Select an IPA from Files in the Minis page, or supply a Linux path in the terminal. No bundled SideInstaller IPA download or automatic update check.
- **Quick renewal.** Reuse the original IPA, saved pairing and encrypted account cache with `renew`. Renewal signs the complete app again.
- **Encrypted login.** Protect the Apple Account, sessions and signing keys with an independent local passphrase. Reveal or hide passwords in the page and live terminal.
- **Step retries.** Retry a failed step. An installation retry keeps the completed signed bundle available.
- **Diagnostics.** Opt in to detailed performance logs beside the executable. English and Simplified Chinese interfaces are included.

## Requirements

| Component | Requirement |
| --- | --- |
| Device | iPhone or iPad on iOS / iPadOS 27+, with Developer Mode enabled |
| Runtime | Open Minis 1.16+ with an ARM64 Linux environment (`uname -m` reports `aarch64`) |
| Connection | [LocalDevVPN](https://apps.apple.com/app/id6755608044), connected; Wi-Fi enabled |
| Account | Apple Account with access to its verification methods |
| App | A local IPA and enough free space for the original IPA, extracted app and signing work |

## Quick start

### 1. Configure Minis and the device

Install Open Minis and LocalDevVPN on the device itself. In **Settings › Privacy & Security › Developer Mode**, enable Developer Mode, restart, and confirm if prompted. Enable Wi-Fi and connect LocalDevVPN. Keep it as the active VPN throughout installation and renewal.

Allow Minis to access the **Local Network** when requested. Keep Minis in the foreground during downloads, imports, signing and installation. Pairing uses Minis's Bonjour bridge and audio keepalive while you briefly switch to Settings; allow the permissions Minis requests. Run `siboot check` if pairing does not appear.

### 2. Download and verify

In the **Minis Linux terminal**, run:

```sh
uname -m
apk add --no-cache curl ca-certificates
curl -fL --retry 3 -o install-minis.sh.part https://github.com/Jay-Void-BA/SideInstaller/releases/download/siboot-v0.22.2/install-minis.sh &&
mv install-minis.sh.part install-minis.sh &&
sh install-minis.sh
```

The installer puts the program at **`$HOME/.siboot/bin/siboot`**. It checks the exact file size and SHA-256 before replacing an existing program. An interrupted download keeps the partial file for the next run; your installed program and app data are preserved.

```sh
sh install-minis.sh --verify
"$HOME/.siboot/bin/siboot" --version
```

The output must include `siboot 0.22.2`, `Jay-Void-BA` and `FrizzleM`. All asset digests are in [SHA256SUMS](https://github.com/Jay-Void-BA/SideInstaller/releases/download/siboot-v0.22.2/SHA256SUMS). If a partial download is corrupt or the server refuses resuming, use `sh install-minis.sh --restart`. The script never launches the program automatically.

For a manual download, save `siboot-aarch64` and `install-minis.sh` together, then verify before running:

```sh
sh install-minis.sh --verify ./siboot-aarch64
chmod +x ./siboot-aarch64
./siboot-aarch64 --version
```

### 3. Import, pair and install

```sh
"$HOME/.siboot/bin/siboot" --web
```

Open the local page offered by Minis. On the first interactive installation, the import guide explains how to select an IPA. Choose the file from **Files**; download an iCloud item to the device before selecting it. The page transfers the file into Minis's Linux filesystem. Keep Minis open until import completes.

Choose a saved pairing, import an existing **RPPairing** file, or pair automatically. For a new pairing, follow the prompt to open **Settings › Privacy & Security › Developer Mode › Pair with siboot**, enter the device passcode and the pairing code, then return to Minis. No second terminal app is involved.

Sign in to Apple when prompted. If you save the login, choose a **separate local vault passphrase**; it is required to unlock the saved login and is never saved. Use the password visibility button on the page, or **Ctrl-V** in the live terminal, to show or hide input. `--plain` uses hidden password input.

For terminal installation, first import the IPA into Minis and supply its Linux path:

```sh
"$HOME/.siboot/bin/siboot" --terminal --ipa /home/App.ipa
```

Replace the example path with the actual path inside Minis. A path from rootshell, another iOS app or iCloud Drive is not directly a Minis Linux path.

## Usage

Use `"$HOME/.siboot/bin/siboot"` below, or the path where you saved the executable.

| Command | Action |
| --- | --- |
| `siboot` | Select a local IPA and run the installation workflow |
| `siboot --ipa /home/App.ipa` | Install a local IPA already inside Minis |
| `siboot renew` | Sign and reinstall the last successfully cached app |
| `siboot check` | Check device connectivity and Minis pairing support |
| `siboot connect` | Verify pairing and open the device tunnel |
| `siboot --pairing-file /home/phone.plist` | Verify and import an RPPairing file |
| `siboot --lang en` / `siboot --lang zh-CN` | Choose English / Simplified Chinese |
| `siboot --terminal` / `siboot --web` | Choose the live terminal / Minis page |
| `siboot --check-anisette` | Diagnose default ani endpoints without IPA, Apple login or VPN |
| `siboot --perf-log` | Write detailed performance measurements |
| `siboot --jobs 2` | Use two signing workers; supported range is 1–4 |
| `siboot --help` | Show all options |

The interface defaults to Chinese; `--lang en` selects English. A free Apple account usually needs renewal within seven days. Renewal still needs internet, LocalDevVPN and any verification Apple requests. It reuses the original IPA without another import or cache copy; it performs complete signing and upload verification. Use a normal IPA installation to change app or developer team.

## Performance and diagnostics

Minis executes Linux code under emulation. Full signing hashes executable pages and resources; AFC performs many file operations over the local device tunnel; iOS then validates and installs the bundle. The installation screen reports upload, file verification and iOS installation times separately. iOS installation time is controlled by the device.

```sh
"$HOME/.siboot/bin/siboot" renew --perf-log
```

Each run writes **`siboot-perf-*.jsonl` beside the actual executable**, including when launched from another directory. With the default installer, this is `$HOME/.siboot/bin/`. Logs contain stage timings, retries, secret-input wait time, extraction counts, hashing/signing durations, upload bytes and file operations, verification, iOS installation progress and a final summary. They accept only code-defined operation names and numeric measurements, without passwords, account names, paths, pairing data or protocol bodies. Detailed records are capped at 16 MiB; normal exit flushes a summary.

Start with **one signing worker**, the default under emulation. Compare the same IPA with `--jobs 2` if needed; more workers can be slower or use more memory. Version 0.22.0 buffers extraction writes, caches directory creation, isolates signing workspaces, moves CPU-heavy signing off the async device workers, and avoids copying the renewal IPA again. Complete resource sealing and per-file upload checks stay enabled. These changes are tested offline; real-device speed depends on the app, Minis and iOS. No device benchmark is claimed.

The ordinary error report is saved under `$HOME/.siboot/siboot.log`. Review it before sharing; the separate performance log has stricter data limits.

## Anisette connectivity

Version 0.22.1 includes Mozilla public CA certificates for the standalone installer, while retaining system/private CA trust. A minimal Minis filesystem with no CA bundle previously let the Apple preflight pass using its embedded Apple root but failed to construct every anisette HTTP client.

```sh
"$HOME/.siboot/bin/siboot" --check-anisette --terminal
"$HOME/.siboot/bin/siboot" --check-anisette --anisette https://ani.iloader.app --terminal
```

The check probes `/v3/client_info` with at most three concurrent requests and a 12-second limit per server. It prints each endpoint's DNS, TLS, timeout or HTTP error, without opening the login vault, pairing, provisioning an identity or signing in to Apple. A successful check proves metadata connectivity; provisioning and Apple authentication can still fail separately. `--perf-log` records numeric timings for these probes.

For older versions or other tools in Minis, restore the system bundle with `apk add --no-cache ca-certificates` and `update-ca-certificates`. When a custom proxy uses its own CA, install that CA in Minis as well; iOS's trust store is separate. A server-specific HTTP 429 is tried on another endpoint; an Apple account rate limit still stops login attempts.

## Apple sign-in compatibility

Version 0.22.2 incorporates [FrizzleM's sign-in fix](https://github.com/FrizzleM/SideInstaller/commit/8c13fceefebe8fec7475555a9ea50a7df0c372d8) for Apple's November 20 change. The built-in macOS-ADI client and the Apple fallback now report the compatible Catalina `Mac OS X` identity, replacing the `macOS;26.6` / `macOS;27.0` identities associated with HTTP 503. Existing modern-Mac anisette identities remain usable; no login or pairing data needs to be deleted.

The default fallback list includes `anisette.lillq.me`. When another server cannot use that pool's identity, the installer tries a compatible stored identity or provisions a separate one. Failed sign-in setup, HTTP status errors and Apple authentication codes are shown distinctly; HTTP failures from the Apple URL-bag request are checked before XML parsing.

**Check the program you actually run.** Download this release's new install script, then run `"$HOME/.siboot/bin/siboot" --version`. It must show **0.22.2**. A command such as `./siboot-aarch64` may still launch a separately downloaded 0.20.0 file. Each install script is pinned to one release. Upgrading preserves the default encrypted data directory; pairing imports remain available if you move the executable.

A log that says login succeeded and then reports **developer error 1100 while signing** records an expired developer session, rather than a failed password check. Restart the current program and sign in again; do not delete the vault or signing keys. The old **Directory not empty** retry failure used a shared extraction directory and is addressed by private per-attempt workspaces in 0.22.0 and later. A successful `--check-anisette` only proves the metadata endpoint is reachable; it does not test Apple authentication.

## Files and migration

| Location | Contents |
| --- | --- |
| Beside the actual executable | `rp_pairing_file.plist` and opt-in performance logs |
| `$HOME/.siboot/` (or `--home PATH`) | Encrypted login/signing cache, original IPA for renewal, and error report |
| Private temporary directory per signing run | Extracted signed bundle; removed when its last owner finishes |

Pairing imports are verified with the device before a private copy replaces the saved record. A failed import preserves the source and previous pairing. The page can upload a pairing file from Files or browse Minis's Linux filesystem. Keep pairing records private: they contain device trust keys.

**From rootshell:** export your original IPA and RPPairing file through Files, import them into Minis, and sign in again. The applications have separate sandboxes; old paths and credentials are not migrated automatically. **From an older Minis release:** the default data directory remains `$HOME/.siboot`. Keep the existing pairing beside the new executable, or import it with `--pairing-file`. Do not delete your data directory to upgrade.

rootshell source and historical releases, including 0.21.4, remain available for reference. They no longer receive runtime support or new WASM releases.

## Troubleshooting

- **`segmentation fault` on startup:** verify the exact file size and SHA-256 first. Earlier reports involved interrupted downloads. Resume with the generated installer; use `--restart` if verification still fails. Check `uname -m` is `aarch64`.
- **No pairing entry in Settings:** keep LocalDevVPN connected and Wi-Fi enabled, allow Local Network for Minis, and use Open Minis 1.16+. Run `siboot check` or import a working RPPairing record.
- **Download lock remains after force-closing:** stop every installer, then remove only the empty `$HOME/.siboot/bin/.siboot-download.lock` with `rmdir` and retry.
- **Slow installation:** use `--perf-log` and compare upload, verification and iOS installation separately. Keep Minis in the foreground and start with `--jobs 1`.

## Development

```sh
bash scripts/prepare-minis.sh /tmp/siboot-source
cargo test --manifest-path /tmp/siboot-source/minis/Cargo.toml --locked --all-targets
cargo clippy --manifest-path /tmp/siboot-source/minis/Cargo.toml --locked --all-targets -- -D warnings
LOCKED=1 bash scripts/test-rust.sh
python3 -m unittest discover -s scripts/tests -v
```

The preparation script applies the pinned source patch and uses this repository's shared Rust core. CI builds a static `aarch64-unknown-linux-musl` release with Rust 1.100 on an ARM64 runner, executes the CLI and complete offline signing probe, and checks encrypted storage, symlinks and resource tampering. Packaging rejects truncated or dynamic ELF files. Publication consumes that exact successful CI artifact and validates its source commit and digests. iOS source checks remain separate; no Apple login or on-device installation is simulated by the offline probe.

## Credits and license

Maintained by **Jay-Void-BA**. Original concept and implementation: **[siboot by FrizzleM](https://github.com/FrizzleM/siboot)**. Shared signing and device code derives from **[SideInstaller by FrizzleM](https://github.com/FrizzleM/SideInstaller)** and its credited dependencies.

The inherited SideInstaller code remains under [LICENSE.md](LICENSE.md); dependency licenses also apply. This repository contains inherited implementation and is **not a clean-room rewrite**, so it is not relabeled as MIT. See the license for its terms.
