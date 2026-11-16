# siboot for rootshell · 0.21.1

在 iPhone／iPad 的 **rootshell** 中，使用自己的 Apple 账户签名、安装本地 IPA，并快速续签上次安装的应用。

维护与移植：**[Jay-Void-BA](https://github.com/Jay-Void-BA)**。原作者：**[FrizzleM](https://github.com/FrizzleM)**，基于 [siboot](https://github.com/FrizzleM/siboot) 与 [SideInstaller](https://github.com/FrizzleM/SideInstaller)。两位作者的署名均保留在运行界面中。

[下载 siboot.wasm](https://github.com/Jay-Void-BA/SideInstaller/releases/download/siboot-v0.21.1/siboot.wasm) · [Release 与 SHA-256](https://github.com/Jay-Void-BA/SideInstaller/releases/tag/siboot-v0.21.1) · [CI](https://github.com/Jay-Void-BA/SideInstaller/actions)

> **iSH／Minis 旧版已弃用（deprecated）**，不再提供该平台的功能更新或二进制发布。请使用 rootshell 版；已有 IPA 和配对文件可按下文迁移。历史源码和 Release 仅留存供迁移与参考。

## 运行条件

- 当前流程面向 **iOS／iPadOS 27+**，已开启“设置 › 隐私与安全性 › 开发者模式”。
- 最新版 [rootshell](https://github.com/kitknox/rootshell)，支持 WASI Preview 1、socket/TLS 和终端 raw 模式接口。
- [LocalDevVPN](https://apps.apple.com/app/id6755608044) 已连接，Wi-Fi 开启。
- Apple 账户、可用网络和自行准备的本地 IPA。Apple 登录、anisette 和签名配置仍需要联网。

**安装流程全部在 rootshell 和 `siboot.wasm` 内执行，无需 Minis，也无需先安装 SideInstaller。设备连接仍需 LocalDevVPN。** rootshell 是应用名称，不代表获得 iOS root 权限。

本程序只接受本地 IPA。没有 SideInstaller IPA 下载入口，也不在运行时检测软件更新。

## rootshell 版配置

### 1. 安装应用并打开本地终端

安装最新版 rootshell（[App Store](https://apps.apple.com/app/rootshell-local-terminal-ssh/id6755794662)／[中国区 App Store](https://apps.apple.com/app/rootshell-%E6%9C%AC%E5%9C%B0%E7%BB%88%E7%AB%AF-ssh/id6763402687)）和 [LocalDevVPN](https://apps.apple.com/app/id6755608044)。在 rootshell 中新建 **Local Shell（本地终端）** 标签页，以下命令均在这台 iPhone／iPad 的本地终端执行。

### 2. 配置权限、网络与后台保活

1. 在系统“设置 › 隐私与安全性 › 开发者模式”中开启开发者模式，按系统提示重启并确认。
2. 打开 Wi-Fi。在 LocalDevVPN 中添加系统 VPN 配置并连接，确认状态为 **Connected**。安装和续签期间保持 LocalDevVPN 为当前连接的 VPN。
3. 允许 rootshell 访问**本地网络**。若曾拒绝，在“设置 › 隐私与安全性 › 本地网络 › rootshell”中开启；首次权限弹窗出现时选择允许。
4. 在 rootshell 的 **Settings › Location Diary Mode** 中开启定位日记模式，选择 **Session Only**，按提示授予定位权限，使切换到系统设置进行配对时程序能继续运行。配置完成后可关闭该模式；以后重新配对时再开启。
5. 允许 rootshell 发送通知，以便在系统设置中查看六位配对码。配对码也会显示在终端中。

### 3. 准备固定目录与本地 IPA

在本地终端的默认起始目录逐条执行：

```sh
mkdir -p siboot
cd siboot
pwd
```

在“文件 › 浏览 › 我的 iPhone／iPad › rootshell › siboot”中放入自己的 `App.ipa`。以后均进入这个目录运行程序；新建标签页后先 `cd siboot`。文件名含空格时使用引号，例如 `--ipa "My App.ipa"`。

程序、IPA 和配对文件需要位于 rootshell 可访问的应用目录。存放在 iCloud Drive、下载目录或其他应用中的文件，先用“文件”的拷贝功能放入上述目录。旧版用户的迁移方法见“使用已有配对文件”。

### 4. 下载、校验并启动 siboot

保持终端位于刚才的 `siboot` 目录，逐条执行：

```sh
curl -fL -C - -o siboot-v0.21.1.wasm.part https://github.com/Jay-Void-BA/SideInstaller/releases/download/siboot-v0.21.1/siboot.wasm
sha256sum siboot-v0.21.1.wasm.part
```

将哈希与 [Release](https://github.com/Jay-Void-BA/SideInstaller/releases/tag/siboot-v0.21.1) 或附件 [SHA256SUMS](https://github.com/Jay-Void-BA/SideInstaller/releases/download/siboot-v0.21.1/SHA256SUMS) 核对。一致后再执行：

```sh
mv siboot-v0.21.1.wasm.part siboot.wasm
./siboot.wasm --version
./siboot.wasm --help
```

版本输出应包含 `siboot 0.21.1 (rootshell)`。随后按下节完成配对和安装；也可运行 `./siboot.wasm` 使用交互菜单。

下载时保持 rootshell 在前台。中断或哈希不一致时，重新执行同一条 `curl` 命令续传，再核对哈希。使用带版本号的 `.part` 文件，避免把旧版本内容续传进新版本。

也可以通过 Safari 下载 `siboot.wasm`，在“文件”中复制到 `rootshell/siboot` 文件夹，核对哈希后从本地终端运行。

配置后的目录大致如下；配对、登录和成功安装后会自动生成相应文件：

```text
siboot/
  siboot.wasm                程序
  App.ipa                    自行准备的本地 IPA
  ifconfig.txt               新配对前生成的 Wi-Fi 信息
  rp_pairing_file.plist       已保存的配对文件
  .siboot/                   加密登录与快速续签缓存
  siboot-perf-*.jsonl         启用 --perf-log 后的性能日志
```

保留 `rp_pairing_file.plist` 和 `.siboot/`，下次即可复用配对、解锁登录并快速续签。程序升级时将新版 `siboot.wasm` 放回同一目录。

## 首次配对与安装

1. 确认已完成上述配置，终端位于 `siboot` 目录，LocalDevVPN 已连接，Location Diary Mode 已设为 Session Only。
2. 在 rootshell 逐条运行，先记录本机 Wi-Fi 地址，再开始配对：

   ```sh
   ifconfig en0 > ifconfig.txt
   ./siboot.wasm pair
   ```

3. 按提示打开“设置 › 隐私与安全性 › 开发者模式”，点击 **Pair with siboot**，输入设备密码及程序通知中的六位配对码。
4. 返回 rootshell，确认配对成功，保持 LocalDevVPN 连接，然后运行：

   ```sh
   ./siboot.wasm --ipa App.ipa
   ```

5. 按提示登录 Apple 账户、输入验证码，等待签名、上传核对和系统安装完成。首次启动需在“设置 › 通用 › VPN 与设备管理”中信任开发者。

也可直接运行 `./siboot.wasm`，在菜单中选择 IPA 和配对方式。若 `ifconfig` 无法取得 Wi-Fi 地址，在“设置 › Wi-Fi › 当前网络右侧 ⓘ › IP 地址”查看本机 IPv4，再运行下面的命令，**将示例地址替换为本机地址**：

```sh
./siboot.wasm pair --wifi-ip 192.168.1.10
```

这个地址用于首次配对；日常通过 LocalDevVPN 连接设备的默认地址为 `10.7.0.1:49152`，通常无需修改 `--device-ip` 或 `--port`。`./siboot.wasm check` 可检查 VPN 设备连接，`./siboot.wasm connect` 可验证已保存的配对和完整隧道。

## 使用已有配对文件

**从已弃用的 iSH／Minis 版迁移：** 两者和 rootshell 的文件系统属于不同应用。先从旧应用通过“文件”的分享或拷贝功能，导出原始 IPA 和 RPPairing 文件到 `rootshell/siboot`，再从菜单选择，或者运行：

```sh
./siboot.wasm --pairing-file from-ish.plist --ipa App.ipa
```

支持 RPPairing 的 plist、JSON 和已知包装格式；传统电脑 lockdown 配对文件不能直接用于这个隧道。导入时先验证文件与本机的配对，再替换已保存的副本。

**`rp_pairing_file.plist` 保存在 `siboot.wasm` 同目录。** 导入失败时保留旧副本和源文件。配对文件含设备信任密钥，请妥善保管。

迁移后在 rootshell 中重新登录并保存本地登录存储。第一次成功安装后，会在 rootshell 内建立新的快速续签缓存；后续运行 `./siboot.wasm renew`。

## 加密保存登录与查看密码

登录时可以选择保存 Apple／iCloud 账户。程序使用 **scrypt + XChaCha20-Poly1305** 加密账户密码、会话、签名私钥及 anisette 缓存；每次使用独立的本地口令解锁。解锁口令至少 12 个字符，不能与 Apple 密码相同，也不会被保存。

密码默认隐藏。输入时用终端 **Ctrl-V** 切换显示／隐藏，提交时清除显示内容；账户密码和解锁口令保留原有空格及 Unicode 字符。登录菜单支持更新密码、临时使用其他账户和忘记登录。

默认数据目录为程序旁的 `.siboot/`。若上次被强制终止而提示数据目录被占用，先结束所有 siboot 标签页，再逐条执行：

```sh
rm .siboot/apple-login.active
./siboot.wasm
```

## 快速续签

成功安装后，程序保存上次应用的**原始 IPA**。下次直接运行：

```sh
./siboot.wasm renew
```

也可以在启动菜单选择“快速续签”。程序复用有效配对和登录会话，重新获取签名配置、完整签名并安装，使用与上次相同的开发者团队。原始 IPA 移走后仍可续签。

免费账户的应用通常有效 7 天，应在到期前续签。续签仍需联网、LocalDevVPN、解锁本地登录存储，以及 Apple 要求的验证。

## 完整签名、重试与耗时

- 递归签名主程序、App Extension、ExtensionKit、Framework、dylib 和其他 Mach-O。
- 封存并复核 `.config`、隐藏文件等资源。普通配置文件通过资源封印保护，并不是可独立代码签名的程序。
- 保留 IPA 内符号链接；rootshell 的 WASI 文件层中保留链接元数据，上传时通过 AFC 在设备上重建。
- 大文件以 1 MiB 缓冲区读取和发送，安装前核对所有文件大小和链接目标。
- 步骤失败后可以重试当前步骤。安装重试复用已完成的签名结果，并重新建立隧道。

程序分别显示上传、文件核对和 iOS 安装耗时。耗时来自解压、哈希／签名、Apple／anisette 请求、加密隧道、许多小文件的往返及系统 `installd`。rootshell 使用 WASM，不保证比 Minis 更快；iOS 的系统安装时间仍由系统决定。

## 性能诊断日志

复现安装或续签缓慢时，增加 `--perf-log`：

```sh
./siboot.wasm --perf-log --ipa App.ipa
./siboot.wasm renew --perf-log
```

每次运行会在 **`siboot.wasm` 同目录** 新建 `siboot-perf-时间戳-序号.jsonl`，启动与结束时显示具体位置。`--home` 不改变日志位置，旧日志保留；默认关闭。

日志为逐行 JSON，包含版本、时间戳、阶段与重试序号，以及：

- 用户输入等待；scrypt 解锁、RSA 密钥生成、CSR、解压、Mach-O 签名、资源哈希与封印复核耗时。
- HTTP 请求与 WebSocket 握手／接收耗时、状态码和字节数；宿主连接调用合计 DNS、TCP 与 TLS 时间。
- 网络收发次数、字节数、累计及最长耗时、采样延迟分桶估计的 P50／P95 上界；协作轮询的短暂超时与实际读写错误分别统计。
- 上传条目编号、大小、打开／分块写入／关闭耗时，上传核对及 iOS 系统安装进度，续签缓存耗时。
- WASM 线性内存容量；这不是系统进程的实际 RAM 占用。

最后一行 `summary` 汇总结果、各操作统计和 `log_write_us`（已统计的序列化／写入耗时）。文件按编号关联，不写入文件名、账户、密码、验证码、令牌、密钥、URL 或请求／响应正文。可以把日志文件用于后续性能分析。

关键阶段立即刷新，持续 I/O 约每秒刷新统计。详细事件达到 16 MiB 后停止新增，仍保留最终统计和 `dropped_events` 数量。正常退出会写入摘要；强制终止时，最后的缓冲数据及摘要可能缺失。

这些是**墙钟耗时**，嵌套／并发操作会重叠，不能简单相加；WASI 时钟通常约 1 ms 精度。日志从程序启动诊断后开始，不包括 rootshell 加载 WASM 的时间。开启日志会增加采集和写入开销，建议与关闭日志的运行结果对照。`--verbose` 仍用于在终端显示错误详情，可以与本参数一起使用。

## 常用命令

| 命令 | 功能 |
| --- | --- |
| `./siboot.wasm --ipa App.ipa` | 签名安装本地 IPA |
| `./siboot.wasm renew` | 续签上次安装的应用 |
| `./siboot.wasm --perf-log --ipa App.ipa` | 安装并在同目录输出详细性能日志 |
| `./siboot.wasm pair` | 在 rootshell 创建配对 |
| `./siboot.wasm check` | 检查设备连接 |
| `./siboot.wasm connect` | 验证配对及完整隧道 |
| `./siboot.wasm developer-mode` | 请求显示开发者模式 |
| `./siboot.wasm developer-mode --turn-on` | 确认后请求重启以启用开发者模式 |
| `./siboot.wasm --lang en` | 英文界面，默认中文 |
| `./siboot.wasm --help` | 查看所有选项 |

## 构建与验证

本仓库保留上游 iOS 应用和已弃用的 iSH／Minis 源码，共享同一套签名核心。**rootshell 是当前面向用户维护和发布的目标。** 旧源码及兼容性测试保留，不代表继续支持旧版安装方式。源码准备方式见 [补丁说明](patches/siboot/README.md)：

```sh
bash scripts/prepare-minis.sh /tmp/siboot-source
rustup target add wasm32-wasip1
cargo build --manifest-path /tmp/siboot-source/rootshell/Cargo.toml --release --locked --target wasm32-wasip1 --bins --examples
bash scripts/check-rootshell-runtime.sh /tmp/siboot-source/rootshell/target/wasm32-wasip1/release/siboot.wasm /tmp/siboot-source/rootshell/target/wasm32-wasip1/release/examples/runtime_check.wasm
```

CI 检查共享 Rust 测试、rootshell 测试、WASI 编译，以及固定版本 **rootshell 原版 Worker** 中的加密保存、CSR、完整 CMS 签名、符号链接、续签、密码切换、HTTP／WebSocket 和软件 TCP 传输。通过全部检查后发布 `siboot.wasm` 与 SHA-256，并将该版本标记为 Latest。**尚未完成真实 Apple 账号到 iPhone 安装的端到端验证。**

## iSH／Minis 版弃用与迁移

iSH／Minis 旧版已弃用，不再更新功能或发布新的 `siboot-aarch64`。历史 [0.20.0 Release](https://github.com/Jay-Void-BA/SideInstaller/releases/tag/siboot-v0.20.0) 仅作归档，不再作为安装指引。`siboot-aarch64` 是 Linux ARM64 程序，不能用于 rootshell。

请按本页“rootshell 版配置”安装 `siboot.wasm`，再按“使用已有配对文件”迁移 IPA 和配对文件。原作者与本 fork 维护者信息、历史源码及许可证继续保留。

## 致谢与许可证

保留 FrizzleM 的原作署名、版权与许可证，Jay-Void-BA 负责本 fork 的定制和 rootshell 移植。[rootshell](https://github.com/kitknox/rootshell) 由 kitknox 维护。共享核心使用 [idevice](https://github.com/jkcoxson/idevice)、[isideload](https://github.com/nab138/isideload) 和 [apple-codesign-quick](https://github.com/Dadoum/apple-crates)，各依赖保留其作者与许可证。

参见 [LICENSE.md](LICENSE.md) 及 vendored 依赖目录中的许可证。
