# siboot 0.22.5

Open Minis now prepares pairing notifications before you switch to Settings and leaves the current pairing code visible in Minis.

- Wait for the ready message before switching to Settings. The program handles the first iOS notification permission prompt in Minis and checks that the silent audio is playing.
- If Settings shows the six-digit input field and no banner arrives, keep that field open. Return to Minis and read **Pairing code**. Dismiss the large audio panel if it covers the page and keep the small player running. Enter the code in the original field in Settings.
- Notification scheduling runs alongside pairing. The page and live terminal keep the current code visible during the exchange, including after a notification error. The terminal clears its active display when that attempt ends.

The simulated device checks cover first notification permission and pairing when the notification bridge refuses to post. Actual banner presentation depends on iOS and Open Minis. [English installation guide](https://github.com/Jay-Void-BA/SideInstaller/blob/siboot-v0.22.5/README.md#pair-with-the-device) · [简体中文](https://github.com/Jay-Void-BA/SideInstaller/blob/siboot-v0.22.5/README.zh-CN.md#配对设备)

Maintained by **Jay-Void-BA**. Original concept and implementation by **FrizzleM**. [SideInstaller by FrizzleM](https://github.com/FrizzleM/SideInstaller)
