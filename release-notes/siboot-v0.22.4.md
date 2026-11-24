# siboot 0.22.4

Update Apple sign-in compatibility in Open Minis and expand the setup, recovery, and developer guides.

- Use the newer Mac client information from FrizzleM's sign-in repair, with the `Mac OS X` spelling and its matching user agent. The GrandSlam Mac fallback uses the same pair. [Upstream change](https://github.com/FrizzleM/SideInstaller/commit/9e8d7c167ebfd6347086aa661d659c01371468b5)
- Match the stored anisette identity to the client used for full password sign-in. This avoids reusing a Catalina or differently named identity that may cause Apple error `-22410`. Earlier identities remain available for saved sessions. Provisioning a new identity can prompt for a verification code.
- Add detailed English and Simplified Chinese instructions for verified downloads, pairing, local IPA installation, renewal, and diagnostics. If you use iCloud Advanced Data Protection, try this release with protection enabled. The guides describe the temporary Settings fallback for accounts that still cannot verify. [English](https://github.com/Jay-Void-BA/SideInstaller/blob/siboot-v0.22.4/README.md#icloud-advanced-data-protection) · [简体中文](https://github.com/Jay-Void-BA/SideInstaller/blob/siboot-v0.22.4/README.zh-CN.md#icloud-高级数据保护)

Download this release's `install-minis.sh` and run it in Open Minis. Verify the installed program with `sh install-minis.sh --verify` and `"$HOME/.siboot/bin/siboot" --version`. Keep the saved state and pairing when upgrading.

A report from iLoader confirms that a related client update supported sign-in with Advanced Data Protection enabled. Minis CI checks headers, saved identity compatibility, and offline signing. Real-account verification on an iPhone or iPad remains to be checked. [iLoader report](https://github.com/nab138/iloader/issues/1203#issuecomment-7080551523)

Maintained by **Jay-Void-BA**. Original concept and implementation by **FrizzleM**. [SideInstaller by FrizzleM](https://github.com/FrizzleM/SideInstaller)
