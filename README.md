# siboot

Sign, install, and renew local iPhone and iPad apps from Open Minis.

**English** · [简体中文](README.zh-CN.md)

[Latest release](https://github.com/Jay-Void-BA/SideInstaller/releases/latest) · [Changelog](CHANGELOG.md) · [CI](https://github.com/Jay-Void-BA/SideInstaller/actions/workflows/checks.yml)

siboot takes a local IPA, pairs with the device, signs with your Apple developer team, and installs the result. The Minis page can import from Files, verify an existing pairing record, and renew the last installed app. The terminal offers the same signing and installation workflow.

> [!IMPORTANT]
> Use the ARM64 release with Open Minis. rootshell support ended with 0.21.4. To bring over an IPA or a pairing record, follow the [migration instructions](#data-and-migration).

[Requirements](#requirements) · [Install siboot](#install-siboot) · [Install an IPA](#install-an-ipa) · [Apple sign-in](#apple-sign-in) · [Renew](#renew-an-app) · [Troubleshooting](#troubleshooting) · [Development](#development)

## Requirements

| Component | What you need |
| --- | --- |
| Device | iPhone or iPad running iOS or iPadOS 27 or later. Turn on Developer Mode to run development-signed apps. |
| Runtime | Open Minis with ARM64 Linux. Run `uname -m` in its terminal and look for `aarch64`. Open Minis 1.16 or later supports pairing on the device. Older Minis builds require an existing compatible pairing record. |
| Connection | Wi-Fi enabled and LocalDevVPN connected on the device you are installing to. Internet access is needed for Apple sign-in and provisioning. |
| Account | An Apple Account with permission to sign for a developer team and access to its Apple verification methods. A Personal Team can sign apps for personal testing. |
| IPA and storage | A complete local IPA you can re-sign. Allow space for the downloaded IPA, the extracted bundle, and the renewal copy. Download items from iCloud Drive to the device before importing. |

Get [Open Minis](https://apps.apple.com/us/app/open-minis/id6759188481) and [LocalDevVPN](https://apps.apple.com/app/id6755608044) from the App Store.

1. Enable Wi-Fi, connect LocalDevVPN, and allow Open Minis to access the local network if iOS asks. Keep LocalDevVPN connected during pairing, installation, and renewal.
2. In **Settings → Privacy & Security → Developer Mode**, turn on Developer Mode, restart, and confirm after unlocking. If the switch is missing, complete pairing and run the `developer-mode` command to request that iOS show it. [Apple's Developer Mode guide](https://developer.apple.com/documentation/xcode/enabling-developer-mode-on-a-device)
3. Keep Open Minis open while a download, import, signature, or installation is in progress. The pairing flow tells you when to switch to Settings and return.

If your Apple Account uses iCloud Advanced Data Protection, start with [the sign-in instructions](#icloud-advanced-data-protection). They cover the updated login and when to try switching to standard data protection.

## Install siboot

In the **Open Minis Linux terminal**, check the architecture and install the download tools if needed.

```sh
uname -m
apk add --no-cache curl ca-certificates
```

Use `siboot-aarch64` when the terminal reports `aarch64`. Standard x86 iSH and the old rootshell WASM host cannot run this file.

Download and run the installer for the latest published release.

```sh
curl -fL --retry 3 -o install-minis.sh.part \
  https://github.com/Jay-Void-BA/SideInstaller/releases/latest/download/install-minis.sh &&
  mv install-minis.sh.part install-minis.sh &&
  sh install-minis.sh
```

The installer writes `$HOME/.siboot/bin/siboot`. It checks the exact byte count and SHA-256 before replacing the installed executable. If the transfer is interrupted, run `sh install-minis.sh` again to resume. If verification reports a corrupt partial file, run `sh install-minis.sh --restart` to start that download over.

Verify and launch the installed version explicitly.

```sh
sh install-minis.sh --verify &&
  "$HOME/.siboot/bin/siboot" --version &&
  "$HOME/.siboot/bin/siboot" --web
```

The version output should match the downloaded release and include `Jay-Void-BA` and `FrizzleM`. Each installer script is pinned to one release. Download the new script when upgrading and use the installed path above. A manually downloaded `./siboot-aarch64` is a separate copy.

<details>
<summary>Download and verify the executable manually</summary>

Download `siboot-aarch64` and `install-minis.sh` from the same release into the same Minis directory. Verify the binary before giving it execute permission.

```sh
sh install-minis.sh --verify ./siboot-aarch64 &&
  chmod 700 ./siboot-aarch64 &&
  ./siboot-aarch64 --version
```

If you also downloaded `SHA256SUMS`, run `sha256sum -c SHA256SUMS` from that directory to check both files. When running a manual copy, use that executable's directory for its pairing file and performance logs. [Release files](https://github.com/Jay-Void-BA/SideInstaller/releases/latest)

</details>

## Install an IPA

### Choose the IPA

Run `"$HOME/.siboot/bin/siboot" --web` and open the page offered by Minis. Choose **Install a local IPA**, then select it in **Files**. Files sends the archive to Minis's local Linux storage. The Files upload accepts an IPA up to 4 GiB. The archive needs a single app in `Payload`. If the import is interrupted, select the original file again.

To use a file already inside the Linux environment, quote the actual path. This also works for paths with spaces.

```sh
"$HOME/.siboot/bin/siboot" --terminal --ipa "/home/My App.ipa"
```

Use the Files picker for archives in iCloud Drive or another iOS app. Use `--ipa` when you have already copied the archive into the Minis Linux filesystem.

### Pair with the device

At the **Pairing** step, reuse the saved record, import an existing RPPairing file, or create a new one.

- **Pair in Minis.** Choose **Start pairing**, leave the silence player open, and go to **Settings → Privacy & Security → Developer Mode**. Scroll down and tap **Pair with siboot**. Enter the device passcode and the six-digit *pairing* code displayed by Minis. Return to Minis when Settings confirms pairing. If Minis notifications are off, read the pairing code in Minis.
- **Import a record.** In the page, choose a file from **Files** or browse the Minis Linux filesystem. From the terminal, run the command below. Use an RPPairing record valid for this device. After the device accepts it, siboot saves a private copy as `rp_pairing_file.plist` beside the actual executable. A failed import preserves the previously saved record.

```sh
"$HOME/.siboot/bin/siboot" connect --pairing-file "/home/phone.plist"
```

`connect` stops after pairing and tunnel verification. Run siboot again with your IPA when you are ready to install. The pairing code belongs in iOS Settings. Apple sign-in may ask for a separate verification code later.

### Sign and open the app

After **Connect**, sign in to your Apple Account and complete any requested verification. siboot registers the device with the selected developer team when needed and obtains signing assets. It signs nested executables, including extensions, frameworks, and dylibs, and seals resources such as hidden `.config` files. It verifies the sealed bundle locally and checks uploaded file sizes and symlink targets before asking iOS to install.

Choose **Retry** after correcting a failed step. A failed signing attempt uses a fresh working directory on retry. An install retry in the same run uses the signed bundle and opens a fresh tunnel. Once the app is installed, open it from the Home Screen. If iOS asks you to trust the developer, follow the prompt under **Settings → General → VPN & Device Management** for the account used to sign.

## Apple sign-in

### iCloud Advanced Data Protection

**Try v0.22.4 or later with Advanced Data Protection still on.** Version 0.22.4 adopts FrizzleM's updated Mac client information and matches the anisette identity to that client on a full sign-in. A usable saved developer session can still be tried first. When a full login needs a new identity, Apple may ask for a code again. Keep your vault, pairing file, and cached IPA when upgrading. An iLoader user reported successful sign-in on an Advanced Data Protection account with the same type of client update. That report is from iLoader, and real-account verification on Minis is still pending. [FrizzleM's change](https://github.com/FrizzleM/SideInstaller/commit/9e8d7c167ebfd6347086aa661d659c01371468b5) · [iLoader report](https://github.com/nab138/iloader/issues/1203#issuecomment-7080551523)

If verification still fails on the affected account and you choose to test with standard data protection, **temporarily turn off iCloud Advanced Data Protection**. On a trusted iPhone or iPad signed in to that Apple Account, open **Settings → [your name] → iCloud → Advanced Data Protection → Turn Off Advanced Data Protection**. Return to Minis and start a fresh sign-in. Keep Apple two-factor authentication enabled. This setting applies to the account and returns categories such as iCloud Backup, Photos, and Notes to standard data protection. You can turn Advanced Data Protection back on from the same screen after testing. Future sign-ins may need verification again. [Apple's settings instructions](https://support.apple.com/en-us/108756) · [iCloud data security overview](https://support.apple.com/en-us/102651)

### Verification codes

1. Check Apple devices already signed in to the **same Apple Account**. Approve the notification you initiated and enter its six-digit code in siboot.
2. If no notification arrives, open **Other ways** and select text or call for a listed trusted number. The choices depend on what Apple supplies for this session. Opening the menu does not request another code.
3. If Apple shows a rate limit or a refusal, keep the error code for troubleshooting. Wait when Apple requests a cooldown. When the issue persists, check the affected stage before changing any account settings.

The prompt reports request outcomes. It can show Apple error codes from plist or JSON responses even when the HTTP request returned 200. Apple controls delivery. [Apple's verification guidance](https://support.apple.com/en-us/102606)

### Save or update your login

The page's **show password** button or **Ctrl-V** in the live terminal toggles visibility. Password entry starts hidden. Plain terminal mode uses hidden input.

After signing in, you can save the account with a **separate local passphrase** of at least 12 characters, different from the Apple password. siboot derives an encryption key from the passphrase and protects the saved login and related session and signing data. You enter the passphrase to unlock that storage each time. If the Apple password changed, choose **Unlock and update saved password**. **Use another Apple Account once** uses temporary data for that run. **Forget saved login** removes the local vault and signing cache after confirmation. Certificate management is a separate action.

## Renew an app

Run the following command before a Personal Team provisioning profile expires, typically after seven days.

```sh
"$HOME/.siboot/bin/siboot" renew
```

The page offers **Quick renew** when it can read the original IPA from the last successful installation. This uses the same developer team and bundle identity, signs the whole app again, and verifies the upload. It reuses the cached original and the pairing record. Keep Wi-Fi and LocalDevVPN active and complete any Apple verification prompted. To switch apps or teams, install the original IPA again with a normal installation. If the cache could not be saved or was moved, import the original again. [Apple Developer account guidance](https://developer.apple.com/help/account/basics/about-your-developer-account)

## Command reference

Here `siboot` means `"$HOME/.siboot/bin/siboot"` or the path to your verified copy.

| Command or option | Behavior |
| --- | --- |
| `siboot` / `siboot --ipa PATH` | Open the app picker or install a local Linux path. |
| `siboot renew` | Sign and reinstall the cached original. Use the same developer team. |
| `siboot check` / `siboot connect` | Check Minis, Apple reachability, and the device, or pair and verify the tunnel. |
| `siboot developer-mode` | Pair, connect, and ask iOS to make its Developer Mode switch available if needed. Use Settings to enable it. |
| `siboot developer-mode --turn-on` | Request activation and a restart after confirmation. With a device passcode, iOS shows the switch for manual activation. |
| `siboot connect --pairing-file PATH` | Verify and import an RPPairing record. The path is inside Minis. |
| `--web` / `--terminal` / `--terminal --plain` | Choose the page, live terminal, or plain terminal. |
| `--lang en` / `--lang zh-CN` | Choose the terminal language. Chinese is the default. Use the language switch on the page for its own display. |
| `--anisette URL` / `--check-anisette` | Pin one server, or check only public server metadata. See [network diagnostics](#network-diagnostics). |
| `--home DIR` | Use a separate state directory. Pairing and performance logs still go beside the executable. |
| `--device-ip IP` / `--port PORT` | Override the LocalDevVPN endpoint. The defaults are `10.7.0.1` and `49152`. |
| `--apple-id EMAIL` | Preselect an Apple Account. Enter the password in the prompt. |
| `--jobs N` / `--perf-log` | Set 1–4 signing workers or record detailed timings. Default worker count is 1. |
| `--verbose` | Show underlying diagnostics in the terminal. Review the output before sharing. |
| `--help` / `--version` | Show all flags or the exact running version and credits. |

## Troubleshooting

| Symptom or log | What to check |
| --- | --- |
| `segmentation fault` at startup, even for `--version` | Run `uname -m` and `sh install-minis.sh --verify` for the actual file. Earlier startup failures came from incomplete downloads. Resume with the current installer, or use `--restart` if the partial file is corrupt. |
| Download lock remains after closing Minis | Confirm every installer has stopped, then remove the empty lock directory with `rmdir "$HOME/.siboot/bin/.siboot-download.lock"`. Rerun the installer. |
| Device handshake times out or Settings shows no pairing entry | Reconnect LocalDevVPN, enable Wi-Fi and grant Minis local network access. Run `check`. Open Minis 1.16+ can create a record, or use `connect --pairing-file PATH` with a valid one. |
| Ani DNS, TLS, timeout, or HTTP error | Run the metadata check below. Look at the failing host. A custom proxy's private CA must be trusted inside Minis. |
| `gsa.apple.com` returns HTTP 503 | Check the installed version and the URL in the error. v0.22.4 uses the `Mac OS X` spelling in the updated Mac client information. Apple can also return a temporary service error. |
| Trusted numbers or code request returns 403, or Apple reports `-22410` or `-23782` | Upgrade and start a new sign-in. The current build matches the provisioning identity to the Mac client information. If no code arrives, follow [the verification steps](#verification-codes) and [the Advanced Data Protection instructions](#icloud-advanced-data-protection). |
| Apple HTTP 429 | Pause sign-in attempts until the limit clears. An HTTP 429 from an anisette endpoint is a separate server limit. |
| Developer error `1100` after sign-in | The developer session expired. Restart the current version and sign in again. A stale session is normally replaced when Apple rejects it during the saved-session check. |
| `Directory not empty` on a signing retry | Check for a separately downloaded older executable. Minis versions from 0.22.0 use private signing directories for each attempt. |
| Installed app fails to open | Verify Developer Mode, developer trust, and the provisioning profile's validity. Renew with the same signing team if the profile expired. |

### Network diagnostics

```sh
"$HOME/.siboot/bin/siboot" --check-anisette --terminal
"$HOME/.siboot/bin/siboot" --check-anisette --anisette https://new.ani.sidestore.io --terminal
```

This probes public `/v3/client_info` metadata, with up to three requests in flight and a 12-second limit per endpoint. It runs before opening login storage or connecting to the device. A successful check establishes metadata reachability. A full sign-in still needs anisette provisioning, Apple authentication, and developer access. TLS uses bundled public roots alongside Minis's system trust.

Auto selection tries the last successful endpoint first when available. Use `--anisette https://new.ani.sidestore.io` on a normal run to try that endpoint alone. Switching endpoints can require provisioning and a fresh Apple verification.

### Slow signing or installation

```sh
"$HOME/.siboot/bin/siboot" renew --perf-log
```

The opt-in `siboot-perf-*.jsonl` file goes beside the executable. Compare `ipa.extract` and `sign.bundle` for archive and CPU work, `install.upload` and `install.verify_upload` for device transfer, and `install.ios` for the system installation phase. The signing worker count affects CPU work. File count, tunnel latency, and iOS validation affect installation separately. Start with the default `--jobs 1` under emulation, and compare the same IPA with `--jobs 2` if signing takes most of the time.

The log contains timing and count fields and caps detailed records at 16 MiB. A normal exit flushes the final summary. For an unresolved failure, share the version, Minis and iOS versions, failing step, relevant HTTP or Apple error, and a reviewed excerpt of `$HOME/.siboot/siboot.log` when available. Keep passwords, codes, pairing files, and vault contents private. [Report an issue](https://github.com/Jay-Void-BA/SideInstaller/issues)

## Data and migration

| Path | Contents |
| --- | --- |
| `$HOME/.siboot/bin/siboot` | Default installed program. |
| Beside the actual executable | `rp_pairing_file.plist` and optional `siboot-perf-*.jsonl`. Pairing records contain device trust keys. |
| `$HOME/.siboot/apple-login.vault` and `isideload/` | Optional encrypted login and signing state. `--home` changes the base path. |
| `$HOME/.siboot/renewals/` | Original IPA and metadata for the last cached successful install. |
| `$HOME/.siboot/siboot.log` | Error report for a failed run when available. |
| Private temporary directory | Extracted signing workspace retained through retries and cleaned up when its owner exits normally. |

**Upgrade Minis** by running the new release's installer in the same destination. If you move the executable, keep the pairing alongside it or import it at the new path. Keep the state directory to reuse the saved login and renewal cache.

**Migrate from rootshell** by exporting your original IPA and compatible RPPairing record through Files, opening them in Minis, and signing in again. Each app has its own sandbox. See the [rootshell archive](patches/rootshell/README.md).

## Development

Update the English and Simplified Chinese READMEs in the same commit as each change. Update any affected component README and keep the guides in the prepared source in sync. For internal changes, record the relevant development or verification details. [Repository instructions](AGENTS.md)

From the SideInstaller checkout, prepare the pinned siboot source in a new directory and link this checkout's Rust core.

```sh
source_dir="$(mktemp -d)/siboot-source"
bash scripts/prepare-minis.sh "$source_dir"
cargo test --locked --manifest-path "$source_dir/minis/Cargo.toml" --all-targets
cargo clippy --locked --manifest-path "$source_dir/minis/Cargo.toml" --all-targets -- -D warnings
LOCKED=1 bash scripts/test-rust.sh
python3 -m unittest discover -s scripts/tests -v
```

CI builds a static `aarch64-unknown-linux-musl` executable on ARM64 with Rust 1.100, runs startup and network checks, and verifies complete offline signing with a fixture. The release job verifies the source revision, target, and digests, then publishes that tested executable with its installer and `SHA256SUMS`. Device and Apple Account behavior require a real-device check. Build prerequisites, patch maintenance, and test scope are in the [developer guide](patches/siboot/README.md).

## Credits and license

Maintained by **Jay-Void-BA**. FrizzleM provided the original idea and implementation for siboot and the SideInstaller code used by its Rust core. [siboot by FrizzleM](https://github.com/FrizzleM/siboot) · [SideInstaller by FrizzleM](https://github.com/FrizzleM/SideInstaller)

The SideInstaller License covers inherited SideInstaller code. The vendored libraries retain their respective licenses and attribution. [License terms](LICENSE.md)
