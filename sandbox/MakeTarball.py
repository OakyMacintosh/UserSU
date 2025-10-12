import subprocess as sb
import sys
import os

builtFolder = "../build/tarball"
target = input("Enter llvm compiler name: (eg. aarch64-linux-android) ")
files = [
    f'../system/sysless-root/target/{target}/release/su',
    f'../build/{target}/libfakeroot.so'
]

if os.path.isdir(builtFolder):
    print(f"The path '{builtFolder}' exists!")
    for file in files:
        sb.run(['cp', file, builtFolder])
else:
    sb.run(['mkdir', '-p', builtFolder])
    print(f"Created directory: {builtFolder}")
    for file in files:
        sb.run(['cp', file, builtFolder])

# Change to parent directory for proper tar structure
os.chdir('../build')
sb.run(['tar', '-czvf', 'UserSUAndroidLinux.tar.gz', 'tarball/'])
