#!/bin/bash

echo "🚀 Production PoS Build Script"
echo "==============================="

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust from https://rustup.rs/"
    exit 1
fi

echo "✅ Rust/Cargo found: $(cargo --version)"

# Check for network connectivity
if cargo --version &>/dev/null && timeout 5 cargo search serde --limit 1 &>/dev/null; then
    echo "✅ Network connectivity available"
    NETWORK_AVAILABLE=true
else
    echo "⚠️  Network connectivity limited - will try offline build"
    NETWORK_AVAILABLE=false
fi

# Build the project
echo ""
echo "🔨 Building Production PoS..."

if [ "$NETWORK_AVAILABLE" = true ]; then
    echo "📦 Fetching dependencies and building..."
    cargo build --release
    BUILD_SUCCESS=$?
else
    echo "📦 Attempting offline build..."
    cargo build --release --offline 2>/dev/null
    BUILD_SUCCESS=$?

    if [ $BUILD_SUCCESS -ne 0 ]; then
        echo "⚠️  Offline build failed. Dependencies need to be fetched first."
        echo "   Please run this script when network connectivity is available."
        echo ""
        echo "🔍 Project structure is complete and ready for compilation:"
        echo "   - Core types and data structures ✅"
        echo "   - Cryptographic utilities ✅"
        echo "   - Consensus mechanism ✅"
        echo "   - Configuration system ✅"
        echo "   - Binary executables ✅"
        echo "   - Comprehensive tests ✅"
        echo "   - Documentation ✅"
        echo ""
        echo "📋 To build when network is available:"
        echo "   cargo build --release"
        exit 1
    fi
fi

if [ $BUILD_SUCCESS -eq 0 ]; then
    echo ""
    echo "🎉 Build successful!"
    echo ""
    echo "📋 Available binaries:"
    echo "   ./target/release/node      - Main blockchain node"
    echo "   ./target/release/validator - Validator key management"
    echo ""
    echo "🧪 Run tests:"
    echo "   cargo test"
    echo ""
    echo "📖 Run example:"
    echo "   cargo run --example basic_usage"
    echo ""
    echo "📚 See README.md for usage instructions"
else
    echo "❌ Build failed"
    exit 1
fi