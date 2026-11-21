# Patched siboot source · Minis ARM64

Maintained by **Jay-Void-BA**. Original concept and implementation: [FrizzleM/siboot](https://github.com/FrizzleM/siboot). The patch is based on `82b808de3a407619be7fbe54b3f313932bb80ade`; original credits and inherited licenses remain.

[retry-local-ipa.patch](retry-local-ipa.patch) contains the active Minis installer, shared translations, the offline native probe and source documentation. Historical rootshell source is retained for reference; rootshell is deprecated and is excluded from active runtime checks and releases.

```sh
bash scripts/prepare-minis.sh /tmp/siboot-source
cargo test --manifest-path /tmp/siboot-source/minis/Cargo.toml --locked --all-targets
cargo clippy --manifest-path /tmp/siboot-source/minis/Cargo.toml --locked --all-targets -- -D warnings
```

Run these commands from the parent SideInstaller repository. The preparation script links this checkout's `rust-core` so tests and releases use the same signing and device fixes. The release target is static `aarch64-unknown-linux-musl`; CI builds with Rust 1.100 on ARM64 and runs the actual release through `scripts/check-minis-runtime.sh`.

The native pipeline validates a selected local IPA before remote work, keeps one renewal snapshot and holds a shared vault/cache lock, then performs pairing, login, complete signing and checked AFC upload. Each signing attempt owns a unique private extraction directory through asynchronous work and install retries. Extraction streams bounded data into buffered files and validates the full symlink graph before making links. Quick renewal reuses the original IPA without copying it again.

The interface includes English/Chinese, password visibility in the page/live terminal, encrypted login, verified pairing import beside the executable, current-step retry, and optional numeric performance logs in that same directory. No SideInstaller IPA download or update detection is offered. Shared code still signs dylibs and other Mach-O files and seals hidden resources such as `.config`; complete resource and per-file upload checks are retained.

The offline probe signs with a local test-only key, verifies its CSR, confirms CMS signatures and recursive resource seals, detects tampering, rejects escaping/cyclic archives, and exercises encrypted storage, renewal, pairing import and workspace cleanup. It does not contact Apple or install on a real iPhone.

See the [English guide](../../README.md) or [中文配置与迁移](../../README.zh-CN.md) for end-user setup. Source export must include new files (`git add -N`) before regenerating the patch with `git diff --binary`.

## Anisette TLS and diagnostics (0.22.1)

The standalone Minis manifest opts into `isideload/bundled-roots`, merging Mozilla public roots with system/private CAs. This fixes HTTP client construction when the Linux filesystem has no CA bundle; native application builds retain platform trust by default. `--check-anisette` probes public client metadata without opening login storage or contacting Apple authentication. It bounds concurrent work and reports each server's full failure. Server HTTP 429 failures fall back; Apple account limits still stop sign-in.

The ARM64 release is checked with empty certificate paths and a local TLS fixture that rejects unknown issuers, accepts a configured private CA and reports HTTP failures. Complete signing tests continue under the empty-CA environment.
