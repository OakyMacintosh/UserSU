# UserSU Sandbox Rootfs generator

```sh
# Install dependencies in Termux first
pkg install python proot
pip install typer

# Create a new sandbox (automatically copies Android binaries)
python generateRootfs.py create ./my-sandbox

# Update binaries later if needed
python generateRootfs.py update-binaries ./my-sandbox

# Enter the sandbox as root
python generateRootfs.py enter ./my-sandbox

# With custom bind mounts (e.g., access sdcard)
python generateRootfs.py enter ./my-sandbox -b /sdcard:/sdcard

# Get info about the sandbox
python generateRootfs.py info ./my-sandbox
```

You can just run `adb shell` and then
```sh
/data/data/com.termux/files/usr/bin/bash
```
which will start bash inside the termux enviroment.