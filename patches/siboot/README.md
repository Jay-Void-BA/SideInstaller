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

## 应用补丁

在干净的 siboot 仓库中执行，`PATCH` 应为本目录补丁的绝对路径：

```sh
git checkout 82b808de3a407619be7fbe54b3f313932bb80ade
git submodule update --init --recursive
PATCH=/absolute/path/to/SideInstaller/patches/siboot/retry-local-ipa.patch
git apply --check "$PATCH"
git apply "$PATCH"
cd minis
cargo test --locked --all-targets
cargo clippy --locked --all-targets -- -D warnings
```

可用 `cargo run --locked --example screens -- local-ipa --web` 预览本地
文件选择和失败重试；这个预览不会连接 Apple 或真实设备。

## 构建及验证范围

补丁只包含源码，未替换上游仓库已有的 `bin/siboot-aarch64`。
本 fork 的 CI 会用 `scripts/prepare-minis.sh` 应用补丁并执行测试；所有检查通过后，
发布工作流构建 ARM64 musl 静态二进制，生成校验值匹配的 `install-minis.sh`，并与
iOS IPA 一起发布。生成的 Minis 安装器默认安装同一个 Release 的 SideInstaller，
不会安装上游的旧 IPA。

手动构建时可设置 `SIBOOT_DEFAULT_IPA_URL` 指定默认安装的 IPA 下载地址。
上游安装命令仍运行上游已发布的二进制，不会自动包含本补丁。

本次已通过 Rust 全目标测试、Clippy 和 Chromium 手机尺寸下的页面验证
（无效 IPA 拒绝、分块导入、重试保留前序步骤、刷新恢复重试提示）。iOS 应用已通过
翻译检查；当前 Linux 环境没有 Xcode，未进行 iOS 编译或真机签名安装。
