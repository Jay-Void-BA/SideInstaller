# rootshell build-175 host backport

[English usage guide](../../README.md#compatibility) · [简体中文使用说明](../../README.zh-CN.md#compatibility)

This optional patch adds the host capabilities needed for the complete siboot workflow while retaining the build-175 source base. It is for developers compiling rootshell from source. It cannot patch an installed App Store app.

The unmodified older host can run siboot's startup, pairing, and TCP checks. Complete installation and renewal need cooperative receive timeouts; treating the older host's placeholder as success would block its single WASM thread indefinitely.

## Apply

Base: [kitknox/rootshell](https://github.com/kitknox/rootshell) at `2428f29990ab1e1e18dd8bf06c40f9f8a5b11273`, tagged upstream as `1.0.16-175`. The older `1.0.15-172` release has the same Worker ABI. Runtime capability detection handles the version label reported by the installed app.

Download [recv-timeout-175.patch](recv-timeout-175.patch), or the equivalent `rootshell-175-backport.patch` release asset. In a clean rootshell checkout, run:

```sh
git checkout 2428f29990ab1e1e18dd8bf06c40f9f8a5b11273
git apply --check /path/to/rootshell-175-backport.patch
git apply /path/to/rootshell-175-backport.patch
```

Replace the patch path with its actual location. Build and sign rootshell using its own contributor instructions, then run the new siboot WASM inside that app.

## Changes

- Expose `rootshell_socket_set_recv_timeout` through both documented import namespaces.
- Store each socket's receive deadline and return `EAGAIN` only for that pending receive. Match the process and call ID so stale deadlines cannot consume a later request.
- Backport WASI directory cookies and stable, bounded filesystem pages from newer rootshell sources. This prevents older Workers from silently ending a large directory listing after the first buffer.

The original kitknox/rootshell source and its MIT license remain applicable to these host changes. Directory pagination follows upstream commit `3ad9c828d78a2f769ddd8cf77ade0a76cbdfc4ce`; the timeout ABI follows the newer pinned host, with cancellation handling.

## Verification

`scripts/check-rootshell-runtime.sh` checks hashes of the pinned original host files, verifies and applies this patch, and runs the resulting Worker alongside unmodified legacy and newer Workers. The capable-host suite covers complete signing of 320 resources in one directory, TLS rejection, HTTP / WebSocket, short writes, timers, and software TCP.

Worker tests use a local filesystem and socket broker matching the host protocol. They do not compile the complete Swift app or replace an end-to-end installation test on an iPhone.
