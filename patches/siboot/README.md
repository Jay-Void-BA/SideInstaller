# Minis 安装器配套修改

SideInstaller 的 iOS 应用和 Minis 安装器属于两个仓库。本目录保留 Minis
修改的完整补丁，避免只提交 SideInstaller 时丢失那部分修改。

- 上游：<https://github.com/FrizzleM/siboot>
- 补丁基线：`82b808de3a407619be7fbe54b3f313932bb80ade`
- 补丁：[retry-local-ipa.patch](retry-local-ipa.patch)
- 当前工作区的已修改源码：`/workspace/siboot/minis`

## 行为

- 步骤失败后提供 **Retry step / Stop**，仅重试该步骤，保留已完成的结果。
- 安装重试复用已签名应用并重建隧道；交接失败后重试不会重新安装。
- 页面提供本地 IPA 文件选择、分块导入和文件检查；终端保留 `--ipa` 路径用法。
- 本地 IPA 不访问 GitHub 下载；普通应用跳过 SideInstaller 专用的配对文件交接。
- 用户取消登录或输入时直接结束；失败日志保留原有处理方式。
- Apple／iCloud 登录可选择加密保存，以独立本地口令解锁；同时保护会话、签名私钥和 anisette 缓存，支持更新密码、一次性账号及清除登录。
- 页面可浏览 Minis／iSH 的配对文件；设备验证成功后以 `0600` 原子保存到程序同目录 `rp_pairing_file.plist`，原文件保留。
- 递归签名所有 Mach-O 和嵌套代码包，封存 `.config` 等普通资源并核对最终签名清单；上传时保留符号链接。

## 应用补丁

补丁依赖本 fork 的 Rust 核心修改。推荐从本仓库准备干净的 Minis 源码：

```sh
bash scripts/prepare-minis.sh /tmp/sideinstaller-minis
cd /tmp/sideinstaller-minis/minis
cargo test --locked --all-targets
cargo clippy --locked --all-targets -- -D warnings
```

可用 `cargo run --locked --example screens -- local-ipa --web` 预览本地
文件选择和失败重试；`cargo run --locked --example screens -- secure-setup --web`
预览配对文件浏览和加密保存、解锁。这些预览不会连接 Apple 或真实设备，只应输入测试密码。

## 构建及验证范围

补丁只包含源码，未替换上游仓库已有的 `bin/siboot-aarch64`。
本 fork 的 CI 会用 `scripts/prepare-minis.sh` 应用补丁并执行测试；所有检查通过后，
`release-siboot.yml` 按 `.github/siboot-release.json` 构建 ARM64 musl 静态二进制，
仅发布 `siboot-aarch64`。0.19.0 默认 IPA 固定为本 fork 已发布的 v2.1.0；
可在页面选择其他本地 IPA。本次不生成新 IPA。

手动构建时可设置 `SIBOOT_DEFAULT_IPA_URL` 指定默认安装的 IPA 下载地址。
上游安装命令仍运行上游已发布的二进制，不会自动包含本补丁。

Rust 测试包含加密和篡改检测、配对导入不覆盖旧文件、深层 Mach-O 与隐藏资源签名、
嵌套扩展标识更新及符号链接上传。Chromium 手机尺寸页面验证覆盖目录浏览、遮蔽密码、
解锁重试和刷新恢复。真实 Apple 账号登录和 iPhone 签名安装仍需真机验证。
