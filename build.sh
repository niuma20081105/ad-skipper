#!/usr/bin/env bash
set -euo pipefail

echo "==> Installing Android Rust targets..."
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android 2>/dev/null || true

echo "==> Installing cargo-ndk..."
cargo install cargo-ndk 2>/dev/null || true

echo "==> Building Rust native libraries..."
cd "$(dirname "$0")/rust"
cargo ndk \
    -t arm64-v8a \
    -t armeabi-v7a \
    -t x86_64 \
    -o ../app/src/main/jniLibs \
    build --release

echo "==> Done! Native libs placed in app/src/main/jniLibs/"
echo "==> Now run: cd ../ && ./gradlew assembleDebug"
