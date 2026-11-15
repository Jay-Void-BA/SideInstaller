<div align="center">

# siboot for Minis

**在 iPhone 的 Minis 中，使用自己的 Apple 账号签名并安装本地 IPA。**

维护与定制：**[Jay-Void-BA](https://github.com/Jay-Void-BA)**

基于 [FrizzleM/siboot](https://github.com/FrizzleM/siboot) 与 [FrizzleM/SideInstaller](https://github.com/FrizzleM/SideInstaller)。

[下载 siboot-aarch64](https://github.com/Jay-Void-BA/SideInstaller/releases/download/siboot-v0.20.0/siboot-aarch64) · [发布与 SHA-256](https://github.com/Jay-Void-BA/SideInstaller/releases/tag/siboot-v0.20.0) · [CI](https://github.com/Jay-Void-BA/SideInstaller/actions)

</div>

## 这是什么

这是面向 Minis 的 siboot 定制版。直接在手机上选择本地 IPA，完成配对、Apple 账号登录、签名和安装；**不需要先安装 SideInstaller，也不需要电脑或 rootshell**。

当前设备连接通过 **LocalDevVPN**，因此除了 Minis 和 siboot，仍需安装并连接 LocalDevVPN。Minis 1.16+ 可以直接创建配对；也可以选择 Minis／iSH 内已有的 RPPairing 文件。

此版本只接受**本地 IPA**，不再自动下载 SideInstaller IPA，也不提供 IPA URL 安装入口。请自行准备有权安装的 IPA。

## 使用条件

- iPhone／iPad，当前安装流程面向 **iOS 27+**。
- [Open Minis](https://apps.apple.com/app/id6759188481)，建议 1.16 或更新版本。
- [LocalDevVPN](https://apps.apple.com/app/id6755608044) 已连接，Wi-Fi 开启。
- 已开启“设置 › 隐私与安全性 › 开发者模式”。
- Apple 账号、可用网络，以及本地 IPA 文件。账号验证、anisette 服务和签名配置需要网络。

## 下载和启动

发布附件只有 **`siboot-aarch64`**。在 Minis 终端选择一个固定目录下载和运行，配对文件会保存在该程序的同目录。

```sh
curl -fL -C - --retry 5 --retry-all-errors \
  -o siboot-aarch64 \
  https://github.com/Jay-Void-BA/SideInstaller/releases/download/siboot-v0.20.0/siboot-aarch64
sha256sum ./siboot-aarch64
```

将输出与[发布页面](https://github.com/Jay-Void-BA/SideInstaller/releases/tag/siboot-v0.20.0)的 SHA-256 核对，完全一致后再执行：

```sh
chmod +x ./siboot-aarch64
./siboot-aarch64 --version
./siboot-aarch64
```

下载时保持 Minis 在前台，等待命令结束。若出现 `connection closed prematurely` 或校验不一致，重新执行同一条 `curl` 命令续传，再核对哈希。下载不完整的 ELF 文件可能一启动就显示 `segmentation fault`。

## 安装本地 IPA

1. 保持 LocalDevVPN 连接，运行 siboot。
2. 在 Minis 打开的页面中，从“文件”选择本地 `.ipa`，点击“使用所选 IPA”。文件以小块导入并检查完整性。
3. 选择配对方式。Minis 1.16+ 可直接配对；已有文件也可从 Minis／iSH 的目录浏览器选择。
4. 按提示登录 Apple 账号，输入验证码。可选择加密保存登录。
5. 等待签名、上传检查与安装完成，再从主屏幕打开应用。首次安装需在“设置 › 通用 › VPN 与设备管理”中信任开发者。

IPA 已经在 Minis 文件系统中时，可以直接指定路径：

```sh
./siboot-aarch64 --ipa /root/App.ipa
```

终端模式支持路径包含空格：

```sh
./siboot-aarch64 --terminal --ipa '/root/My App.ipa'
```

免费 Apple 账号签名通常有效 7 天，到期前需重新签名；Apple 的应用数量、证书和 App ID 限额仍适用。

## 快速续签

成功安装后，siboot 会保留**上次安装应用的原始 IPA**。下次可在页面点击“快速续签 应用名”，或在终端运行：

```sh
./siboot-aarch64 renew
```

续签复用缓存 IPA 和仍然有效的配对，省去重新选择文件和配对方式。保存了加密登录时，仍需解锁；没有保存登录时仍要输入 Apple 账号。程序重新获取签名配置、完整签名并重新安装，不复用旧的过期签名。

快速续签要求使用上次安装的 Apple 开发者团队，避免无意生成另一个应用标识。页面导入会直接移动导入副本；从外部路径安装时会复制一份原始 IPA，并显示保存进度，需要相应的存储空间。缓存只保留最近一次成功安装的应用，位于 `~/.siboot/renewals`；原始本地文件不会被修改。更换要续签的应用时，正常安装新的本地 IPA 即可。

## 为什么 Install 可能较慢

安装分为三段：

1. **上传**：通过 Minis／iSH 中的加密隧道和 AFC 发送文件；每个文件的打开、写入、关闭都有开销，小文件多时尤其明显。
2. **核对**：逐个检查上传文件的大小及符号链接目标，防止安装缺少资源的应用。
3. **iOS 安装**：由系统安装服务验证签名并处理应用，进度由 iPhone 返回。

本版本按 **1 MiB** 分块读取和发送大文件，避免一次将整个大文件读入内存，并持续显示上传进度、文件核对进度及三个阶段的耗时。快速续签减少前置操作，仍需要重新签名、传输和系统安装；它不绕过完整签名或上传核对。

## 中文、英文和密码查看

- 网页界面默认中文，顶部可以切换 **中文／English**，选择保存在当前浏览器中。
- 终端与 `--help` 默认中文；使用 `SIBOOT_LANG=en ./siboot-aarch64` 切换英文。
- 密码和解锁口令默认隐藏，网页输入框旁可点击“显示密码／隐藏密码”。切换语言不改变已输入的内容。
- 实时终端输入密码时，**Ctrl+V** 切换查看。简洁输出模式保持密码隐藏。
- 日志和页面事件不记录或回放密码。原始协议错误保留原文，便于排查。
- 运行界面及版本信息显示 **Jay-Void-BA**，上游项目署名保留。

## 保存 Apple／iCloud 登录

首次成功登录后，可选择加密保存。请设置与 Apple 密码不同、至少 12 个字符的本地解锁口令。

- 使用 **scrypt + XChaCha20-Poly1305** 保护账号、密码、开发者会话、签名私钥与 anisette 缓存。
- 解锁口令不保存到磁盘；下次使用已保存的账号时需要输入。
- 登录界面支持更新密码、临时使用其他账号，以及删除已保存的登录和本地签名数据。
- 删除本地数据不会撤销 Apple 证书；错误解锁口令不会向 Apple 发起登录。
- 保存目录默认 `~/.siboot`，可通过 `--home` 更改。存储文件权限 `0600`，目录权限 `0700`。

Minis／iSH 没有系统钥匙串，因此本版本使用独立口令加密保存。登录验证需要联系 Apple；anisette 服务与网络可用性仍会影响登录。

## 选择和保存配对文件

在配对页面浏览 Minis／iSH 内的目录，或输入完整路径。命令行也支持：

```sh
./siboot-aarch64 --pairing-file /root/Phone.plist --ipa /root/App.ipa
```

支持 XML／二进制 RPPairing plist 及对应 JSON 导出。公私钥必须匹配；传统 lockdown 配对文件不适用于此隧道。

**验证成功后**，程序以 `0600` 权限将副本原子保存到可执行文件同目录：

```text
/home/siboot-aarch64
/home/rp_pairing_file.plist
```

源文件保留，导入失败也会保留旧配对。通过符号链接启动时，保存位置以真实可执行文件所在目录为准。

## 完整签名与重试

- 递归识别并签名所有 Mach-O，包括深层 dylib、无扩展名代码和嵌套 app／appex／framework／xpc。
- `.config`、隐藏文件和其他普通资源写入签名清单，签名后再次核对最终内容。
- 上传保留相对符号链接，并检查文件大小和链接目标后再安装。
- 步骤失败时可重试当前步骤，保留前面已经完成的结果。安装重试复用签名结果并重建隧道。

## 其他命令

```sh
./siboot-aarch64 check
./siboot-aarch64 connect
./siboot-aarch64 developer-mode
./siboot-aarch64 --help
```

`developer-mode --turn-on` 会先询问再请求开启开发者模式，iPhone 可能重启；需要遵守系统的锁屏密码限制。

## 源码、构建与 CI

此仓库保留 SideInstaller 的共享 Rust 核心和历史 iOS 源码。Minis 版本的源码补丁位于 [`patches/siboot`](patches/siboot/README.md)，构建时应用到固定的 siboot 上游提交，并使用本仓库的核心。

| 路径 | 内容 |
| --- | --- |
| `patches/siboot/` | Minis 源码补丁及说明，含中英文词条与页面 |
| `rust-core/` | Apple 登录、配对、签名与设备通信核心 |
| `scripts/prepare-minis.sh` | 准备固定版本的 Minis 源码 |
| `scripts/check-minis-runtime.sh` | 在 Minis 所用的 iSH 引擎中验证 ARM64 成品启动 |
| `.github/siboot-release.json` | Minis 发布版本与说明 |
| `.github/workflows/release-siboot.yml` | CI 通过后构建并发布单个 `siboot-aarch64` |
| `ios-app/`、`ios-tests/` | 保留的 SideInstaller iOS 应用与测试，并非运行 siboot 的前置条件 |

准备并测试 Minis 源码：

```sh
bash scripts/prepare-minis.sh /tmp/siboot-source
cd /tmp/siboot-source/minis
cargo test --locked --all-targets
cargo clippy --locked --all-targets -- -D warnings
```

在 ARM64 Alpine／musl 环境构建：

```sh
RUSTFLAGS='-C target-feature=+crt-static -C relocation-model=static' \
  cargo build --release --locked --target aarch64-unknown-linux-musl
```

CI 检查 Minis、共享 Rust 核心、现有 iOS 测试与翻译。发布前检查 ELF 架构、静态链接和 iSH 启动，再上传二进制并记录 SHA-256。页面流程另用 Chromium 手机尺寸验证；CI 不能代替真实 Apple 账号和 iPhone 安装测试。

## 上游与许可

本项目由 **Jay-Void-BA** 维护定制，感谢 **FrizzleM** 的 siboot、SideInstaller，以及 idevice、isideload、apple-codesign-quick 和 OpenMinis 等项目。

本仓库是独立 fork。上游代码的版权、署名和许可继续保留，详见 [LICENSE.md](LICENSE.md) 及各依赖的许可文件。
