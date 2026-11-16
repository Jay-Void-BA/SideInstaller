<h1 align="center">siboot</h1>

<p align="center">
  <a href="README.md">English</a> · <strong>简体中文</strong>
</p>

<p align="center">
  在 iPhone／iPad 上签名、安装与续签本地 IPA。<br>
  为 rootshell 移植的完整 WASI 安装器。
</p>

<p align="center">
  <a href="https://github.com/Jay-Void-BA/SideInstaller/releases/tag/siboot-v0.21.4"><img src="https://img.shields.io/badge/release-0.21.4-2563eb?style=flat-square" alt="siboot 0.21.4"></a>
  <a href="https://github.com/Jay-Void-BA/SideInstaller/actions/workflows/checks.yml"><img src="https://github.com/Jay-Void-BA/SideInstaller/actions/workflows/checks.yml/badge.svg?branch=main" alt="CI 检查状态"></a>
  <a href="#requirements"><img src="https://img.shields.io/badge/platform-rootshell%20%7C%20iOS%2027%2B-64748b?style=flat-square" alt="rootshell · iOS／iPadOS 27+"></a>
  <a href="https://github.com/Jay-Void-BA/SideInstaller/blob/main/LICENSE.md"><img src="https://img.shields.io/badge/license-Custom-64748b?style=flat-square" alt="SideInstaller 自定义许可证"></a>
</p>

<p align="center">
  <a href="https://github.com/Jay-Void-BA/SideInstaller/releases/tag/siboot-v0.21.4">下载</a> ·
  <a href="#quick-start">快速开始</a> ·
  <a href="#usage">常用命令</a> ·
  <a href="#performance">性能诊断</a> ·
  <a href="#migration">旧版迁移</a> ·
  <a href="#development">开发</a>
</p>

<p align="center">
  维护：<a href="https://github.com/Jay-Void-BA">Jay-Void-BA</a> ·
  原始思路：<a href="https://github.com/FrizzleM">FrizzleM</a>
</p>

> [!IMPORTANT]
> **iSH／Minis 旧版已弃用（deprecated）**，不再提供该平台的功能更新或二进制发布。已有用户请按[迁移指南](#migration)转到 rootshell；历史源码和 Release 仅留存供迁移与参考。

## 功能

| 功能 | 说明 |
| --- | --- |
| 本地安装 | 在 rootshell 内完成设备配对、Apple 登录、IPA 签名、上传和安装 |
| 完整签名 | 递归处理主程序、扩展、Framework、dylib 与其他 Mach-O，封存并复核 `.config` 等资源 |
| 快速续签 | 缓存上次应用的原始 IPA，使用 `renew` 重新签名安装 |
| 加密登录 | 使用独立本地口令保护账户、会话与签名私钥，支持输入时显示／隐藏密码 |
| 步骤重试 | 出错后重试当前步骤，安装重试复用已完成的签名结果 |
| 性能诊断 | 按需将阶段耗时、网络与文件传输统计写入程序同目录 |
| 中文／英文 | 默认中文，使用 `--lang en` 切换英文 |

安装流程全部在 rootshell 和 `siboot.wasm` 内执行，设备连接仍需 LocalDevVPN。本程序只接受本地 IPA，不提供 SideInstaller IPA 下载入口，也不在运行时检测软件更新。

<a id="requirements"></a>

## 运行条件

| 项目 | 要求 |
| --- | --- |
| 系统 | iOS／iPadOS **27+**，已开启开发者模式 |
| 终端 | [rootshell](https://github.com/kitknox/rootshell) **1.0.16（178）** |
| 设备连接 | [LocalDevVPN](https://apps.apple.com/app/id6755608044) 已连接，Wi-Fi 开启 |
| 签名材料 | 自己的 Apple 账户、可用网络和本地 IPA |

Apple 登录、anisette 和签名配置需要联网。rootshell 是应用名称，不代表获得 iOS root 权限。

<a id="compatibility"></a>

## rootshell 兼容性

**仅支持 rootshell 1.0.16（178）**，使用其原版 socket／TLS、接收超时、终端和文件系统接口，无需修改宿主。早期版本及 build 175 回移方案不再受支持，也不再随 Release 提供。

CI 固定使用未经修改的 [1.0.16-178 源码](https://github.com/kitknox/rootshell/tree/192eab95001aef3f2ea4b282c0d3f38a3539fb3f)，校验 Worker 与 broker 的哈希，并模拟其单个根目录预打开项和 64 KiB 回包缓冲区。请先在 rootshell 设置中确认版本。

<a id="quick-start"></a>

## rootshell 版配置

### 1. 安装应用与配置权限

安装 rootshell **1.0.16（178）**（[App Store](https://apps.apple.com/app/rootshell-local-terminal-ssh/id6755794662)／[中国区 App Store](https://apps.apple.com/app/rootshell-%E6%9C%AC%E5%9C%B0%E7%BB%88%E7%AB%AF-ssh/id6763402687)）和 [LocalDevVPN](https://apps.apple.com/app/id6755608044)。

1. 在系统“设置 › 隐私与安全性 › 开发者模式”中开启开发者模式，按提示重启并确认。
2. 打开 Wi-Fi，在 LocalDevVPN 中添加系统 VPN 配置并连接，确认状态为 **Connected**。安装和续签期间保持 LocalDevVPN 为当前连接的 VPN。
3. 允许 rootshell 访问**本地网络**。若曾拒绝，在“设置 › 隐私与安全性 › 本地网络 › rootshell”中开启；首次弹窗出现时选择允许。
4. 新配对前，在 rootshell 的 **Settings › Location Diary Mode** 中开启定位日记模式，选择 **Session Only**，按提示授予定位权限，使切换到系统设置时程序能继续运行。配对完成后可关闭；重新配对时再开启。
5. 允许 rootshell 发送通知，以便在系统设置中查看六位配对码。配对码也会显示在终端中。

### 2. 准备工作目录

在 rootshell 中新建 **Local Shell（本地终端）** 标签页。以下命令均在这台 iPhone／iPad 的本地终端执行。在默认起始目录逐条运行：

```sh
mkdir -p siboot
cd siboot
pwd
```

在“文件 › 浏览 › 我的 iPhone／iPad › rootshell › siboot”中放入自己的 `App.ipa`。iCloud Drive、下载目录或其他应用中的文件，需要先拷贝到 rootshell 可访问的应用目录。新建标签页后先进入 `siboot` 目录。178 传给 WASM 的 argv 只含文件名，因此应先进入程序所在目录再启动。

仅首次配置时创建 `siboot`。目录已存在时，直接 `cd siboot` 进入即可。“文件”中的 `rootshell` 文件夹由应用自动提供。

### 3. 下载并校验

在工作目录下载固定版本的安装脚本，下载成功后才执行：

```sh
curl -fL --retry 3 -o install-rootshell.sh.part https://github.com/Jay-Void-BA/SideInstaller/releases/download/siboot-v0.21.4/install-rootshell.sh &&
mv install-rootshell.sh.part install-rootshell.sh &&
sh install-rootshell.sh
```

脚本会续传当前版本的 `.wasm.part`，核对**精确字节数和 SHA-256**，通过后才替换 `siboot.wasm`。下载中断或校验失败时保留原程序，以及配对、登录与续签数据。重复执行时，已完整且通过校验的程序不会重复下载。

```sh
sh install-rootshell.sh --verify
./siboot.wasm --version
```

版本输出应包含 `siboot 0.21.4 (rootshell 1.0.16 / 178)`。部分文件损坏或服务器拒绝续传时，运行 `sh install-rootshell.sh --restart` 重新下载当前版本；验证完成前仅改写部分下载文件。强制结束可能留下 `.siboot-download.lock`，先结束所有下载脚本，再用 `rmdir .siboot-download.lock` 删除这个空目录。

也可在 Safari 下载 `siboot.wasm` 和 `install-rootshell.sh`，拷贝到 `rootshell/siboot` 后运行 `sh install-rootshell.sh --verify`。所有发布附件的哈希见 [SHA256SUMS](https://github.com/Jay-Void-BA/SideInstaller/releases/download/siboot-v0.21.4/SHA256SUMS)。

> [!TIP]
> 下载时保持 rootshell 在前台。安装脚本或 `--verify` 成功后才运行 WASM。“数据段超出模块大小”发生在 siboot 启动前，因此校验由模块外的下载脚本完成。

<a id="first-install"></a>
<a id="首次配对与安装"></a>

### 4. 配对并安装

确认 LocalDevVPN 已连接，Location Diary Mode 已设为 Session Only。在工作目录逐条运行：

```sh
ifconfig en0 > ifconfig.txt
./siboot.wasm pair
```

按提示打开“设置 › 隐私与安全性 › 开发者模式”，点击 **Pair with siboot**，输入设备密码及通知／终端中的六位配对码。返回 rootshell，确认配对成功后运行：

```sh
./siboot.wasm --ipa App.ipa
```

按提示登录 Apple 账户、输入验证码，等待签名、上传核对和系统安装完成。首次启动应用时，需在“设置 › 通用 › VPN 与设备管理”中信任开发者。

文件名含空格时使用引号，例如 `--ipa "My App.ipa"`。也可运行 `./siboot.wasm`。首次交互启动会说明如何通过“文件”把 IPA 复制进 rootshell 文件夹，复制后按回车确认；取消后下次仍会显示，显式传入 `--ipa` 则跳过教程。文件选择器支持编号及完整本地路径；已有配对文件的用户可直接[导入配对](#migration)。

<details>
<summary>无法取得 Wi-Fi 地址时</summary>

在“设置 › Wi-Fi › 当前网络右侧 ⓘ › IP 地址”查看本机 IPv4。将下列示例地址替换为本机地址，代替读取 `ifconfig.txt`：

```sh
./siboot.wasm pair --wifi-ip 192.168.1.10
```

这个地址用于新配对。日常通过 LocalDevVPN 连接设备的默认地址为 `10.7.0.1:49152`，通常无需修改 `--device-ip` 或 `--port`。

</details>

<details>
<summary>文件、配对与缓存的保存位置</summary>

配对、登录和成功安装后，会自动生成相应文件：

```text
siboot/
  siboot.wasm                程序
  App.ipa                    自行准备的本地 IPA
  ifconfig.txt               新配对前生成的 Wi-Fi 信息
  rp_pairing_file.plist       已保存的配对文件
  .siboot/                   加密登录与快速续签缓存
  siboot-perf-*.jsonl         启用 --perf-log 后的性能日志
```

默认数据目录为程序旁的 `.siboot/`。保留配对文件与数据目录，下次即可复用配对、解锁登录并快速续签。程序升级时，将新版 `siboot.wasm` 放回同一目录。旧版从子目录启动时，可能错误地把数据放在沙盒根目录；这些文件仍保留，可传入 `--home /.siboot` 继续使用旧存储，必要时用 `--pairing-file /rp_pairing_file.plist` 导入旧配对。迁移存储前，后续运行也应保留显式的 `--home`。

</details>

<a id="usage"></a>

## 常用命令

| 命令 | 用途 |
| --- | --- |
| `./siboot.wasm --ipa App.ipa` | 签名安装本地 IPA |
| `./siboot.wasm renew` | 续签上次安装的应用 |
| `./siboot.wasm --perf-log --ipa App.ipa` | 安装并保存性能日志 |
| `./siboot.wasm pair` | 创建配对，保存到程序同目录 |
| `./siboot.wasm check` | 检查 VPN 设备连接 |
| `./siboot.wasm connect` | 验证已保存的配对与完整隧道 |
| `./siboot.wasm developer-mode` | 请求显示开发者模式 |
| `./siboot.wasm developer-mode --turn-on` | 确认后请求重启以启用开发者模式 |
| `./siboot.wasm --lang en` | 使用英文界面 |
| `./siboot.wasm --help` | 查看全部选项 |

### 快速续签

成功安装后，程序保存上次应用的**原始 IPA**。下次运行 `./siboot.wasm renew`，或在启动菜单选择“快速续签”，即可使用上次的开发者团队重新获取签名配置、完整签名并安装。原始 IPA 移走后仍可续签。

免费账户的应用通常有效 7 天，应在到期前续签。续签仍需联网、LocalDevVPN、解锁本地登录存储，以及 Apple 要求的验证。

### 登录与密码

登录时可选择保存 Apple／iCloud 账户。账户密码、会话、签名私钥及 anisette 缓存使用 **scrypt + XChaCha20-Poly1305** 加密，每次使用独立的本地口令解锁。口令至少 12 个字符，不能与 Apple 密码相同，也不会被保存。

输入密码时，按终端 **Ctrl-V** 切换显示／隐藏，提交、取消或输入结束时均清除当前显示内容。文件名和远程消息中的终端控制字符会转义后再显示。账户密码和解锁口令保留原有空格及 Unicode 字符。登录菜单支持更新密码、临时使用其他账户和忘记登录。

<a id="performance"></a>

## 性能诊断日志

复现安装或续签缓慢时，增加 `--perf-log`：

```sh
./siboot.wasm --perf-log --ipa App.ipa
./siboot.wasm renew --perf-log
```

每次运行会在 **`siboot.wasm` 同目录** 新建独立的 `siboot-perf-时间戳-序号.jsonl`，启动与结束时显示具体位置。默认关闭，旧日志保留，`--home` 不改变日志位置。

日志包含签名、网络、逐文件上传和系统安装耗时，便于定位瓶颈。不记录文件名、账户、密码、验证码、令牌、密钥、URL 或请求／响应正文。`--verbose` 用于在终端显示错误详情，可以同时使用。

<details>
<summary>日志字段、统计与计时限制</summary>

日志为逐行 JSON，包含版本、时间戳、阶段与重试序号，以及：

| 范围 | 记录内容 |
| --- | --- |
| 签名与本地计算 | scrypt 解锁、RSA 密钥生成、CSR、解压、Mach-O 签名、资源哈希与封印复核耗时 |
| 网络 | HTTP 请求、WebSocket 握手／接收耗时、状态码和字节数；宿主连接调用合计 DNS、TCP 与 TLS 时间 |
| 收发统计 | 次数、字节数、累计及最长耗时、采样延迟分桶估计的 P50／P95 上界；协作轮询的短暂超时与实际读写错误分别统计 |
| 上传与安装 | 条目编号、大小、打开／分块写入／关闭耗时、上传核对和 iOS 安装进度 |
| 其他 | 用户输入等待、续签缓存耗时、WASM 线性内存容量 |

最后一行 `summary` 汇总结果、各操作统计和 `log_write_us`（已统计的序列化／写入耗时）。文件按编号关联，可以将日志用于后续性能分析。

关键阶段立即刷新，持续 I/O 约每秒刷新统计。详细事件达到 **16 MiB** 后停止新增，仍保留最终统计和 `dropped_events` 数量。正常退出会写入摘要；强制终止可能缺少最后的缓冲数据及摘要。

这些是**墙钟耗时**，嵌套／并发操作会重叠，不能简单相加；WASI 时钟通常约 1 ms 精度。计时不包括 rootshell 加载 WASM 的时间；内存字段是线性内存容量，不是系统进程的实际 RAM 占用。开启日志也会增加采集和写入开销，建议与关闭日志的运行结果对照。

</details>

<details>
<summary>完整签名、上传与步骤重试</summary>

- 递归签名主程序、App Extension、ExtensionKit、Framework、dylib 和其他 Mach-O。
- 封存并复核 `.config`、隐藏文件等资源。普通配置文件通过资源封印保护，不是可独立代码签名的程序。
- 保留 IPA 内符号链接；rootshell 的 WASI 文件层中保留链接元数据，上传时通过 AFC 在设备上重建。
- 单次文件读取最多 32 KiB，以适配 rootshell 的 64 KiB JSON／Base64 回包；合并成 1 MiB 的 AFC 上传数据包，安装前仍核对所有文件大小和链接目标。
- 步骤失败后可重试当前步骤。安装重试复用已完成的签名结果，并重新建立隧道。

</details>

<a id="migration"></a>

## iSH／Minis 版弃用与迁移

iSH／Minis 旧版已弃用，不再更新功能或发布新的 `siboot-aarch64`。历史 [0.20.0 Release](https://github.com/Jay-Void-BA/SideInstaller/releases/tag/siboot-v0.20.0) 仅作归档。该文件是 Linux ARM64 程序，不能在 rootshell 中执行。

1. 按[rootshell 版配置](#quick-start)安装应用、配置权限并下载 `siboot.wasm`。
2. 从旧应用通过“文件”的分享或拷贝功能，导出**原始 IPA** 和 RPPairing 文件到 `rootshell/siboot`。两者的文件系统属于不同应用，无法直接互相访问。
3. 在 rootshell 本地终端进入该目录，替换下列文件名后运行：

   ```sh
   ./siboot.wasm --pairing-file from-ish.plist --ipa App.ipa
   ```

4. 在 rootshell 重新登录并保存本地登录存储。第一次成功安装后会建立新的续签缓存，后续运行 `./siboot.wasm renew`。

支持 RPPairing 的 plist、JSON 和已知包装格式；传统电脑 lockdown 配对文件不能直接用于这个隧道。没有可用文件时，在 rootshell 中[重新配对](#首次配对与安装)。

导入时先验证文件与本机的配对，再将副本保存为 **程序同目录的 `rp_pairing_file.plist`**。导入失败时保留旧副本和源文件。配对文件含设备信任密钥，请妥善保管。

## 常见问题

### WASM 数据段超出模块大小

该错误发生在 rootshell 解析模块时，siboot 尚未启动：加载到的字节少于模块声明的长度。先运行 `sh install-rootshell.sh --verify`，失败则运行 `sh install-rootshell.sh --restart`。不要执行或改名未通过校验的部分下载文件。若校验成功，但在 1.0.16（178）仍出现同样错误，请提供文件大小、SHA-256 和完整报错，以便单独调查宿主加载路径。

### 安装耗时较长

程序分别显示上传、文件核对和 iOS 安装耗时。解压、哈希／签名、Apple／anisette 请求、加密隧道、许多小文件的往返及系统 `installd` 都可能占用时间。rootshell 使用 WASM，不保证比 Minis 更快；系统安装时间由 iOS 决定。使用[性能诊断日志](#performance)判断具体耗时阶段。

本地 I/O 限制单次读取大小，并缓冲 ZIP 解压写入、复用已验证的父目录；解压日志记录 `directory_checks_saved`（省下的目录检查次数）；文件选择缓存条目类型并合并终端输出，快速续签只验证一次缓存快照。这些改动减少宿主调用，保留完整签名与上传核对；真机速度变化仍需通过 `--perf-log` 测量。

### 配对入口没有出现

确认 Wi-Fi 已开启、rootshell 已获本地网络权限，Location Diary Mode 为 Session Only。切换到系统设置时保持程序运行，等待数秒；换过 Wi-Fi 后重新生成 `ifconfig.txt`，或显式传入本机 `--wifi-ip`。

### 数据目录提示被占用

若上次被强制终止，先结束所有 siboot 标签页，确认没有程序仍在使用该目录，再逐条执行：

```sh
rm .siboot/apple-login.active
./siboot.wasm
```

<a id="development"></a>

## 开发与验证

rootshell 是当前面向用户维护和发布的目标。仓库保留上游 iOS 应用、已弃用的 iSH／Minis 源码及兼容性测试，共享同一套签名核心。源码布局见[补丁说明](https://github.com/Jay-Void-BA/SideInstaller/blob/main/patches/siboot/README.md)。

<details>
<summary>从源码构建与运行检查</summary>

在 SideInstaller 仓库根目录执行：

```sh
bash scripts/prepare-minis.sh /tmp/siboot-source
# Worker 与 JavaScriptCore 检查需要 Node 24 和 Bun 1.3.2。
rustup target add wasm32-wasip1
cargo build --manifest-path /tmp/siboot-source/rootshell/Cargo.toml --release --locked --target wasm32-wasip1 --bins --examples
bash scripts/check-rootshell-runtime.sh /tmp/siboot-source/rootshell/target/wasm32-wasip1/release/siboot.wasm /tmp/siboot-source/rootshell/target/wasm32-wasip1/release/examples/runtime_check.wasm
```

脚本名称为兼容已有开发流程保留。旧源码和测试的留存，不代表继续支持旧版安装方式。

</details>

CI 检查共享 Rust、rootshell、WASI 编译及通过 Bun 运行的 JavaScriptCore 解析，以及未经修改的 **1.0.16（178）** Worker，覆盖单根目录预打开的子目录启动、真实回包上限、大目录完整签名、HTTP／WebSocket、协作 TCP、加密存储、续签及中断／损坏下载。全部检查通过后生成 `siboot.wasm`、`install-rootshell.sh` 和 `SHA256SUMS`，并标记 Latest。

**尚未完成真实 Apple 账号到 iPhone 安装的端到端验证。**

## 作者与致谢

| 作者／项目 | 贡献 |
| --- | --- |
| [Jay-Void-BA](https://github.com/Jay-Void-BA) | 开发、维护与 rootshell 适配 |
| [FrizzleM](https://github.com/FrizzleM) | 原始设计思路与项目 [siboot](https://github.com/FrizzleM/siboot)、[SideInstaller by FrizzleM](https://github.com/FrizzleM/SideInstaller)；本实现沿用部分代码 |
| [kitknox/rootshell](https://github.com/kitknox/rootshell) | 终端与 WASM 运行环境 |
| [jkcoxson/idevice](https://github.com/jkcoxson/idevice) | 设备通信 |
| [nab138/isideload](https://github.com/nab138/isideload) | 侧载与签名流程 |
| [apple-codesign-quick](https://github.com/Dadoum/apple-crates) | Apple 代码签名 |

Jay-Void-BA 与 FrizzleM 的署名均保留在运行界面中。原作者信息、历史源码、版权及许可证继续保留。

## 许可证

本仓库沿用 [SideInstaller 自定义许可证](https://github.com/Jay-Void-BA/SideInstaller/blob/main/LICENSE.md)，使用与分发请遵循其中条款。各 vendored 依赖保留其作者与各自许可证。
