# siboot 0.21.0 — rootshell 完整安装器

维护与移植：**Jay-Void-BA**。原作：[FrizzleM/siboot](https://github.com/FrizzleM/siboot) 与 [FrizzleM/SideInstaller](https://github.com/FrizzleM/SideInstaller)。

- 新发布附件 **`siboot.wasm`**，直接在 rootshell 的 WASI 运行时中完成配对、Apple 账户登录、签名、上传和安装，无需 Minis 或 SideInstaller IPA。
- 保留中文／英文、密码显示与隐藏、独立口令加密保存登录及签名缓存、失败步骤重试、配对文件导入和快速续签。
- 配对文件保存在 `siboot.wasm` 同目录。Minis／iSH 中的文件先通过“文件”复制进 rootshell，再在程序中选择。
- 递归签名主程序、扩展、Framework、dylib 和其他 Mach-O；封存并校验 `.config` 等资源。WASI 中保留符号链接元数据，安装时通过 AFC 重建。
- Apple HTTPS 由 rootshell 的原生 TLS 验证证书；隧道使用可协作接收的 TCP 接口，适配单线程运行时。签名不再依赖 Linux 线程池或 mmap。
- 重写 README，以 rootshell 为默认安装方式。仅接受本地 IPA，不检测软件更新，也不下载 SideInstaller IPA。

仍需 **LocalDevVPN、Wi-Fi、开发者模式和网络连接**。免费账号通常需要每 7 天重新签名。rootshell 不代表取得 iOS root 权限，也不保证安装更快；程序会分别显示上传、文件核对和系统安装耗时。

CI 包含共享 Rust 测试、rootshell 测试、WASI 编译，以及固定版本 rootshell 原版 Worker 中的加密存储、CSR、完整 CMS 签名、符号链接、密码切换、续签、配对保存、HTTP／WebSocket 和软件 TCP 传输检查。未完成真实 Apple 账号到 iPhone 安装的端到端验证。

下载后核对本页 SHA-256 或附件 `SHA256SUMS`，确认文件完整再运行。旧版 `siboot-aarch64` 继续保留在 [0.20.0](https://github.com/Jay-Void-BA/SideInstaller/releases/tag/siboot-v0.20.0)；它不能在 rootshell 中运行。
