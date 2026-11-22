# siboot 0.22.2 · Apple sign-in compatibility

Incorporate the [upstream November 21 sign-in repair by FrizzleM](https://github.com/FrizzleM/SideInstaller/commit/8c13fceefebe8fec7475555a9ea50a7df0c372d8). Upstream identified Apple's November 20 HTTP 503 response to clients naming their OS `macOS`; the previous Minis client and Apple fallback still sent those identifiers.

- Use the compatible Catalina `Mac OS X;10.15.7` client and user agent for built-in macOS-ADI sign-in and the corresponding Apple fallback.
- Preserve existing modern-Mac provisioned identities. Add `anisette.lillq.me` to the fallback list and skip a pool identity that another server reports as `adi_pb too large`, without deleting other identities.
- Show actionable Apple/developer codes and HTTP errors instead of only `login failed`. Distinguish sign-in setup from authentication, and check URL-bag HTTP status before parsing its body.
- Add regression checks for compatible headers, existing-identity reuse, pool isolation, HTTP 503 reporting and error summaries. Retain TLS certificate verification, encrypted storage, complete signing, upload checks and private retry workspaces.

Download this version's new `install-minis.sh`, run it, then verify the actual program with:

```sh
"$HOME/.siboot/bin/siboot" --version
```

It must report **0.22.2**. An old `./siboot-aarch64` file remains a separate program; use the new installed path. Your encrypted login and signing data are preserved. A historical 0.20.0 log that completed login before developer error 1100 describes a signing-session failure; its shared extraction-directory retry error is fixed in the current runtime.

The compatibility change follows upstream's observed protocol behavior. Offline and ARM64 CI checks do not establish successful authentication for a real Apple Account; real-device sign-in still needs validation.

Maintained by Jay-Void-BA; original concept and code by FrizzleM. Existing licenses remain.
