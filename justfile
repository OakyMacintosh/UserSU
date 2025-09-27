alias bu := build_usersu
alias bm := build_manager

build_usersu:
    cross build --target aarch64-linux-android --release --manifest-path ./userspace/usud/Cargo.toml

clippy:
    cargo fmt --manifest-path ./userspace/usud/Cargo.toml
    cross clippy --target x86_64-pc-windows-gnu --release --manifest-path ./userspace/usud/Cargo.toml
    cross clippy --target aarch64-linux-android --release --manifest-path ./userspace/usud/Cargo.toml