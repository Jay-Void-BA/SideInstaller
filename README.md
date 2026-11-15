# siboot for rootshell · 0.21.0

在 iPhone／iPad 的 **rootshell** 中，使用自己的 Apple 账户签名、安装本地 IPA，并快速续签上次安装的应用。

维护与移植：**[Jay-Void-BA](https://github.com/Jay-Void-BA)**。原作者：**[FrizzleM](https://github.com/FrizzleM)**，基于 [siboot](https://github.com/FrizzleM/siboot) 与 [SideInstaller](https://github.com/FrizzleM/SideInstaller)。两位作者的署名均保留在运行界面中。

[下载 siboot.wasm](https://github.com/Jay-Void-BA/SideInstaller/releases/download/siboot-v0.21.0/siboot.wasm) · [Release 与 SHA-256](https://github.com/Jay-Void-BA/SideInstaller/releases/tag/siboot-v0.21.0) · [CI](https://github.com/Jay-Void-BA/SideInstaller/actions)

## 运行条件

- 当前流程面向 **iOS／iPadOS 27+**，已开启“设置 › 隐私与安全性 › 开发者模式”。
- 最新版 [rootshell](https://github.com/kitknox/rootshell)，支持 WASI Preview 1、socket/TLS 和终端 raw 模式接口。
- [LocalDevVPN](https://apps.apple.com/app/id6755608044) 已连接，Wi-Fi 开启。
- Apple 账户、可用网络和自行准备的本地 IPA。Apple 登录、anisette 和签名配置仍需要联网。

**安装流程全部在 rootshell 和 `siboot.wasm` 内执行，无需 Minis，也无需先安装 SideInstaller。设备连接仍需 LocalDevVPN。** rootshell 是应用名称，不代表获得 iOS root 权限。

本程序只接受本地 IPA。没有 SideInstaller IPA 下载入口，也不在运行时检测软件更新。

## 下载与启动

在 rootshell 本地终端选择固定目录。以下命令请逐条执行：

```sh
curl -fL -C - -o siboot-v0.21.0.wasm.part https://github.com/Jay-Void-BA/SideInstaller/releases/download/siboot-v0.21.0/siboot.wasm
sha256sum siboot-v0.21.0.wasm.part
```

将哈希与 [Release](https://github.com/Jay-Void-BA/SideInstaller/releases/tag/siboot-v0.21.0) 或附件 [SHA256SUMS](https://github.com/Jay-Void-BA/SideInstaller/releases/download/siboot-v0.21.0/SHA256SUMS) 核对。一致后再执行：

```sh
mv siboot-v0.21.0.wasm.part siboot.wasm
./siboot.wasm --version
./siboot.wasm
```

下载时保持 rootshell 在前台。中断或哈希不一致时，重新执行同一条 `curl` 命令续传，再核对哈希。使用带版本号的 `.part` 文件，避免把旧版本内容续传进新版本。

也可以通过 Safari 下载 `siboot.wasm`，在“文件”中保存到 rootshell 文件夹，再从终端运行。

## 首次配对与安装

1. 在“文件”中把 `.ipa` 复制到 rootshell 文件夹。
2. 创建新配对前，开启 rootshell › Settings › Location Diary Mode › Session Only，以便切换到系统设置时保活。
3. 在 rootshell 逐条运行：

   ```sh
   ifconfig en0 > ifconfig.txt
   ./siboot.wasm pair
   ```

4. 按提示打开“设置 › 隐私与安全性 › 开发者模式”，点击 **Pair with siboot**，输入设备密码及程序通知中的六位配对码。
5. 返回 rootshell，保持 LocalDevVPN 连接，然后运行：

   ```sh
   ./siboot.wasm --ipa App.ipa
   ```

6. 按提示登录 Apple 账户、输入验证码，等待签名、上传核对和系统安装完成。首次启动需在“设置 › 通用 › VPN 与设备管理”中信任开发者。

也可直接运行 `./siboot.wasm`，在菜单中选择 IPA 和配对方式。`--wifi-ip 192.168.1.10` 可以直接指定本机 Wi-Fi IPv4 地址，代替读取 `ifconfig.txt`。

## 使用已有配对文件

Minis／iSH 和 rootshell 的文件系统属于不同应用。先通过“文件”的分享或拷贝功能，将原来的 RPPairing 文件复制进 rootshell，再从菜单选择，或者运行：

```sh
./siboot.wasm --pairing-file from-ish.plist --ipa App.ipa
```

支持 RPPairing 的 plist、JSON 和已知包装格式；传统电脑 lockdown 配对文件不能直接用于这个隧道。导入时先验证文件与本机的配对，再替换已保存的副本。

**`rp_pairing_file.plist` 保存在 `siboot.wasm` 同目录。** 导入失败时保留旧副本和源文件。配对文件含设备信任密钥，请妥善保管。

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

## 常用命令

| 命令 | 功能 |
| --- | --- |
| `./siboot.wasm --ipa App.ipa` | 签名安装本地 IPA |
| `./siboot.wasm renew` | 续签上次安装的应用 |
| `./siboot.wasm pair` | 在 rootshell 创建配对 |
| `./siboot.wasm check` | 检查设备连接 |
| `./siboot.wasm connect` | 验证配对及完整隧道 |
| `./siboot.wasm developer-mode` | 请求显示开发者模式 |
| `./siboot.wasm developer-mode --turn-on` | 确认后请求重启以启用开发者模式 |
| `./siboot.wasm --lang en` | 英文界面，默认中文 |
| `./siboot.wasm --help` | 查看所有选项 |

## 构建与验证

本仓库保留上游 iOS 应用和 Minis 源码支持，共享同一套签名核心。rootshell 是当前默认发布目标。源码准备方式见 [补丁说明](patches/siboot/README.md)：

```sh
bash scripts/prepare-minis.sh /tmp/siboot-source
rustup target add wasm32-wasip1
cargo build --manifest-path /tmp/siboot-source/rootshell/Cargo.toml --release --locked --target wasm32-wasip1 --bins --examples
bash scripts/check-rootshell-runtime.sh /tmp/siboot-source/rootshell/target/wasm32-wasip1/release/siboot.wasm /tmp/siboot-source/rootshell/target/wasm32-wasip1/release/examples/runtime_check.wasm
```

CI 检查共享 Rust 测试、rootshell 测试、WASI 编译，以及固定版本 **rootshell 原版 Worker** 中的加密保存、CSR、完整 CMS 签名、符号链接、续签、密码切换、HTTP／WebSocket 和软件 TCP 传输。通过全部检查后发布 `siboot.wasm` 与 SHA-256，并将该版本标记为 Latest。**尚未完成真实 Apple 账号到 iPhone 安装的端到端验证。**

Minis 用户可继续使用 [0.20.0 的 siboot-aarch64](https://github.com/Jay-Void-BA/SideInstaller/releases/tag/siboot-v0.20.0)。它是 Linux ARM64 程序；rootshell 使用本页的 WASM 文件。

## 致谢与许可证

保留 FrizzleM 的原作署名、版权与许可证，Jay-Void-BA 负责本 fork 的定制和 rootshell 移植。[rootshell](https://github.com/kitknox/rootshell) 由 kitknox 维护。共享核心使用 [idevice](https://github.com/jkcoxson/idevice)、[isideload](https://github.com/nab138/isideload) 和 [apple-codesign-quick](https://github.com/Dadoum/apple-crates)，各依赖保留其作者与许可证。

参见 [LICENSE.md](LICENSE.md) 及 vendored 依赖目录中的许可证。
