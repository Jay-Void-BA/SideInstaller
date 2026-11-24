# rootshell archive

rootshell runtime support and WASM releases ended with **siboot 0.21.4**. For current installation, use Open Minis and the static Linux ARM64 release. [Current setup](../../README.md) · [简体中文](../../README.zh-CN.md)

## Move an existing installation to Minis

1. Export the original IPA from rootshell through the iOS Files app. Keep a copy you can select again for future installations.
2. If the device has a working RPPairing record, export it privately. Pairing records contain device trust keys. Use a new pairing in Minis when the old record is unavailable or the device rejects it.
3. Install and verify the ARM64 release in Open Minis. Open its page, choose the local IPA from Files, then import the pairing file during **Pairing**. A verified record is saved beside the new executable.
4. Sign in to Apple again and complete verification. The apps have separate sandboxes. The Minis vault and quick-renew cache are created by the current installer.

A rootshell `siboot.wasm` file is a WASI module. The current `siboot-aarch64` is an ARM64 Linux ELF for the Open Minis terminal. Use the release files and verification script for the runtime you are using. [Data paths and migration](../../README.md#data-and-migration)

## Historical material

- [Last rootshell release](https://github.com/Jay-Void-BA/SideInstaller/releases/tag/siboot-v0.21.4) records the 1.0.16 build 178 host and its release checks.
- [The pinned siboot patch](../siboot/retry-local-ipa.patch) preserves the older `rootshell/` entry points.
- [WASI host checks](../../scripts/check-rootshell-runtime.sh), [WASM packaging](../../scripts/package_rootshell.py), and the `rootshell-http` and `rootshell-runtime` crates are references for historical builds.
- The current Minis release pipeline builds a native binary with Linux file and socket APIs. Performance metrics use the shared [siboot-perf crate](../../rust-core/vendor/siboot-perf/Cargo.toml). [Minis build and release guide](../siboot/README.md)

Archive maintained by **Jay-Void-BA**. Original concept and implementation by **FrizzleM**. [SideInstaller by FrizzleM](https://github.com/FrizzleM/SideInstaller) · [License](../../LICENSE.md)
