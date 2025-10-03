alias b := build
flakeman := "nix"
aarch64 := "aarch64-unknown-linux-gnu-g++"
armv7l := "armv7l-unknown-linux-gnueabihf-g++"

build:
    if test ! -e build; then mkdir build; fi
    # build libfakeroot
    {{ aarch64 }} -shared -fPIC -O2 -std=c++17 -D__stub_defined=1  -o build/lib/libfakeroot.aarch64.so ./system/sysless-root/src/clang++/libfakeroot.cxx
    {{ armv7l }} -shared -fPIC -O2 -std=c++17 -D__stub_defined=1 -o build/lib/libfakeroot.armv7l.so ./system/sysless-root/src/clang++/libfakeroot.cxx