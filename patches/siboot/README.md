# siboot 定制源码：rootshell 与 Minis

维护与移植：**Jay-Void-BA**。原作者：[FrizzleM/siboot](https://github.com/FrizzleM/siboot)。补丁基线为 `82b808de3a407619be7fbe54b3f313932bb80ade`，许可证及原作者信息保留。

[retry-local-ipa.patch](retry-local-ipa.patch) 包含当前 rootshell 完整安装器、保留的 Minis 安装器、共享中文词条及上游仓库文档。共享核心位于本仓库的 `rust-core/`；准备脚本将其接入源码树。

## 准备与构建

在本仓库根目录运行：

```sh
bash scripts/prepare-minis.sh /tmp/siboot-source
rustup target add wasm32-wasip1
cargo build --manifest-path /tmp/siboot-source/rootshell/Cargo.toml --release --locked --target wasm32-wasip1 --bins --examples
cargo test --manifest-path /tmp/siboot-source/rootshell/Cargo.toml --locked --bin siboot
bash scripts/check-rootshell-runtime.sh /tmp/siboot-source/rootshell/target/wasm32-wasip1/release/siboot.wasm /tmp/siboot-source/rootshell/target/wasm32-wasip1/release/examples/runtime_check.wasm
```

脚本名称为兼容已有开发流程保留，准备的源码同时包含 `rootshell/` 与 `minis/`。

## 当前行为

rootshell 0.21.1 内完成本地 IPA 选择、设备配对、Apple 登录、完整签名、上传核对、安装和快速续签。运行时不需要 Minis。使用 rootshell 原生的 socket/TLS、文件和终端 ABI，不执行 Linux ELF，不转发安装任务。

两端保留中文／英文、Jay-Void-BA 与 FrizzleM 署名、密码查看／隐藏、独立口令加密登录及签名缓存、失败步骤重试、原始 IPA 续签缓存、配对文件验证后保存到程序同目录。只接受本地 IPA，移除 SideInstaller IPA 下载入口。

签名核心递归处理主程序、扩展、Framework、dylib 和其他 Mach-O，并封存 `.config` 等资源。rootshell 缺少原生符号链接，WASI 文件层保留链接元数据，签名器按链接封印，AFC 安装时重建。原生平台继续使用操作系统链接、Rayon 和 mmap。

Minis／iSH 与 rootshell 不能直接访问对方的沙盒。已有文件先通过“文件”复制进目标应用，再选择导入。设备连接仍需要 LocalDevVPN。

## 性能诊断

rootshell 新增 `--perf-log`，在 WASM 同目录生成独立的 JSONL 性能日志，记录操作耗时、重试、输入等待、签名与哈希、逐条目上传、安装进度及网络统计。接口仅接受代码定义的操作名和数字，不接收输入、路径、URL 或协议正文。详细事件限制为 16 MiB，正常退出仍保留摘要。组件测试和原版 Worker 检查验证日志落盘与敏感字段过滤。详见主 README。

## CI 与发布

`.github/siboot-release.json` 声明当前版本。`checks.yml` 检查共享核心、Minis 和 rootshell；rootshell 额外进行 WASI 编译及固定版本原版 Worker 中的离线运行测试。`release-siboot.yml` 仅针对通过全部检查的同一源码提交构建，复验后发布 `siboot.wasm`、`SHA256SUMS`，并标记 Latest。

实际 Apple 登录及 iPhone 安装仍需真机验证。Minis 0.20.0 的 `siboot-aarch64` 保留在历史 Release；基线中已有的二进制和安装脚本不作为本 fork 当前的发布成品。
