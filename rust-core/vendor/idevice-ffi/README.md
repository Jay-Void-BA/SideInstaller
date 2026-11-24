# idevice-ffi (vendored)

Copy of the `ffi/` crate from
[jkcoxson/idevice](https://github.com/jkcoxson/idevice) @
`7bd551c16c6dd2e058740d85a2d9399a51a776e9` — the same revision as
`vendor/idevice` and the git dependency in `rust-core/Cargo.toml`. It is the C
API StikDebug ships; `rust-core/src/lib.rs` re-exports its `#[no_mangle]`
symbols into SideInstaller's static library, and `DeviceConnection.swift` calls
them directly.

## Consumers and validation

The iOS app reaches the vendored device library through this C ABI. The Minis installer uses the Rust API directly and uses its own native tunnel setup. Keep the Rust and C copies on the same idevice revision when updating the shared code. [Rust integration](../idevice/README.md)

`build.rs` uses cbindgen to generate this crate's `idevice.h`. For ABI changes, build the Rust core, review the generated declarations, and update the copy in `rust-core/include/idevice.h` used by Swift. Check both the Swift callers and ownership of any pointers or handles crossing the boundary.

Run `LOCKED=1 bash scripts/test-rust.sh` from the project root for the shared Rust paths. Run `bash scripts/check-ios.sh` on a Mac with the CI's Xcode and available simulator to verify header imports, linking, and Swift tests. See the iOS job for tool versions. [iOS CI job](../../../.github/workflows/checks.yml)

## Local changes

The changes below cover `Cargo.toml`, `build.rs`, and `src/tunnel_provider.rs` and the generated C header. Compare against the pinned upstream revision when updating the vendored code.

**1. `Cargo.toml` — `crate-type = ["rlib"]`.**

Upstream builds `staticlib`, `cdylib` and `rlib` in one pass. The standalone
`cdylib` link fails for the iOS *device* target, and SideInstaller only needs
the `rlib` that `rust-core` links into its own static library.

**2. `Cargo.toml` — `idevice` comes from the pinned git revision.**

Upstream points at `path = "../idevice"` inside its workspace. This copy uses the
git URL and revision from `rust-core/Cargo.toml`'s direct dependency, so the
`[patch."https://github.com/jkcoxson/idevice.git"]` there redirects both to
`vendor/idevice` and cargo builds one `idevice`.

**3. `build.rs` — the header copy into `../cpp/include` can't fail the build.**

Upstream writes the generated header to `../cpp/include/idevice.h` and panics if
that folder is missing. Here it is created if needed and a failed write is
ignored. That copy is `rust-core/vendor/cpp/include/idevice.h`; the header Swift
actually includes is `rust-core/include/idevice.h`, copied from this crate's
`idevice.h` by hand when the API changes (only some doc-comment wording differs
today).

**4. `src/tunnel_provider.rs` — `tunnel_create_rppairing_multihost`, with a
classified failure.**

About 600 added lines (commits `fe7e3cf`, `2dce8ac` and `4882060`, 2026-08).
`createListener` tells the host which port the device opened for the tunnel but
not which address reaches it. Upstream dials the address RSD was reached on and
gives up on the first error. On a loopback VPN that forwards the RSD port but not
the new one, that error says nothing useful.

- `tunnel_create_rppairing_multihost` takes extra candidate hosts from the caller
  (in practice the local address of the interface pairing ran over) and an
  optional `out_failure_kind`. `tunnel_create_rppairing` keeps its signature and
  calls it with neither.
- Candidates, best first and without duplicates: the host that worked earlier in
  this process, the RSD peer address, `127.0.0.1`, then the caller's extras.
- The list is swept breadth-first (one attempt per candidate, then another
  round), up to 6 rounds, 800 ms per `connect()`, 700 ms between rounds, 18 s in
  all. Depth-first spent long enough on dead candidates that the device's
  listener had gone stale by the time the live one was dialled.
- When a candidate accepts the connection but the TLS-PSK/CDTunnel handshake
  fails, the device is asked for a new listener and the same host is dialled
  again on the new port, at most 3 handshakes per sweep.
- `TunnelFailureKind` (exported to C) says which step failed: RSD unreachable,
  pair-verify refused, tunnel port refused on the host RSD answered on, timed
  out, TLS handshake failed, or other. `DeviceConnection.swift` picks its advice
  from it. The raw error is still in the returned `IdeviceFfiError`.
- Every attempt and its errno is logged at INFO/WARN as it happens, and the
  failure ends with a one-line summary per candidate.

`idevice.h` here, `vendor/cpp/include/idevice.h` and `rust-core/include/idevice.h`
carry the new function and enum.

## Re-vendoring

None of these changes is upstream as of `master` @ `8970209` (2026-10-27).
Re-vendor from the same revision as `vendor/idevice`, keep changes 1 and 2 or the
build breaks, and carry change 4 over or the Swift side stops compiling.
Regenerate `idevice.h` and copy it to `rust-core/include/` afterwards.
