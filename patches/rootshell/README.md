# rootshell runtime target

siboot supports the unmodified **rootshell 1.0.16 (178)** host. No host patch is required. Build 175 and its former backport are no longer supported; older release assets remain in release history.

The pinned upstream commit is [`192eab95001aef3f2ea4b282c0d3f38a3539fb3f`](https://github.com/kitknox/rootshell/tree/192eab95001aef3f2ea4b282c0d3f38a3539fb3f), tagged `1.0.16-178`.

[`scripts/check-rootshell-runtime.sh`](../../scripts/check-rootshell-runtime.sh) verifies SHA-256 for the original Worker, host page, runtime, socket broker, filesystem broker, and process model. It runs the actual Worker with a local broker that models the single root preopen, mkdir errors, short writes, directory pagination, and 64 KiB JSON/Base64 reply mailbox.

The checks use Node 24 and Bun 1.3.2 to validate both V8 and JavaScriptCore parsing, including truncated modules. They cover first-launch guidance, repeated directory reuse, subdirectory launches, encrypted storage, PKCS#10, recursive CMS signing, links and large resources, renewal, password EOF, HTTP/WebSocket, and cooperative TCP. Download packaging also rejects section overflows and exercises interrupted transfers, corruption, resume, and verification before replacement.

These checks do not execute Network.framework or WKWebView on a real iPhone. A successful Worker test does not constitute an Apple Account-to-device end-to-end installation test. Configuration and integrity-checked downloads are documented in the [English README](../../README.md#quick-start) and [中文说明](../../README.zh-CN.md#quick-start).
