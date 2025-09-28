cc := gcc
cflags := -Wall -Wextra -Werror -O2
ldflags := -static

usud := ./userspace/usud

build-fakeroot:
    $(cc) $(cflags) $(usud)/src/clang/fakeroot.c -o build/libfakeroot.so $(ldflags)
    @echo "Built libfakeroot.so"

build-sud:
    cargo build --release --manifest-path $(usud)/Cargo.toml
    cargo build --release --manifest-path $(usud)/Cargo.toml --bin su --target aarch64-linux-android
    cargo build --release --manifest-path $(usud)/Cargo.toml --bin usersu --target aarch64-linux-android
    