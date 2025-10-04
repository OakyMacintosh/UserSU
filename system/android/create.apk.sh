#!/usr/bin/env bash

set -e

# check if build directory exists, and the libfakeroot is compiled
if [ ! -d ../../build/lib ]; then
    echo "AHEM, pls run just on the root of the project first."
    exit 1    
fi

cp ../../build/lib/libfakeroot.aarch64.so ./main/jniLibs/aarch64/libfakeroot.so
cp ../../build/lib/libfakeroot.armv7l.so ./main/jniLibs/armv7l/libfakeroot.so
cp ../sysless-root/target/aarch64-linux-android/release/su ./main/assets/aarch64/su
cp ../sysless-root/target/armv7-linux-androideabi/release/su ./main/assets/armv7l/su

# check if gradle is installed

if [ ! command -v gradle >/dev/null 2>&1 ]; then
    echo "Pls install gradle bro."
    exit 1
fi

gradle build