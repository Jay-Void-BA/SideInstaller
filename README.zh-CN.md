<h1 align="center">siboot</h1>
<p align="center">在 Minis 中签名、安装和续签本地 IPA。</p>
<p align="center"><a href="README.md">English</a> · <strong>简体中文</strong></p>
<p align="center">
  <a href="https://github.com/Jay-Void-BA/SideInstaller/releases/tag/siboot-v0.22.1"><img src="https://img.shields.io/badge/release-0.22.1-blue" alt="版本"></a>
  <a href="https://github.com/Jay-Void-BA/SideInstaller/actions/workflows/checks.yml"><img src="https://github.com/Jay-Void-BA/SideInstaller/actions/workflows/checks.yml/badge.svg?branch=main" alt="检查"></a>
  <a href="LICENSE.md"><img src="https://img.shields.io/badge/license-Custom-lightgrey" alt="许可证"></a>
</p>
<p align="center">维护：<a href="https://github.com/Jay-Void-BA">Jay-Void-BA</a> · 原始思路与代码：<a href="https://github.com/FrizzleM">FrizzleM</a></p>

siboot 在 iPhone 或 iPad 上完成本地 IPA 导入、设备配对、Apple 登录、完整签名和安装。运行环境为 **Minis + `siboot-aarch64`**；连接设备还需要 LocalDevVPN。

> [!IMPORTANT]
> **当前支持 Minis ARM64，rootshell 已弃用。** 0.22.0 回归静态 Linux ARM64 程序，不需要 rootshell 应用、WASM 模块或跨应用配对中继。普通 x86 iSH 无法运行此程序。

## 功能

- **完整签名**：递归签名主程序、扩展、Framework、dylib 及其他 Mach-O，封存 `.config` 等隐藏资源，保留有效符号链接，并在上传前核对资源封印。
- **本地安装**：在 Minis 页面从“文件”选择 IPA，或在终端指定 Linux 路径。没有 SideInstaller IPA 下载入口或自动更新检测。
- **快速续签**：通过 `renew` 复用原始 IPA、已保存配对和加密账户缓存，重新完整签名并安装。
- **加密登录**：以独立本地口令保护 Apple 账户、会话和签名密钥；页面与实时终端支持查看、隐藏密码。
- **步骤重试**：当前步骤失败后可重试；安装重试复用已经完成签名的应用。
- **性能诊断**：可选详细日志保存在程序同目录；提供中文、英文界面。

## 环境要求

| 项目 | 要求 |
| --- | --- |
| 设备 | iOS / iPadOS 27+，已开启开发者模式 |
| 运行环境 | Open Minis 1.16+，ARM64 Linux 环境，`uname -m` 输出 `aarch64` |
| 设备连接 | [LocalDevVPN](https://apps.apple.com/app/id6755608044) 已连接，Wi-Fi 已开启 |
| 账户 | 可完成验证的 Apple 账户 |
| 安装包 | 本地 IPA，以及存放原始 IPA、解压应用和签名工作文件所需空间 |

## 快速开始

### 1. 配置 Minis 和设备

在设备上安装 Open Minis 与 LocalDevVPN。在 **设置 › 隐私与安全性 › 开发者模式** 中开启开发者模式，按提示重启并确认。开启 Wi-Fi，连接 LocalDevVPN，安装和续签期间保持其为当前 VPN。

允许 Minis 访问**本地网络**。下载、导入、签名和安装期间保持 Minis 在前台。新配对使用 Minis 的 Bonjour 桥接与音频保活，允许短暂切换到系统设置；请按 Minis 提示授予权限。设置中没有配对入口时先运行 `siboot check`。

### 2. 下载与验证

在 **Minis 的 Linux 终端**中运行：

```sh
uname -m
apk add --no-cache curl ca-certificates
curl -fL --retry 3 -o install-minis.sh.part https://github.com/Jay-Void-BA/SideInstaller/releases/download/siboot-v0.22.1/install-minis.sh &&
mv install-minis.sh.part install-minis.sh &&
sh install-minis.sh
```

脚本将程序安装为 **`$HOME/.siboot/bin/siboot`**，核对精确大小与 SHA-256 后才替换旧程序。下载中断时保留分片，下次运行继续下载；已安装程序与应用数据不会因此丢失。

```sh
sh install-minis.sh --verify
"$HOME/.siboot/bin/siboot" --version
```

输出应包含 `siboot 0.22.1`、`Jay-Void-BA` 和 `FrizzleM`。[SHA256SUMS](https://github.com/Jay-Void-BA/SideInstaller/releases/download/siboot-v0.22.1/SHA256SUMS) 提供全部发布文件的摘要。分片损坏或服务器拒绝续传时，使用 `sh install-minis.sh --restart`。脚本不会自动启动程序。

也可手动下载 `siboot-aarch64` 和 `install-minis.sh`，放入同一目录，验证后运行：

```sh
sh install-minis.sh --verify ./siboot-aarch64
chmod +x ./siboot-aarch64
./siboot-aarch64 --version
```

### 3. 导入、配对、安装

```sh
"$HOME/.siboot/bin/siboot" --web
```

打开 Minis 提供的本地页面。首次交互安装会显示 IPA 导入教程。从**“文件”**选择 IPA；iCloud 文件请先下载到设备。页面会将文件传入 Minis 的 Linux 文件系统，完成前保持 Minis 在前台。

选择已保存的配对、导入现有 **RPPairing** 文件，或自动配对。新配对时按提示进入 **设置 › 隐私与安全性 › 开发者模式 › 与 siboot 配对**，输入设备密码和配对码，然后返回 Minis。不需要另一个终端应用。

按提示登录 Apple。保存登录时请设置**独立的本地保险库口令**，以后用它解锁已保存登录；程序不会保存该口令。页面使用密码可见按钮，实时终端使用 **Ctrl-V** 查看或隐藏密码。`--plain` 使用隐藏密码输入。

使用终端安装时，先把 IPA 导入 Minis，再指定其 Linux 路径：

```sh
"$HOME/.siboot/bin/siboot" --terminal --ipa /home/App.ipa
```

示例路径需换成 Minis 内的实际路径。rootshell、其他 iOS 应用或 iCloud Drive 的路径不能直接作为 Minis 的 Linux 路径。

## 使用方法

下表中的 `siboot` 请使用 `"$HOME/.siboot/bin/siboot"` 或你实际保存的程序路径。

| 命令 | 功能 |
| --- | --- |
| `siboot` | 选择本地 IPA 并运行安装流程 |
| `siboot --ipa /home/App.ipa` | 安装已经位于 Minis 内的本地 IPA |
| `siboot renew` | 重新签名并安装上次成功缓存的应用 |
| `siboot check` | 检查连接及 Minis 配对支持 |
| `siboot connect` | 验证配对并建立设备隧道 |
| `siboot --pairing-file /home/phone.plist` | 验证并导入 RPPairing 文件 |
| `siboot --lang en` / `siboot --lang zh-CN` | 选择英文／简体中文 |
| `siboot --terminal` / `siboot --web` | 选择实时终端／Minis 页面 |
| `siboot --check-anisette` | 无需 IPA、Apple 登录或 VPN，检测默认 ani 服务 |
| `siboot --perf-log` | 输出详细性能测量 |
| `siboot --jobs 2` | 使用两个签名线程，可选范围为 1–4 |
| `siboot --help` | 查看全部参数 |

界面默认为中文，`--lang en` 切换英文。免费 Apple 账户签名通常需要在七天内续签。续签仍需联网、LocalDevVPN 和 Apple 要求的验证；复用原始 IPA，无需重新导入或拷贝缓存，但仍执行完整签名和逐文件上传核对。更换应用或开发者团队时请使用普通 IPA 安装。

## 性能与诊断

Minis 通过模拟运行 Linux 程序。完整签名需要散列可执行页与资源；AFC 上传会通过本机设备隧道执行大量文件操作；之后由 iOS 验证并安装应用。界面分别显示上传、逐文件核对和 iOS 安装耗时，最后一部分由设备控制。

```sh
"$HOME/.siboot/bin/siboot" renew --perf-log
```

每次运行在**实际可执行程序同目录**生成独立的 `siboot-perf-*.jsonl`，从其他目录启动也一样。默认安装时位于 `$HOME/.siboot/bin/`。日志包含阶段耗时、重试、密码输入等待、解压数量、散列与签名耗时、上传字节与文件操作、核对、iOS 安装进度及最终摘要。记录接口只接受代码内固定操作名和数值，不接收密码、账户名、路径、配对内容或协议正文。详细记录上限为 16 MiB，正常退出时刷新摘要。

默认使用**一个签名线程**，适合模拟环境。可用同一 IPA 对比 `--jobs 2`；更多线程可能更慢或增加内存。0.22.0 缓冲解压写入、缓存目录创建、隔离签名工作目录，将重型签名移出异步设备线程，并避免续签重复拷贝 IPA。完整资源封印与逐文件上传检查继续保留。这些改动经过离线验证，实际速度取决于 IPA、Minis 和 iOS；不宣称已经取得真机性能数据。

普通错误报告位于 `$HOME/.siboot/siboot.log`，分享前请检查内容。独立性能日志采用更严格的数据限制。

## ani 连接诊断

0.22.1 为独立程序内置 Mozilla 公共 CA 证书，并继续支持系统及私有 CA。此前，精简 Minis 文件系统没有 CA 证书时，Apple 检查可因内置 Apple 根证书而通过，但全部 ani HTTP 客户端都会创建失败。

```sh
"$HOME/.siboot/bin/siboot" --check-anisette --terminal
"$HOME/.siboot/bin/siboot" --check-anisette --anisette https://ani.iloader.app --terminal
```

检测最多并发三个请求，每台服务器限时 12 秒，只请求 `/v3/client_info`，逐项显示 DNS、TLS、超时或 HTTP 错误。不打开登录保险库，不配对、不配置身份、不登录 Apple。检测通过仅代表客户端信息接口可达，后续身份配置或 Apple 认证仍可能单独失败。可加 `--perf-log` 记录数值计时。

旧版本及 Minis 中其他网络工具可通过 `apk add --no-cache ca-certificates` 和 `update-ca-certificates` 修复系统证书。自定义代理使用私有 CA 时，还需将该 CA 安装到 Minis；iOS 的证书库与 Minis 独立。ani 服务自身的 HTTP 429 会转试其他节点；Apple 账户限流仍会停止登录尝试。

## 文件与迁移

| 位置 | 内容 |
| --- | --- |
| 实际程序同目录 | `rp_pairing_file.plist` 和可选性能日志 |
| `$HOME/.siboot/`，或 `--home PATH` | 加密登录与签名缓存、续签原始 IPA、错误报告 |
| 每次签名独立的私有临时目录 | 解压、签名后的应用；最后一个持有者结束时清理 |

导入配对先经过设备验证，再以私有副本替换保存的记录。失败不会覆盖原文件或旧配对。页面支持从“文件”上传，或浏览 Minis 内的 Linux 文件系统。配对记录含设备信任密钥，请妥善保管。

**从 rootshell 迁移**：通过“文件”导出原始 IPA 和 RPPairing 文件，在 Minis 中导入并重新登录。两者沙盒独立，不会自动迁移旧路径和登录凭据。**从旧 Minis 版升级**：默认数据目录仍为 `$HOME/.siboot`。将原配对保留在新程序同目录，或用 `--pairing-file` 导入；升级无需删除数据目录。

rootshell 源码和 0.21.4 等历史发布保留供参考，不再提供运行时支持或新 WASM 发布。

## 常见问题

- **启动出现 `segmentation fault`**：先核对文件精确大小与 SHA-256。此前报告涉及下载中断；使用生成的安装脚本续传，仍校验失败时用 `--restart`。确认 `uname -m` 为 `aarch64`。
- **设置中没有配对入口**：保持 LocalDevVPN 连接和 Wi-Fi 开启，允许 Minis 本地网络访问，使用 Open Minis 1.16+。运行 `siboot check`，或导入有效 RPPairing 文件。
- **强制关闭后残留下载锁**：先停止全部安装脚本，再用 `rmdir` 仅移除空的 `$HOME/.siboot/bin/.siboot-download.lock` 目录后重试。
- **安装很慢**：打开 `--perf-log`，分别比较上传、核对和 iOS 安装耗时。保持 Minis 在前台，先使用 `--jobs 1`。

## 开发

```sh
bash scripts/prepare-minis.sh /tmp/siboot-source
cargo test --manifest-path /tmp/siboot-source/minis/Cargo.toml --locked --all-targets
cargo clippy --manifest-path /tmp/siboot-source/minis/Cargo.toml --locked --all-targets -- -D warnings
LOCKED=1 bash scripts/test-rust.sh
python3 -m unittest discover -s scripts/tests -v
```

准备脚本应用固定基线补丁，并接入本仓库共享 Rust 核心。CI 在 ARM64 runner 中使用 Rust 1.100 构建静态 `aarch64-unknown-linux-musl` 程序，执行实际 CLI 和完整离线签名检查，验证加密存储、符号链接及资源篡改检测。打包阶段拒绝残缺或动态 ELF。发布消费该次成功 CI 的同一份产物，并核对源码提交与摘要。iOS 源码检查独立保留；离线检查不模拟真实 Apple 登录或真机安装。

## 致谢与许可证

维护：**Jay-Void-BA**。原始思路与实现：**[FrizzleM 的 siboot](https://github.com/FrizzleM/siboot)**。共享签名与设备实现源自 **[SideInstaller by FrizzleM](https://github.com/FrizzleM/SideInstaller)** 及其中注明的依赖。

继承的 SideInstaller 代码继续遵循 [LICENSE.md](LICENSE.md)，依赖另遵循各自许可。仓库包含继承实现，**并非净室重写**，因此不能整体改标为 MIT；具体权利以许可证为准。
