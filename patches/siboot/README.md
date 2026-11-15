# Minis／siboot 定制源码

维护：**Jay-Void-BA**。上游：[FrizzleM/siboot](https://github.com/FrizzleM/siboot)。

- 补丁基线：`82b808de3a407619be7fbe54b3f313932bb80ade`。
- 完整补丁：[retry-local-ipa.patch](retry-local-ipa.patch)，包含 Minis 源码、中文词条、页面和文档。
- 依赖本 fork 的共享 Rust 核心，请通过脚本准备完整源码。

## 当前行为

- 只签名、安装本地 IPA，移除 SideInstaller IPA 自动下载、下载按钮和默认地址。IPA URL 在登录前拒绝。
- 默认中文，页面可切换 English；`SIBOOT_LANG=en` 使用英文终端。原始协议详情保留原文。
- 运行时显示 `Jay-Void-BA`，保留上游署名。密码与解锁口令可显示／隐藏，默认隐藏；实时终端 Ctrl+V 切换查看，日志与页面事件不记录秘密输入。
- 页面与 `renew` 命令支持快速续签上次应用，缓存原始 IPA、复用有效配对，仍需重新签名和安装；使用相同开发者团队。
- 大文件按 1 MiB 读取与发送，显示上传、核对、iOS 安装耗时，保留完整上传检查。
- 失败后重试当前步骤，保留已完成结果。安装重试复用签名结果并重建隧道。
- 加密保存 Apple／iCloud 登录、会话、私钥和 anisette 缓存。独立口令解锁，可更新密码、临时切换账号或清除登录。
- 浏览 Minis／iSH 的配对文件，验证后以 `0600` 原子保存至程序同目录的 `rp_pairing_file.plist`，保留源文件及失败时的旧配对。
- 完整递归签名 Mach-O 和嵌套代码包，封存并校验 `.config` 等普通资源，上传保留符号链接。

当前网络连接方式仍需要 LocalDevVPN；Minis 1.16+ 可直接配对，无需先安装 SideInstaller 或 rootshell。

## 准备、测试、预览

在本仓库根目录运行：

```sh
bash scripts/prepare-minis.sh /tmp/siboot-source
cd /tmp/siboot-source/minis
cargo test --locked --all-targets
cargo clippy --locked --all-targets -- -D warnings
cargo run --locked --example screens -- local-ipa --web
cargo run --locked --example screens -- secure-setup --web
```

预览不联系 Apple 或真实设备，只应输入测试密码。`locales/zh-CN.json` 同时供终端和页面使用；语言切换仅修改展示文本，不修改输入内容。

## 发布

`.github/siboot-release.json` 声明版本，`release-siboot.yml` 在 CI 通过后构建 ARM64 musl 静态程序。发布前在 Minis 所用 iSH 引擎中执行版本与中文帮助检查，然后仅上传 `siboot-aarch64` 并记录 SHA-256。不会自动下载或打包新的 SideInstaller IPA。

上游 `minis/bin/` 和旧 `install.sh` 是基线内容，不能代替本 fork 的 Release 成品。

真实 Apple 账号登录及 iPhone 安装仍需要真机验证。
