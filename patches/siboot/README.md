# Maintaining siboot for Minis

This directory connects the siboot installer to this repository's Rust signing and device code. User setup and Apple Account troubleshooting are in the [English README](../../README.md) and [简体中文](../../README.zh-CN.md).

## Source layout

| Path | Role |
| --- | --- |
| `scripts/prepare-minis.sh` | Checks out FrizzleM/siboot at `82b808de3a407619be7fbe54b3f313932bb80ade`, applies the patch, and links the current SideInstaller checkout. |
| `patches/siboot/retry-local-ipa.patch` | Contains the Minis CLI, page, Chinese catalog, source READMEs, and offline fixture runner. It also holds the historical rootshell changes. |
| `<prepared>/minis/` | Independent Cargo workspace with a lockfile for the Linux build. See its README after preparing the source. |
| `rust-core/src/apple_session.rs` | Opens the Apple developer session and checks whether an existing saved session still works. |
| `rust-core/vendor/isideload/` | Apple authentication, developer resources, archive extraction, and signing orchestration. |
| `rust-core/vendor/idevice/` | RPPairing, tunnel transport, AFC upload, and installation proxy. |
| `rust-core/vendor/apple-codesign-quick/` | Mach-O signatures, nested bundles, and resource seals. |
| `rust-core/vendor/siboot-perf/` | Opt-in local metrics. Test fixtures and release checks use the same event format. |

The prepared source's `SideInstaller` entry points to the checkout being tested. Start from the SideInstaller root so the patch, shared code, and release metadata agree. [Pinned source](https://github.com/FrizzleM/siboot/tree/82b808de3a407619be7fbe54b3f313932bb80ade)

## Build and test locally

Use Git, Bash, a Rust toolchain with Clippy, and Python 3.11 or later. Native Rust dependencies also require a C toolchain, CMake, Perl, Clang, and system headers. Rust 1.100 is the reference release toolchain. Create a new destination for each clean preparation.

```sh
source_dir="$(mktemp -d)/siboot-source"
bash scripts/prepare-minis.sh "$source_dir"
cargo test --locked --manifest-path "$source_dir/minis/Cargo.toml" --all-targets
cargo clippy --locked --manifest-path "$source_dir/minis/Cargo.toml" --all-targets -- -D warnings
LOCKED=1 bash scripts/test-rust.sh
python3 -m unittest discover -s scripts/tests -v
```

You can set `SIBOOT_REPOSITORY` to a local Git clone containing the pinned commit. For network-code changes, build the host CLI and run the local TLS fixture against it. The fixture exercises the metadata probe with empty system CA paths and a private CA. It needs `openssl` on the machine running the test.

```sh
cargo build --locked --manifest-path "$source_dir/minis/Cargo.toml" --bin siboot
python3 scripts/check-minis-network.py "$source_dir/minis/target/debug/siboot"
```

Run `python3 scripts/check_translations.py --base HEAD` for changes to iOS strings. Minis strings live in `minis/locales/zh-CN.json`. Its tests check that translated strings retain their named placeholders. The page translates display text in `minis/src/i18n.js` and the terminal uses `minis/src/i18n.rs`.

## Update the pinned patch

1. Make the changes in a prepared source checkout. Keep the root user guides and the prepared source copies in sync. Links to the parent project point through the `SideInstaller` entry in the prepared copies.
2. Mark added files with `git add -N -- path/to/new/file`. Export the changes under `README.md`, `README.zh-CN.md`, `minis`, and `rootshell` from the pinned baseline using the command below.
3. Prepare a second directory and run the relevant tests against the applied patch. Test the produced tree so forgotten files and stale patch context show up before CI.
4. For a release, update `minis/Cargo.toml`, the siboot package version in `minis/Cargo.lock`, `.github/siboot-release.json`, the release notes, and any version-specific guidance together. The user guides link to the latest published installer. The generated script still pins an exact version and digest.

```sh
git -C "$source_dir" diff --binary 82b808de3a407619be7fbe54b3f313932bb80ade -- \
  README.md README.zh-CN.md minis rootshell > patches/siboot/retry-local-ipa.patch
```

## Behavior to preserve

- **Input and retries.** Validate the local IPA before device or Apple work. Hold the selected original for the workflow. Give each signing attempt its own private extraction directory and retain the signed bundle while retrying an installation.
- **Pairing.** Check imports against the target device before replacing `rp_pairing_file.plist`. Resolve that path from the executable. Reopen the tunnel for installation after the Apple login and signing steps.
- **Login.** Keep the optional vault and signing cache under the state directory lock. Saved sessions are attempted before full password sign-in. For a full Mac sign-in, select an identity provisioned with identical client information. Keep earlier identity files available for their saved sessions. v0.22.4 uses FrizzleM's newer `Mac OS X;26.6` client and matching user agent. [Authentication integration](../../rust-core/vendor/isideload/README.md#current-minis-integration)
- **Code requests.** Keep the distinction between a request, Apple's refusal, and actual delivery. Show relevant Apple errors without logging codes. For the iCloud Advanced Data Protection fallback, refer to the [user guide](../../README.md#icloud-advanced-data-protection).
- **Signing and upload.** Sign nested executables, seal resources and symlinks, then check staged files and links on the device before installation. Quick renew uses the cached original with the previous bundle identity.
- **Logs.** Keep coded operation names and numeric measurements in `siboot-perf-*.jsonl`. Normal failure reports can contain device and protocol context. Filter and review those separately.

## CI and publication

`checks.yml` runs shared Rust, Minis, translation, and iOS simulator jobs. The ARM64 job builds `siboot` and `offline_check` in the pinned Rust 1.100 Alpine image with static linking. It runs the actual ARM64 executable, checks CLI flags and early invalid-IPA errors, verifies TLS behavior, and tests full CMS and resource signing, links, vault behavior, renewal, and cleanup. The fixture uses a local test key and makes no real Apple Account login or iPhone installation.

The ARM64 job uploads `minis-release-candidate` with `siboot-aarch64`, `install-minis.sh`, `SHA256SUMS`, and `build-info.json`. [Release verification](../../scripts/minis_artifact.py) ties the artifact to the source commit, version, target, and digests. The release workflow publishes the three public assets from that successful main-branch artifact. A release version already published is left in place. [CI workflow](../../.github/workflows/checks.yml) · [Release workflow](../../.github/workflows/release-siboot.yml)

Maintained by **Jay-Void-BA**, with the original idea and source by **FrizzleM**. [SideInstaller by FrizzleM](https://github.com/FrizzleM/SideInstaller) · [License](../../LICENSE.md)
