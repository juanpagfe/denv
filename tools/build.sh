#!/bin/bash
# Build all compiled tools (Rust and Go projects under tools/)
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUT_DIR="$SCRIPT_DIR/bin"
mkdir -p "$OUT_DIR"

RUST_TOOLS=(dupe pack netwatch tunnel)
GO_TOOLS=(sync timeline)

built=0
failed=0

# Rust tools
for tool in "${RUST_TOOLS[@]}"; do
    if [ -d "$SCRIPT_DIR/$tool/src" ]; then
        echo "Building $tool (Rust)..."
        if cargo build --release --manifest-path "$SCRIPT_DIR/$tool/Cargo.toml" 2>&1; then
            cp "$SCRIPT_DIR/$tool/target/release/$tool" "$OUT_DIR/$tool"
            echo "  ✓ $tool built successfully"
            built=$((built + 1))
        else
            echo "  ✗ $tool failed to build"
            failed=$((failed + 1))
        fi
    fi
done

# Go tools
for tool in "${GO_TOOLS[@]}"; do
    if [ -d "$SCRIPT_DIR/$tool" ] && [ -f "$SCRIPT_DIR/$tool/go.mod" ]; then
        echo "Building $tool (Go)..."
        if (cd "$SCRIPT_DIR/$tool" && go build -o "$OUT_DIR/$tool" .) 2>&1; then
            echo "  ✓ $tool built successfully"
            built=$((built + 1))
        else
            echo "  ✗ $tool failed to build"
            failed=$((failed + 1))
        fi
    fi
done

echo ""
echo "Build complete: $built succeeded, $failed failed"
echo "Binaries in: $OUT_DIR/"

if [ "$failed" -gt 0 ]; then
    exit 1
fi
