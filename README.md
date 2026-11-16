<h1 align="center">siboot</h1>

<p align="center">
  <strong>English</strong> · <a href="README.zh-CN.md">简体中文</a>
</p>

<p align="center">
  Sign, install, and renew local IPA files on your iPhone or iPad.<br>
  A complete WASI installer for rootshell.
</p>

<p align="center">
  <a href="https://github.com/Jay-Void-BA/SideInstaller/releases/tag/siboot-v0.21.4"><img src="https://img.shields.io/badge/release-0.21.4-2563eb?style=flat-square" alt="siboot 0.21.4"></a>
  <a href="https://github.com/Jay-Void-BA/SideInstaller/actions/workflows/checks.yml"><img src="https://github.com/Jay-Void-BA/SideInstaller/actions/workflows/checks.yml/badge.svg?branch=main" alt="CI status"></a>
  <a href="#requirements"><img src="https://img.shields.io/badge/platform-rootshell%20%7C%20iOS%2027%2B-64748b?style=flat-square" alt="rootshell · iOS / iPadOS 27+"></a>
  <a href="https://github.com/Jay-Void-BA/SideInstaller/blob/main/LICENSE.md"><img src="https://img.shields.io/badge/license-Custom-64748b?style=flat-square" alt="SideInstaller custom license"></a>
</p>

<p align="center">
  <a href="https://github.com/Jay-Void-BA/SideInstaller/releases/tag/siboot-v0.21.4">Download</a> ·
  <a href="#quick-start">Quick start</a> ·
  <a href="#usage">Usage</a> ·
  <a href="#performance">Diagnostics</a> ·
  <a href="#migration">Migration</a> ·
  <a href="#development">Development</a>
</p>

<p align="center">
  Maintained by <a href="https://github.com/Jay-Void-BA">Jay-Void-BA</a> ·
  Original concept by <a href="https://github.com/FrizzleM">FrizzleM</a>
</p>

> [!IMPORTANT]
> **The iSH / Minis version is deprecated.** It no longer receives feature updates or binary releases. Existing users can follow the [migration guide](#migration) to move to rootshell. Historical source and releases remain available for reference.

## Features

| Feature | Description |
| --- | --- |
| Local installation | Pair your device, sign in to Apple, sign an IPA, and install it from rootshell |
| Complete signing | Recursively sign apps, extensions, frameworks, dylibs, and other Mach-O files; seal and verify resources such as `.config` files |
| Quick renewal | Keep the original IPA from the last installation and renew it with `renew` |
| Encrypted credentials | Protect account data, sessions, and signing keys with a separate local passphrase; reveal or hide passwords while typing |
| Step retries | Retry a failed step; installation retries reuse completed signing work |
| Performance logs | Save stage timings, network statistics, and file transfer measurements beside the program |
| Chinese and English | Chinese interface by default; use `--lang en` for English |

The installation workflow runs entirely in rootshell and `siboot.wasm`. Device connectivity requires LocalDevVPN. Bring your own local IPA; there is no SideInstaller IPA download option or automatic update check.

<a id="requirements"></a>

## Requirements

| Component | Requirement |
| --- | --- |
| System | iOS / iPadOS **27+**, with Developer Mode enabled |
| Terminal | [rootshell](https://github.com/kitknox/rootshell) **1.0.16 (178)** |
| Device connection | [LocalDevVPN](https://apps.apple.com/app/id6755608044) connected, with Wi-Fi enabled |
| Signing | Your own Apple Account, an internet connection, and a local IPA |

Apple authentication, anisette, and provisioning require internet access. The name rootshell does not imply iOS root access.

<a id="compatibility"></a>

## rootshell compatibility

**rootshell 1.0.16 (178) is the only supported target.** The installer uses its original socket/TLS, receive-timeout, terminal, and filesystem APIs. It does not require a host patch. Earlier builds and the old build-175 backport are no longer supported or included in releases.

CI pins the unmodified [1.0.16-178 source](https://github.com/kitknox/rootshell/tree/192eab95001aef3f2ea4b282c0d3f38a3539fb3f), verifies the Worker and broker hashes, and models its single root preopen and 64 KiB reply mailbox. Check the installed version in rootshell Settings before continuing.

<a id="quick-start"></a>
<a id="rootshell-版配置"></a>

## Getting started

### 1. Install the apps and configure permissions

Install rootshell **1.0.16 (178)** from the [App Store](https://apps.apple.com/app/rootshell-local-terminal-ssh/id6755794662) or [China App Store](https://apps.apple.com/app/rootshell-%E6%9C%AC%E5%9C%B0%E7%BB%88%E7%AB%AF-ssh/id6763402687), and install [LocalDevVPN](https://apps.apple.com/app/id6755608044).

1. Enable **Settings › Privacy & Security › Developer Mode**, then restart and confirm when prompted.
2. Enable Wi-Fi. Add the VPN configuration in LocalDevVPN and connect. Confirm that it shows **Connected**, and keep LocalDevVPN as the active VPN during installation and renewal.
3. Allow rootshell to access the **Local Network**. If you previously denied access, enable it under **Settings › Privacy & Security › Local Network › rootshell**.
4. Before creating a pairing, enable **Settings › Location Diary Mode** in rootshell, select **Session Only**, and grant location access when prompted. This keeps the program running while you switch to system settings. You can turn it off after pairing; enable it again when creating another pairing.
5. Allow rootshell notifications so you can read the six-digit pairing code while in Settings. The code also appears in the terminal.

### 2. Create a working directory

Open a **Local Shell** tab in rootshell. Run the following commands one at a time from its default starting directory, on the iPhone or iPad itself:

```sh
mkdir -p siboot
cd siboot
pwd
```

Copy your `App.ipa` into **Files › Browse › On My iPhone / iPad › rootshell › siboot**. Files in iCloud Drive, Downloads, or another app need to be copied into rootshell's accessible app directory first. Return to the `siboot` directory when opening a new terminal tab. Build 178 passes only the module filename in argv, so enter the program directory before launching it.

Create `siboot` only for the first setup. If the folder already exists, enter it with `cd siboot`. The `rootshell` folder in Files is created by the app.

### 3. Download and verify

From the working directory, download the version-pinned installer, then run it only if the download succeeded:

```sh
curl -fL --retry 3 -o install-rootshell.sh.part https://github.com/Jay-Void-BA/SideInstaller/releases/download/siboot-v0.21.4/install-rootshell.sh &&
mv install-rootshell.sh.part install-rootshell.sh &&
sh install-rootshell.sh
```

The script resumes this version's `.wasm.part` file, checks its **exact size and SHA-256**, and only then replaces `siboot.wasm`. Failed or corrupt downloads preserve the installed program and your pairing, login, and renewal data. Re-running the script skips a binary that already passes verification.

```sh
sh install-rootshell.sh --verify
./siboot.wasm --version
```

The output should include `siboot 0.21.4 (rootshell 1.0.16 / 178)`. If a partial file is corrupt or the server refuses resuming, run `sh install-rootshell.sh --restart` to fetch that version again. It replaces only the partial file until verification succeeds. A forced termination may leave `.siboot-download.lock`; stop all download scripts before removing that empty directory with `rmdir .siboot-download.lock`.

You can also download `siboot.wasm` and `install-rootshell.sh` in Safari and copy them into `rootshell/siboot`, then run `sh install-rootshell.sh --verify`. All release assets have hashes in [SHA256SUMS](https://github.com/Jay-Void-BA/SideInstaller/releases/download/siboot-v0.21.4/SHA256SUMS).

> [!TIP]
> Keep rootshell in the foreground while downloading. Run the verified WASM only after the installer or `--verify` succeeds. A WebAssembly data-section overflow happens before siboot can run; the downloader checks integrity outside the module.

<a id="first-install"></a>
<a id="首次配对与安装"></a>

### 4. Pair and install

Confirm that LocalDevVPN is connected and Location Diary Mode is set to Session Only. From the working directory, run:

```sh
ifconfig en0 > ifconfig.txt
./siboot.wasm pair
```

Open **Settings › Privacy & Security › Developer Mode**, tap **Pair with siboot**, and enter your device passcode followed by the six-digit code from the notification or terminal. Return to rootshell, confirm that pairing succeeded, and run:

```sh
./siboot.wasm --ipa App.ipa
```

Follow the prompts to sign in to your Apple Account and enter any verification code. Wait for signing, upload verification, and system installation to finish. Before launching the installed app for the first time, trust its developer under **Settings › General › VPN & Device Management**.

Quote filenames containing spaces, for example `--ipa "My App.ipa"`. Run `./siboot.wasm` for the interactive menu. On the first interactive launch, a short guide explains how to copy an IPA from Files into the rootshell folder; acknowledge it after copying. Cancelled guides appear again, while an explicit `--ipa` skips the guide. The picker accepts a full local path as well as a numbered selection. If you already have a pairing file, [import it](#migration) instead.

<details>
<summary>If the Wi-Fi address cannot be detected</summary>

Find your device's IPv4 address under **Settings › Wi-Fi › ⓘ beside the current network › IP Address**. Replace the example below with that address to pair without `ifconfig.txt`:

```sh
./siboot.wasm pair --wifi-ip 192.168.1.10
```

This address is used for a new pairing. Normal device connections use `10.7.0.1:49152` through LocalDevVPN; you usually do not need to change `--device-ip` or `--port`.

</details>

<details>
<summary>Files, pairing records, and cached data</summary>

Pairing, sign-in, and a successful installation create the corresponding files:

```text
siboot/
  siboot.wasm                Program
  App.ipa                    Your local IPA
  ifconfig.txt               Wi-Fi information for a new pairing
  rp_pairing_file.plist       Saved pairing record
  .siboot/                   Encrypted sign-in data and renewal cache
  siboot-perf-*.jsonl         Performance logs, when --perf-log is enabled
```

The default data directory is `.siboot/` beside the program. Keep it and the pairing record to reuse pairing, unlock saved credentials, and renew the last app. When upgrading, put the new `siboot.wasm` back in the same directory. Earlier versions could incorrectly put state at the sandbox root when launched from a subdirectory. Those files are preserved: use `--home /.siboot` to reuse that storage and, if needed, `--pairing-file /rp_pairing_file.plist` to import the old pairing. Keep the explicit `--home` on later runs until you move the storage.

</details>

<a id="usage"></a>

## Usage

| Command | Purpose |
| --- | --- |
| `./siboot.wasm --ipa App.ipa` | Sign and install a local IPA |
| `./siboot.wasm renew` | Renew the last installed app |
| `./siboot.wasm --perf-log --ipa App.ipa` | Install with performance logging |
| `./siboot.wasm pair` | Create a pairing record beside the program |
| `./siboot.wasm check` | Check the VPN connection to the device |
| `./siboot.wasm connect` | Verify saved pairing and the complete tunnel |
| `./siboot.wasm developer-mode` | Request that Developer Mode be shown |
| `./siboot.wasm developer-mode --turn-on` | Request a restart to enable Developer Mode after confirmation |
| `./siboot.wasm --lang en` | Use the English interface |
| `./siboot.wasm --help` | Show all options |

### Quick renewal

After a successful installation, siboot saves the app's **original IPA**. Run `./siboot.wasm renew`, or choose quick renewal from the startup menu, to obtain fresh provisioning, sign the complete app, and install it using the same developer team. Renewal still works after the source IPA is moved.

Apps signed with a free account typically expire after seven days. Renew before expiry. Renewal requires internet access, LocalDevVPN, your local vault passphrase, and any verification requested by Apple.

### Credentials and passwords

You can save your Apple / iCloud account during sign-in. Account passwords, sessions, signing keys, and the anisette cache are encrypted using **scrypt + XChaCha20-Poly1305** and unlocked with a separate local passphrase. The passphrase must contain at least 12 characters, must differ from your Apple password, and is never stored.

Press **Ctrl-V** in the terminal to reveal or hide a password while typing. Visible input is cleared on submission, cancellation, or end of input. Control characters in file names and remote messages are escaped before display. Passwords and vault passphrases preserve spaces and Unicode characters. The account menu lets you update a password, temporarily use another account, or forget saved credentials.

<a id="performance"></a>

## Performance diagnostics

Add `--perf-log` when investigating slow installation or renewal:

```sh
./siboot.wasm --perf-log --ipa App.ipa
./siboot.wasm renew --perf-log
```

Each run creates a separate `siboot-perf-<timestamp>-<sequence>.jsonl` file **beside `siboot.wasm`** and prints its location at startup and completion. Logging is off by default. Existing logs are kept, and `--home` does not change their location.

Logs include signing, network, per-file upload, and system installation timings. They exclude filenames, account names, passwords, verification codes, tokens, keys, URLs, and request or response bodies. Use `--verbose` alongside this option for error details in the terminal.

<details>
<summary>Log fields, statistics, and timing limits</summary>

Each line is a JSON event containing version information, timestamps, stages, retry numbers, and relevant measurements:

| Area | Measurements |
| --- | --- |
| Signing and local computation | scrypt unlock, RSA key generation, CSR, extraction, Mach-O signing, resource hashing, and seal verification timings |
| Network | HTTP requests, WebSocket handshake and receive timings, status codes, and bytes; host connection calls combine DNS, TCP, and TLS time |
| Send and receive statistics | Calls, bytes, cumulative and maximum duration, estimated P50 / P95 histogram upper bounds; cooperative polling timeouts are counted separately from actual I/O errors |
| Upload and installation | Entry numbers and sizes, open / chunk write / close timings, upload verification, and iOS installation progress |
| Other | Input waiting time, renewal cache timings, and WASM linear memory capacity |

The final `summary` event contains the outcome, operation statistics, and `log_write_us`, the measured serialization and write time. Files are identified by entry number.

Critical stages flush immediately; ongoing I/O flushes statistics roughly once per second. Detailed events stop at **16 MiB**, while final statistics and the `dropped_events` count are retained. Normal exits write a summary. Force-quitting may lose the last buffered events and summary.

Measurements are **wall-clock durations**. Nested or concurrent operations overlap and must not be added together. The WASI clock usually has about 1 ms resolution. Timing excludes loading the WASM module. Memory values describe linear memory capacity rather than physical process RAM. Logging adds collection and write overhead; compare runs with logging enabled and disabled.

</details>

<details>
<summary>Signing, uploads, and retries</summary>

- Sign the main executable, App Extensions, ExtensionKit extensions, frameworks, dylibs, and other Mach-O files recursively.
- Seal and verify `.config`, hidden files, and other resources. Configuration files are protected by the resource seal; they are not independently signed executables.
- Preserve IPA symbolic links through WASI file metadata, then recreate them on the device through AFC.
- Read at most 32 KiB per host call to fit the 64 KiB JSON/Base64 reply buffer, then combine reads into 1 MiB AFC upload packets; verify all file sizes and link targets before installation.
- Retry the current step after a failure. Installation retries reuse completed signing work and establish a new tunnel.

</details>

<a id="migration"></a>

## Migrating from iSH / Minis

The iSH / Minis version is deprecated and no longer receives feature updates or new `siboot-aarch64` builds. The [0.20.0 release](https://github.com/Jay-Void-BA/SideInstaller/releases/tag/siboot-v0.20.0) is an archive. Its Linux ARM64 executable cannot run in rootshell.

1. Follow [Getting started](#quick-start) to install the apps, configure permissions, and download `siboot.wasm`.
2. Use Files or the old app's sharing feature to export the **original IPA** and RPPairing record into `rootshell/siboot`. The apps have separate filesystems and cannot access each other's sandbox directly.
3. Open that directory in rootshell's local terminal, replace the filenames below with your own, and run:

   ```sh
   ./siboot.wasm --pairing-file from-ish.plist --ipa App.ipa
   ```

4. Sign in again and save credentials in rootshell. The first successful installation creates a new renewal cache; use `./siboot.wasm renew` afterward.

RPPairing plist, JSON, and known wrapper formats are supported. Traditional desktop lockdown pairing files cannot be used for this tunnel. If you do not have a usable record, [create a new pairing](#first-install).

Imports are verified against the device before a copy is saved as **`rp_pairing_file.plist` beside the program**. Failed imports preserve both the saved record and the source file. Pairing records contain device trust keys; keep them private.

## Troubleshooting

Local I/O uses reads of at most 32 KiB, buffered ZIP extraction with reused parent directories, and combined 1 MiB upload packets. The extraction log records `directory_checks_saved`. File selection caches entry types and batches terminal output; quick renewal validates one cache snapshot. These reduce host calls while preserving complete signature and upload verification. Device speed and iOS installation time still need measurement with `--perf-log`.

### WASM data section would overflow the module

This error occurs while rootshell parses the module, before siboot starts. The loaded bytes are shorter than a section declares. Run `sh install-rootshell.sh --verify`; if it fails, use `sh install-rootshell.sh --restart`. Do not execute or rename an unverified partial download. If verification succeeds but the parser still fails on 1.0.16 (178), report the file size, SHA-256, and complete message so host loading can be investigated separately.

### Installation takes a long time

siboot reports upload, file verification, and iOS installation timings separately. Extraction, hashing and signing, Apple and anisette requests, encrypted tunnels, many small-file round trips, and system `installd` work can all contribute. Running as WASM in rootshell does not guarantee a speedup over Minis; system installation time is controlled by iOS. Use [performance logs](#performance) to locate the slow stage.

### The pairing entry does not appear

Check that Wi-Fi is on, rootshell has Local Network permission, and Location Diary Mode is set to Session Only. Keep the program running while switching to Settings and allow a few seconds for discovery. After changing Wi-Fi networks, regenerate `ifconfig.txt` or pass the device's current `--wifi-ip` explicitly.

### The data directory is reported as busy

If a previous run was force-quit, close all siboot terminal tabs first. Confirm that no process is still using the directory, then run these commands one at a time:

```sh
rm .siboot/apple-login.active
./siboot.wasm
```

<a id="development"></a>

## Development

rootshell is the actively maintained user-facing target. The repository retains the original iOS app, deprecated iSH / Minis sources, and compatibility checks around the shared signing core. See the [source preparation guide](https://github.com/Jay-Void-BA/SideInstaller/blob/main/patches/siboot/README.md) for the layout.

<details>
<summary>Build from source and run checks</summary>

From the SideInstaller repository root:

```sh
bash scripts/prepare-minis.sh /tmp/siboot-source
# Install Node 24 and Bun 1.3.2 for the Worker and JavaScriptCore checks.
rustup target add wasm32-wasip1
cargo build --manifest-path /tmp/siboot-source/rootshell/Cargo.toml --release --locked --target wasm32-wasip1 --bins --examples
bash scripts/check-rootshell-runtime.sh /tmp/siboot-source/rootshell/target/wasm32-wasip1/release/siboot.wasm /tmp/siboot-source/rootshell/target/wasm32-wasip1/release/examples/runtime_check.wasm
```

The script name is retained for compatibility with existing development workflows. Keeping old source and tests does not imply continued support for the deprecated installation method.

</details>

CI covers shared Rust tests, rootshell tests, WASI builds, JavaScriptCore parsing via Bun, and the unmodified **1.0.16 (178)** Worker. It checks single-preopen subdirectory launches, the real mailbox limit, complete signing of large directories, HTTP / WebSocket, cooperative TCP, encrypted storage, renewal, and interrupted/corrupt downloads. Once checks pass, the release workflow publishes `siboot.wasm`, `install-rootshell.sh`, and `SHA256SUMS`, then marks the release as Latest.

**End-to-end installation with a real Apple Account and iPhone has not yet been verified.**

## Acknowledgements

| Author / project | Contribution |
| --- | --- |
| [Jay-Void-BA](https://github.com/Jay-Void-BA) | Development, maintenance, and rootshell support |
| [FrizzleM](https://github.com/FrizzleM) | Original ideas and projects: [siboot](https://github.com/FrizzleM/siboot) and [SideInstaller by FrizzleM](https://github.com/FrizzleM/SideInstaller); this implementation includes code from those projects |
| [kitknox/rootshell](https://github.com/kitknox/rootshell) | Terminal and WASM runtime |
| [jkcoxson/idevice](https://github.com/jkcoxson/idevice) | Device communication |
| [nab138/isideload](https://github.com/nab138/isideload) | Sideloading and signing workflows |
| [apple-codesign-quick](https://github.com/Dadoum/apple-crates) | Apple code signing |

Thanks to FrizzleM for the original design and groundwork. Both Jay-Void-BA and FrizzleM remain credited in the running program. Original copyright notices and applicable licenses are retained.

## License

See the [SideInstaller license](https://github.com/Jay-Void-BA/SideInstaller/blob/main/LICENSE.md) for usage and distribution terms. Vendored dependencies retain their own copyright notices and licenses.
