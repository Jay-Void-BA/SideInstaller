# rootshell integration — deprecated

The supported runtime is [Minis ARM64](../../README.md). rootshell runtime support and WASM releases ended at 0.21.4. These pinned host references and compatibility scripts are historical; they are not active build or release gates.

The old WASI runtime and transport remain in the shared source tree for reference. Active performance diagnostics live in `rust-core/vendor/siboot-perf` and have no rootshell dependency. Minis uses native Linux files, sockets and signing; no rootshell relay or ABI shim is needed.

Migration: export the original IPA and RPPairing file through Files, import them into Minis, and sign in again. See [中文说明](../../README.zh-CN.md#文件与迁移). Credits: Jay-Void-BA and original author FrizzleM; inherited license terms remain.
