#!/bin/bash
set -euo pipefail

echo "========================================="
echo " AD Skipper - 一键环境搭建 & 打包"
echo "========================================="
echo ""

RUST_DIR="$(cd "$(dirname "$0")/rust" && pwd)"
APP_DIR="$(cd "$(dirname "$0")/app" && pwd)"
PROJECT_DIR="$(dirname "$0")"

NEED_INSTALL=0

# --- 检查 Rust ---
if command -v rustup &>/dev/null; then
    echo "[OK] Rust: $(rustc --version 2>/dev/null || echo 'found')"
else
    echo "[MISS] Rust not found"
    NEED_INSTALL=1
fi

# --- 检查 Java ---
if command -v java &>/dev/null; then
    echo "[OK] Java: $(java -version 2>&1 | head -1)"
else
    echo "[MISS] Java not found"
    NEED_INSTALL=1
fi

# --- 检查 Android SDK ---
if [ -n "${ANDROID_HOME:-}" ] || [ -n "${ANDROID_SDK_ROOT:-}" ]; then
    echo "[OK] Android SDK found"
else
    echo "[MISS] Android SDK not found (set ANDROID_HOME)"
    NEED_INSTALL=1
fi

if [ "$NEED_INSTALL" = "1" ]; then
    echo ""
    echo "=== 请先安装缺失的工具 ==="
    echo ""
    echo "1. 安装 Rust:"
    echo "   https://rustup.rs"
    echo "   winget install Rustlang.Rustup"
    echo ""
    echo "2. 安装 JDK 17+:"
    echo "   winget install Microsoft.OpenJDK.17"
    echo ""
    echo "3. 安装 Android SDK:"
    echo "   安装 Android Studio 或用 sdkmanager 命令行"
    echo "   https://developer.android.com/studio"
    echo ""
    echo "   设置环境变量:"
    echo "   ANDROID_HOME=C:\\Users\\Administrator\\AppData\\Local\\Android\\Sdk"
    echo ""
    exit 1
fi

echo ""
echo "=== Step 1/3: 安装 Rust Android 目标 ==="
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android

echo ""
echo "=== Step 2/3: 编译 Rust 原生库 ==="
if ! command -v cargo-ndk &>/dev/null; then
    cargo install cargo-ndk
fi

cd "$RUST_DIR"
cargo ndk \
    -t arm64-v8a \
    -t armeabi-v7a \
    -t x86_64 \
    -o "$APP_DIR/src/main/jniLibs" \
    build --release

echo ""
echo "=== Step 3/3: 打包 APK ==="
cd "$PROJECT_DIR"
if [ ! -f gradle/wrapper/gradle-wrapper.jar ]; then
    echo "需要先生成 Gradle Wrapper..."
    if command -v gradle &>/dev/null; then
        gradle wrapper
    else
        echo "请在有 gradle 的机器上运行 'gradle wrapper' 生成 gradle-wrapper.jar"
        echo "或者从这里下载: https://services.gradle.org/distributions/gradle-8.5-bin.zip"
        exit 1
    fi
fi

./gradlew assembleDebug

echo ""
echo "=== 完成! ==="
echo "APK 位置: app/build/outputs/apk/debug/app-debug.apk"
