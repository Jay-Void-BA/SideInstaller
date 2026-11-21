# siboot 0.22.1 · Minis anisette connectivity

Minimal Minis / Alpine installations without a CA bundle could pass the embedded Apple-root preflight while every anisette HTTP client failed during construction. This release fixes that initialization failure without disabling TLS verification.

- Opt the standalone Minis build into bundled Mozilla public root certificates, merged with system/private roots. Apply the same policy to anisette and Apple HTTP clients; native app builds keep platform trust by default.
- Add `--check-anisette`: check default endpoints or `--anisette URL` without IPA selection, device pairing, login storage, provisioning or Apple authentication. Limit concurrency to three and each request to 12 seconds; show specific TLS, DNS and HTTP errors. Optional performance logs contain numeric timings only.
- Preserve failures from every attempted server instead of replacing them with the last error. Treat an anisette server's HTTP 429 as a server failure; retain the stop for Apple account rate limits.
- Exercise the actual ARM64 release without system CAs, reject an unknown TLS issuer, accept a configured private CA, and report HTTP failures through a local TLS fixture. Keep complete signing, resource seals, encrypted login, pairing imports, renewal and upload verification checks.

Update using this release's `install-minis.sh`. Diagnose with:

```sh
"$HOME/.siboot/bin/siboot" --check-anisette --terminal
```

For existing Minis tools, `apk add --no-cache ca-certificates` followed by `update-ca-certificates` installs the system bundle. A successful client-info check does not verify provisioning or Apple login. Device-specific DNS/proxy failures still require the diagnostic output from that device.

Maintained by Jay-Void-BA. Original concept and code: FrizzleM/siboot and SideInstaller by FrizzleM. Existing license terms remain.
