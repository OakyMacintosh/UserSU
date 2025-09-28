# UserSU

<img width="100" height="100" alt="UserSU Logo" src="https://github.com/user-attachments/assets/f8d1a3f5-eade-404c-9d2e-710eb66168cf" />

A user-based root solution for Android devices. (+ Any other POSIX device)

[![License: Unlicense](https://img.shields.io/badge/license-Unlicense-blue.svg)](http://unlicense.org/)


## Features

1. User-Managed `su` and root access management.
2. Can be re-compiled anywhere (On Android with the NDK or with `termux-gcc`)
3. Also works on iOS! (needs to be already jailbroken)

## How it works?

UserSU runs in userspace, so, the there's no kernel state changes. It uses a fakeroot library to make applications that use `su` think they're running as root, file system operations are sandboxed onto the `/data/data/me.usersu/files/fs` directory, which has all the Android filesystem structure. So you will be able to change the `system` partition without really modifing the device's partition, so it's way more secure, no risks of breaking your device, no flashing `boot_a`, no bootloader unlocking headache, just instant root! It also works together with [Shizuku](https://github.com/rikkaapps/shizuku).

>[!NOTE]
> UserSU still has no Manager app, so some tasks are completely manual!

>[!NOTE]
> UserSU is on constant development.

**UserSU** uses the same root-emulation context as [OpenRoot](https://github.com/oakymacintosh/openroot) used.