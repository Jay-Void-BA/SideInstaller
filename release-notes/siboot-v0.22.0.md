# siboot 0.22.0 · Minis ARM64

The supported runtime returns to Open Minis 1.16+ with a static Linux ARM64 executable. rootshell is deprecated; no WASM or pairing relay is required. LocalDevVPN and iOS / iPadOS 27+ remain required for device connectivity.

- Restore the complete Minis workflow: local IPA import, native pairing or verified RPPairing import, encrypted Apple login, complete recursive signing, checked upload and quick renewal.
- Isolate extraction in private, owned workspaces. Stream bounded ZIP extraction, validate symlink graphs before creation, and preserve hidden resources, frameworks, dylibs and extensions.
- Buffer extraction writes and cache directory creation. Keep CPU-heavy signing off async device workers, default to one signing worker under emulation, and add `--jobs 1..4` for measured comparisons.
- Reuse the original renewal IPA without another cache copy. Keep a consistent renewal snapshot and lock shared account/cache state through the workflow.
- Add native `--perf-log` beside the executable, with numeric measurements and bounded private logs. Keep full signing, resource sealing and per-file AFC checks enabled.
- Ship a release-pinned installer that resumes downloads, checks exact size and SHA-256, rejects bad paths, and replaces the existing program only after verification.
- Run the actual ARM64 release in CI, including offline CMS signing, resource tampering, archive safety, encrypted storage, pairing import and renewal checks. Publish the exact tested artifact after all checks pass.
- Rewrite the default English README and Chinese guide for Minis. Retain Jay-Void-BA and FrizzleM credits and inherited license terms.

## Install / 安装

See [English setup](https://github.com/Jay-Void-BA/SideInstaller#readme) or [中文配置](https://github.com/Jay-Void-BA/SideInstaller/blob/main/README.zh-CN.md).

下载 `install-minis.sh` 后在 Minis 中运行 `sh install-minis.sh`。默认程序位于 `$HOME/.siboot/bin/siboot`；配对文件和可选性能日志保存在程序同目录。`siboot renew --perf-log` 可分析续签各阶段耗时。普通 x86 iSH 无法运行此 ARM64 程序。

Offline and CI checks do not replace Apple-service or physical-device testing. Actual Minis/iOS installation speed has not been benchmarked here. The repository includes inherited implementation, not a clean-room rewrite; the existing license remains.
