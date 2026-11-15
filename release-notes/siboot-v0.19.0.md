仅发布 Minis／iSH 使用的 `siboot-aarch64`，不包含新的 IPA。

- 可选择安全保存 Apple／iCloud 登录。使用独立口令解锁，scrypt 派生密钥、XChaCha20-Poly1305 认证加密；密码、开发者会话、签名私钥及 anisette 缓存均受保护，解锁口令不写入磁盘。支持更新密码、临时使用其他账号及清除保存的登录。
- 页面可浏览 Minis／iSH 内的配对文件，也支持 `--pairing-file PATH`。导入文件须通过密钥格式检查和设备验证，之后以 `0600` 权限原子保存至程序同目录的 `rp_pairing_file.plist`；验证失败保留已有文件。
- 完整递归签名深层 dylib、无扩展名 Mach-O、嵌套 app／appex／framework／xpc；普通 `.config`、隐藏文件和其他资源写入签名清单，并核对最终内容。AFC 上传保留符号链接，检查完整上传后才安装。
- 保留步骤重试、本地 IPA 安装及取消操作。默认 IPA 仍为本仓库已发布的 v2.1.0，可自行选择本地 IPA。

此版本经 Rust 单元测试、Clippy 和 Chromium 手机尺寸页面测试验证。未执行真实 Apple 账号登录或 iPhone 安装测试。
