# siboot

在 Open Minis 中为 iPhone 和 iPad 签名、安装和续签本地 IPA。

[English](README.md) · **简体中文**

[最新发布](https://github.com/Jay-Void-BA/SideInstaller/releases/latest) · [更新记录](CHANGELOG.md) · [持续集成](https://github.com/Jay-Void-BA/SideInstaller/actions/workflows/checks.yml)

siboot 从本地 IPA 开始，完成设备配对、Apple 账户验证、开发者签名和安装。可以在 Minis 页面从“文件”导入 IPA、选择已有配对记录，或快速续签上次安装的应用。终端也提供完整的签名安装流程。

> [!IMPORTANT]
> 请使用 Open Minis 的 ARM64 版。rootshell 支持止于 0.21.4。要迁移原始 IPA 或已有配对记录，请参阅[文件与迁移](#文件与迁移)。

[环境要求](#环境要求) · [安装 siboot](#安装-siboot) · [安装 IPA](#安装-ipa) · [Apple 账户登录](#apple-账户登录) · [续签应用](#续签应用) · [故障排查](#故障排查) · [开发](#开发)

## 环境要求

| 项目 | 要求 |
| --- | --- |
| 设备 | 运行 iOS 或 iPadOS 27 及以上的 iPhone 或 iPad。运行开发签名应用需要开启开发者模式。 |
| 运行环境 | 支持 ARM64 Linux 的 Open Minis。在终端运行 `uname -m`，确认输出为 `aarch64`。Open Minis 1.16 或以上支持本机配对，旧版需要导入有效配对记录。 |
| 连接 | 在目标设备上开启 Wi-Fi 并连接 LocalDevVPN。Apple 登录和获取签名资料需要互联网。 |
| 账户 | 可为开发者团队签名、可完成 Apple 验证的 Apple 账户。个人团队可以用于个人测试。 |
| IPA 和空间 | 准备可重新签名的完整本地 IPA。为 IPA、解压后的应用和续签缓存预留空间。iCloud 云盘中的文件要先下载到设备上。 |

通过 App Store 安装 [Open Minis](https://apps.apple.com/cn/app/open-minis/id6759188481) 和 [LocalDevVPN](https://apps.apple.com/app/id6755608044)。

1. 开启 Wi-Fi，连接 LocalDevVPN。如果 iOS 询问，允许 Open Minis 访问本地网络。配对、安装和续签时保持 LocalDevVPN 连接。
2. 前往 **设置 → 隐私与安全性 → 开发者模式**，开启开发者模式，重启并在解锁后确认。如果看不到开关，完成配对后运行 `developer-mode`，让 iOS 显示该选项。[Apple 开发者模式说明](https://developer.apple.com/documentation/xcode/enabling-developer-mode-on-a-device)
3. 下载、导入、签名和安装时保持 Minis 运行。配对环节会提示何时进入“设置”、何时返回。

如果你的 Apple 账户开启了 iCloud 高级数据保护，先阅读[对应的登录说明](#icloud-高级数据保护)，了解兼容性修复和何时可尝试切换回标准数据保护。

## 安装 siboot

在 **Open Minis 的 Linux 终端**检查架构。缺少下载工具时安装 curl 和 CA 证书。

```sh
uname -m
apk add --no-cache curl ca-certificates
```

确认输出为 `aarch64` 后使用 `siboot-aarch64`。普通 x86 iSH 和旧 rootshell 的 WASM 环境无法运行这个文件。

下载并运行最新公开版本的安装脚本。

```sh
curl -fL --retry 3 -o install-minis.sh.part \
  https://github.com/Jay-Void-BA/SideInstaller/releases/latest/download/install-minis.sh &&
  mv install-minis.sh.part install-minis.sh &&
  sh install-minis.sh
```

脚本将程序安装到 `$HOME/.siboot/bin/siboot`，核对精确字节数与 SHA-256 后才替换旧程序。下载中断时再次运行 `sh install-minis.sh` 续传。校验提示分片损坏时，运行 `sh install-minis.sh --restart` 重新下载。

校验并启动实际安装的程序。

```sh
sh install-minis.sh --verify &&
  "$HOME/.siboot/bin/siboot" --version &&
  "$HOME/.siboot/bin/siboot" --web
```

版本输出应与下载的发布版本一致，并显示 `Jay-Void-BA` 和 `FrizzleM`。安装脚本固定对应一个版本。升级时下载新版脚本，并使用上面的安装路径启动。自行下载的 `./siboot-aarch64` 是另一份程序。

<details>
<summary>手动下载和校验可执行文件</summary>

把同一版本的 `siboot-aarch64` 和 `install-minis.sh` 下载到 Minis 中的同一目录。先校验，再赋予执行权限。

```sh
sh install-minis.sh --verify ./siboot-aarch64 &&
  chmod 700 ./siboot-aarch64 &&
  ./siboot-aarch64 --version
```

若同时下载了 `SHA256SUMS`，可在该目录运行 `sha256sum -c SHA256SUMS`，一起核对脚本和二进制文件。使用手动下载的程序时，配对和性能日志以它所在的目录为准。[发布文件](https://github.com/Jay-Void-BA/SideInstaller/releases/latest)

</details>

## 安装 IPA

### 选择 IPA

运行 `"$HOME/.siboot/bin/siboot" --web`，打开 Minis 提供的页面，选择 **安装本地 IPA**，再从“文件”选取。页面会把 IPA 导入 Minis 的 Linux 文件系统。页面导入上限为 4 GiB，IPA 的 `Payload` 中应有一个主应用。导入中断时重新选择原文件。

IPA 已在 Minis 文件系统中时，传入实际路径。有空格的路径用引号包起来。

```sh
"$HOME/.siboot/bin/siboot" --terminal --ipa "/home/My App.ipa"
```

存放在其他 iOS 应用或 iCloud 云盘里的 IPA，请通过“文件”选取。已经复制到 Minis 的 Linux 文件系统时，再用 `--ipa` 指定路径。

### 配对设备

在**配对**步骤，选择复用已保存的记录、导入 RPPairing 文件，或创建新配对。

- **在 Minis 内配对。** 点**开始配对**，留在 Minis 等待提示可以切换到设置。如果 iOS 询问，允许 Minis 通知。保持静音播放，前往 **设置 → 隐私与安全性 → 开发者模式**，滚动到底部点 **Pair with siboot**。按提示输入设备锁屏密码。六位*配对码*会在 Minis 内显示，也可能通过通知送达。设置确认配对后返回 Minis。
- **导入已有配对。** 页面支持从“文件”选取，也可浏览 Minis 内的 Linux 文件。终端可使用下面的命令。请选择适用于这台设备的 RPPairing 记录。设备验证通过后，siboot 将副本保存为实际程序同目录的 `rp_pairing_file.plist`。导入失败会保留之前有效的记录。

```sh
"$HOME/.siboot/bin/siboot" connect --pairing-file "/home/phone.plist"
```

`connect` 在配对和隧道验证完成后停止。准备好 IPA 后，再运行 siboot 进行安装。配对码填入 iOS 设置，稍后的 Apple 账户验证码填入 siboot 的登录界面。

**已经显示六位输入框，却没收到横幅？** 保留输入框和当前 siboot 流程，切回 Minis，在页面或实时终端查看**配对码**。如果大音频面板挡住页面，收起大面板并让小播放器继续播放。再切回设置中原来的输入框填入配对码。0.22.5 起，首次通知授权会在 Minis 前台处理，通知失败时仍会显示当前配对码。下次配对前，也可以在 **设置 → 通知 → Open Minis** 打开横幅。

### 签名并打开应用

**连接**完成后登录 Apple 账户，完成 Apple 要求的验证。siboot 在需要时向开发者团队注册设备并获取签名资料。它签名扩展、框架、dylib 等嵌套可执行文件，将 `.config` 等隐藏资源包含在资源封印中。程序会在本地验证封印，在请求 iOS 安装前核对上传文件的大小与符号链接目标。

失败后可先处理提示的问题，再点**重试**。重试签名时使用新的工作目录。同一次运行中重试安装时会复用已签名的应用，并重新建立隧道。安装完成后从主屏幕打开应用。若 iOS 要求信任开发者，前往 **设置 → 通用 → VPN 与设备管理**，按提示信任签名所用的账户。

## Apple 账户登录

### iCloud 高级数据保护

**请先保持 iCloud 高级数据保护开启，并使用 v0.22.4 或更新版本登录。** 0.22.4 移植 FrizzleM 的较新 Mac 客户端标识，完整登录会选择与该标识匹配的 anisette 身份。已有会话仍可先尝试复用。需要建立新身份时，Apple 可能再次发送验证请求。升级时保留保险库、配对记录和 IPA 缓存。iLoader 有用户报告，在启用高级数据保护的账户上采用同类客户端修复后成功登录。该记录来自 iLoader，Minis 版仍需用真实账户验证。[FrizzleM 的修复](https://github.com/FrizzleM/SideInstaller/commit/9e8d7c167ebfd6347086aa661d659c01371468b5) · [iLoader 验证记录](https://github.com/nab138/iloader/issues/1203#issuecomment-7080551523)

如果该账户仍无法通过验证，并且你愿意用标准数据保护排查，可以**暂时关闭 iCloud 高级数据保护**。在已登录同一 Apple 账户的受信任 iPhone 或 iPad 上，进入 **设置 → [你的姓名] → iCloud → 高级数据保护 → 关闭高级数据保护**，再返回 Minis 发起一次新登录。请保留 Apple 双重认证。关闭高级数据保护会让整个账户恢复标准数据保护，iCloud 云备份、照片、备忘录等将失去这项额外的端对端加密保护。测试后可从同一页面重新打开。以后登录时仍可能需要验证。[Apple 设置说明](https://support.apple.com/zh-cn/108756) · [iCloud 数据安全概览](https://support.apple.com/zh-cn/102651)

### 收不到验证码

1. 查看已经登录**同一 Apple 账户**的受信任设备。确认通知是自己发起的登录，点允许，将显示的六位验证码填入 siboot。
2. 没有收到通知时，点**其他方式**，从 Apple 为本次登录提供的受信任号码中选择短信或电话。打开方式菜单不会重新发码。
3. Apple 提示限流或拒绝时，记下错误码。有冷却提示时等待后再试。持续失败时，先根据失败步骤和错误定位原因，再决定是否调整账户设置。

界面显示验证码请求的结果。即使 HTTP 状态为 200，Apple 在 plist 或 JSON 中返回的错误也会显示。验证码由 Apple 负责送达。[Apple 验证码说明](https://support.apple.com/zh-cn/102606)

### 保存或更新登录

密码默认隐藏。页面可以点击显示密码按钮，实时终端可用 **Ctrl-V** 切换可见性。简洁终端使用隐藏输入。

登录成功后，可以用至少 12 个字符的**独立本地保险库口令**保存登录。该口令应与 Apple 密码不同。程序用它派生加密密钥，保护已保存的登录和关联的会话及签名资料。以后每次复用时输入该口令解锁。Apple 密码修改后，选**解锁并更新保存的密码**。**仅本次使用其他 Apple 账户**采用本次运行的临时数据。**忘记保存的登录**在确认后移除本地保险库和签名缓存。证书管理是单独的操作。

## 续签应用

个人团队的描述文件通常在七天后过期，请提前续签。

```sh
"$HOME/.siboot/bin/siboot" renew
```

能读取上次成功安装的原始 IPA 时，页面会提供**快速续签**。续签使用原开发者团队和应用标识，复用原始 IPA 与配对记录，同时重新签名整个应用并验证上传。打开 Wi-Fi、保持 LocalDevVPN 连接，并完成任何 Apple 验证。更换应用或开发者团队请使用普通 IPA 安装。缓存丢失或此前未保存时，重新导入原始 IPA。[Apple 开发者账户说明](https://developer.apple.com/help/account/basics/about-your-developer-account)

## 命令参考

表中的 `siboot` 代表 `"$HOME/.siboot/bin/siboot"`，也可替换成已校验程序的实际路径。

| 命令或选项 | 用途 |
| --- | --- |
| `siboot` / `siboot --ipa PATH` | 从页面选择 IPA，或安装 Minis 内的本地 IPA。 |
| `siboot renew` | 用同一开发者团队为缓存的原始 IPA 续签并安装。 |
| `siboot check` / `siboot connect` | 检查 Minis、Apple 服务器和设备连通性，或完成配对并验证隧道。 |
| `siboot developer-mode` | 配对并连接，在需要时让 iOS 显示开发者模式开关，再到设置中开启。 |
| `siboot developer-mode --turn-on` | 确认后请求开启并重启。设备有锁屏密码时，iOS 会显示开关供你手动开启。 |
| `siboot connect --pairing-file PATH` | 校验并导入 Minis 内的 RPPairing 文件。 |
| `--web` / `--terminal` / `--terminal --plain` | 选择网页、实时终端或简洁终端。 |
| `--lang en` / `--lang zh-CN` | 切换终端语言，默认简体中文。网页语言通过页面里的选项切换。 |
| `--anisette URL` / `--check-anisette` | 固定一个节点，或仅探测公开客户端信息。见[网络诊断](#网络诊断)。 |
| `--home DIR` | 指定状态目录。配对和性能日志仍保存到实际程序同目录。 |
| `--device-ip IP` / `--port PORT` | 覆盖 LocalDevVPN 设备端点，默认分别为 `10.7.0.1` 和 `49152`。 |
| `--apple-id EMAIL` | 预选 Apple 账户，在提示界面输入密码。 |
| `--jobs N` / `--perf-log` | 设置 1–4 个签名线程或记录性能日志。默认 1 个线程。 |
| `--verbose` | 显示底层诊断。分享前请检查输出。 |
| `--help` / `--version` | 查看完整选项，或查看正在运行的版本和署名。 |

## 故障排查

| 现象或日志 | 排查方法 |
| --- | --- |
| 启动立即报 `segmentation fault`，连 `--version` 也失败 | 确认 `uname -m` 输出为 `aarch64`，用 `sh install-minis.sh --verify` 校验实际运行的文件。以往的此类报告涉及下载中断。用新版脚本续传，分片损坏时用 `--restart`。 |
| 关闭 Minis 后下载锁残留 | 确认所有安装脚本都已停止，用 `rmdir "$HOME/.siboot/bin/.siboot-download.lock"` 移除空的锁目录，再运行脚本。 |
| 设备握手超时，或设置里没有配对入口 | 重连 LocalDevVPN，打开 Wi-Fi，允许 Minis 本地网络访问。运行 `check`。在 Open Minis 1.16+ 创建配对，或用 `connect --pairing-file PATH` 导入有效记录。 |
| 六位配对码输入框已出现，但没有通知 | 保留当前输入框，切回 Minis 查看配对码，保持小播放器运行后再切回设置。见[配对步骤](#配对设备)。 |
| ani 报 DNS、TLS、超时或 HTTP 错误 | 运行下方的客户端信息检测，看报错来自哪个主机。自定义代理的私有 CA 也需要在 Minis 中受信任。 |
| `gsa.apple.com` 返回 HTTP 503 | 核对实际版本和错误里的 URL。v0.22.4 的较新 Mac 客户端仍使用兼容的 `Mac OS X` 字样。Apple 服务本身也可能临时出错。 |
| 受信任号码或发码请求报 403，或 Apple 报 `-22410`、`-23782` | 升级后重新登录。新版会匹配客户端与配置身份。仍收不到验证码时，依次查看[验证步骤](#收不到验证码)和[高级数据保护说明](#icloud-高级数据保护)。 |
| Apple 返回 HTTP 429 | 暂停登录尝试，等待 Apple 限流解除。ani 节点的 429 属于独立的服务器限流。 |
| 登录完成后报告开发者错误 `1100` | 开发者会话过期。重新启动当前版本并登录。复用会话时，程序通常会在 Apple 拒绝已保存的会话后重新登录。 |
| 重试签名提示 `Directory not empty` | 检查是否仍运行单独下载的旧程序。Minis 从 0.22.0 起为每次签名尝试创建独立工作目录。 |
| 安装成功后应用无法打开 | 检查开发者模式、开发者信任和描述文件有效期。描述文件过期后，用同一团队续签。 |

### 网络诊断

```sh
"$HOME/.siboot/bin/siboot" --check-anisette --terminal
"$HOME/.siboot/bin/siboot" --check-anisette --anisette https://new.ani.sidestore.io --terminal
```

检测只访问公开的 `/v3/client_info`，最多同时探测三个节点，每个节点限时 12 秒。该检测在打开登录存储或连接设备前执行。检测通过说明客户端信息接口可达。完整登录还需要配置 anisette 身份、Apple 验证和开发者访问权限。程序内置公共 CA，同时支持 Minis 的系统信任库。

自动选择会优先尝试上次成功的节点。在普通安装命令中加 `--anisette https://new.ani.sidestore.io`，可仅使用该节点。更换节点可能需要配置身份并重新完成 Apple 验证。

### 签名或安装较慢

```sh
"$HOME/.siboot/bin/siboot" renew --perf-log
```

程序同目录会生成 `siboot-perf-*.jsonl`。对比 `ipa.extract` 和 `sign.bundle` 了解解压及 CPU 耗时，通过 `install.upload` 和 `install.verify_upload` 区分传输与核对，再看 `install.ios` 了解系统安装耗时。签名线程数影响 CPU 工作，文件数量、隧道延迟和 iOS 校验单独影响安装。模拟环境先用默认 `--jobs 1`。若主要耗时在签名，可用同一 IPA 比较 `--jobs 2`。

性能日志记录计时与计数，详细记录上限为 16 MiB，正常结束时会写入汇总。反馈持续性问题时，请提供版本、Minis 和 iOS 版本、失败步骤、相关 HTTP 或 Apple 错误，以及检查过的 `$HOME/.siboot/siboot.log` 摘录（如有）。请保管好密码、验证码、配对文件和保险库内容。[提交问题](https://github.com/Jay-Void-BA/SideInstaller/issues)

## 文件与迁移

| 路径 | 内容 |
| --- | --- |
| `$HOME/.siboot/bin/siboot` | 默认安装的可执行文件。 |
| 实际程序同目录 | `rp_pairing_file.plist` 和可选的 `siboot-perf-*.jsonl`。配对记录包含设备信任密钥。 |
| `$HOME/.siboot/apple-login.vault` 与 `isideload/` | 可选的加密登录与签名状态。`--home` 会改变其基础目录。 |
| `$HOME/.siboot/renewals/` | 上次缓存成功的原始 IPA 和元数据。 |
| `$HOME/.siboot/siboot.log` | 失败时生成的普通错误报告（如有）。 |
| 独立私有临时目录 | 解压和签名工作区，在重试期间保留，持有者正常退出时清理。 |

**升级 Minis 版**时，将新版本安装到原目录。如果移动程序，可一起移动原配对记录，或在新位置重新导入。保留状态目录即可复用保存的登录和续签缓存。

**从 rootshell 迁移**时，先通过“文件”导出原始 IPA 和兼容的 RPPairing 记录，再在 Minis 中选取并重新登录。两个应用的沙盒彼此独立。[rootshell 历史资料](patches/rootshell/README.md)

## 开发

每次改动都在同一次提交中更新英文和简体中文 README。涉及有独立 README 的组件时，也更新对应说明，并同步补丁里的使用指南。内部调整应记录相关开发或验证细节。[仓库维护规则](AGENTS.md)

在 SideInstaller 仓库中运行以下命令。准备脚本将固定版本的 siboot 源码放入新目录，并接入当前仓库的 Rust 核心。

```sh
source_dir="$(mktemp -d)/siboot-source"
bash scripts/prepare-minis.sh "$source_dir"
cargo test --locked --manifest-path "$source_dir/minis/Cargo.toml" --all-targets
cargo clippy --locked --manifest-path "$source_dir/minis/Cargo.toml" --all-targets -- -D warnings
LOCKED=1 bash scripts/test-rust.sh
python3 -m unittest discover -s scripts/tests -v
```

CI 在 ARM64 上使用 Rust 1.100 构建静态 `aarch64-unknown-linux-musl` 程序，执行启动、网络和完整离线签名检查。发布任务核对源码提交、目标和摘要，再发布通过测试的同一份程序、安装脚本与 `SHA256SUMS`。Apple 账户和设备行为需在真机验证。构建依赖、补丁维护和测试范围见[开发说明](patches/siboot/README.md)。

## 致谢与许可证

由 **Jay-Void-BA** 维护。FrizzleM 提供 siboot 的原始思路与实现，以及 Rust 核心使用的 SideInstaller 代码。[siboot by FrizzleM](https://github.com/FrizzleM/siboot) · [SideInstaller by FrizzleM](https://github.com/FrizzleM/SideInstaller)

继承的 SideInstaller 代码适用 SideInstaller 许可证，随仓库提供的依赖保留各自的许可证和署名。[许可证条款](LICENSE.md)
