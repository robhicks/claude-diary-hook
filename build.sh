#!/bin/bash

set -e

echo "🦀 Building Claude Diary Hook..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed. Please install from https://rustup.rs/"
    exit 1
fi

# Build the project
echo "📦 Compiling..."
cargo build --release

# Make executable
chmod +x target/release/claude-diary-hook

echo "✅ Build complete!"
echo "📍 Binary location: $(pwd)/target/release/claude-diary-hook"

# Test the binary
echo "🧪 Running basic test..."
if echo '{"event_type": "test", "user_prompt": "Build test"}' | ./target/release/claude-diary-hook --test > /dev/null; then
    echo "✅ Basic test passed!"
else 
    echo "❌ Basic test failed!"
    exit 1
fi

echo ""
echo "🎉 Claude Diary Hook is ready to use!"
echo ""
echo "Next steps:"
echo "1. Add to your Claude Code hooks configuration:"
echo '   {"hooks": {"user-prompt-submit": "'$(pwd)'/target/release/claude-diary-hook"}}'
echo ""
echo "2. Test with: ./test.sh"
echo "3. View README.md for full documentation"