#RUSTFLAGS='-C target-feature=+crt-static' cargo build --release --target x86_64-unknown-linux-musl
cargo build --release
docker build . -t operator-rs:latest