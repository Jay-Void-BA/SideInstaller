# siboot 0.22.3 · Verification-code reporting

When Apple rejected a text or call request, Minis previously claimed that a code had been sent to Apple devices. An HTTP-successful trusted-device request could also hide an Apple error in its body. This release reports the request outcome without claiming delivery.

- Preserve Apple plist/JSON error codes in trusted-device requests, including errors returned with HTTP 200; retain phone service errors on successful HTTP status as well.
- Replace delivery claims with neutral verification prompts. Add Chinese warnings, same-account guidance and a shortcut hint for trusted-number text/call options. Explain when Apple supplies no trusted phone number for the session.
- Keep the warning when returning from Other ways. Opening the menu or returning to the code boxes does not request another code.
- Log request HTTP statuses and trusted-number counts without verification codes or protocol bodies. Add regression checks for rejected and unconfirmed responses and the prompt wording.

Download this release’s new `install-minis.sh`, run it and check `"$HOME/.siboot/bin/siboot" --version` reports **0.22.3**. Upgrading preserves encrypted login and signing data. If no code arrives, check that the device uses the same Apple Account, or choose Other ways to request a text/call to an available trusted number. You can cancel and explicitly select a server with `--anisette https://new.ani.sidestore.io`.

These changes fix error handling and diagnostics; they cannot confirm or force Apple’s code delivery. Automated and offline checks do not replace real-account, real-device verification.

Maintained by Jay-Void-BA; original concept and code by FrizzleM. Existing licenses remain.
