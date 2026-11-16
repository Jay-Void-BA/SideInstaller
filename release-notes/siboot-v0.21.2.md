# siboot 0.21.2 — rootshell ABI compatibility

Maintained by Jay-Void-BA. Original projects and design: FrizzleM/siboot and SideInstaller by FrizzleM.

- Fix the startup failure `import function rootshell_socket:rootshell_socket_set_recv_timeout must be callable` on older rootshell Workers, including the ABI used by 1.0.15 / build 175.
- Probe the optional API using an invalid descriptor. A legacy zero-returning placeholder is never treated as working timeout support.
- Older hosts can launch, show help and menus, pair, and perform TCP reachability checks. **Complete installation, renewal, device tunnels, and Developer Mode operations still require host receive-timeout support.** Missing support is reported before Apple sign-in instead of allowing an unbounded receive to freeze the installer.
- For full functionality, update rootshell to a build providing the API, or rebuild build 175 with the included `rootshell-175-backport.patch`. The patch also backports directory pagination to preserve complete signing of large bundles. It does not modify an installed App Store app.
- CI uses unmodified legacy and modern Workers plus the backported Worker. Checks cover startup, capability failures, encrypted storage, recursive CMS signing, symbolic links, 320-file directory sealing, HTTP / WebSocket, and cooperative software TCP.

Download `siboot.wasm` for normal use. The `.patch` asset is optional and intended for developers compiling rootshell. Verify downloads against `SHA256SUMS`.

Real Apple Account-to-iPhone installation and a full Swift build of the backported host still require device/build validation.
