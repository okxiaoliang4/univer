#!/bin/bash

# OT Core Memory Test Runner
# Tests memory usage of set-range-values operations with 20,000 cells

set -e

cd "$(dirname "$0")"

echo "=================================="
echo "OT Core Memory Test"
echo "=================================="
echo ""

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo not found. Please install Rust."
    exit 1
fi

# Parse arguments
MODE="release"
CELLS=20000
MUTATIONS=5

while [[ $# -gt 0 ]]; do
    case $1 in
        --debug)
            MODE="debug"
            shift
            ;;
        --cells)
            CELLS="$2"
            shift 2
            ;;
        --mutations)
            MUTATIONS="$2"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --debug         Run in debug mode (default: release)"
            echo "  --cells NUM     Number of cells per mutation (default: 20000)"
            echo "  --mutations NUM Number of mutations to generate (default: 5)"
            echo "  --help          Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0                           # Run with default settings"
            echo "  $0 --debug                   # Run in debug mode"
            echo "  $0 --cells 50000             # Test with 50,000 cells"
            echo "  $0 --mutations 10            # Generate 10 mutations"
            echo ""
            echo "To stop: Press Ctrl+C"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Run with --help for usage information"
            exit 1
            ;;
    esac
done

echo "Configuration:"
echo "  Mode: $MODE"
echo "  Cells per mutation: $CELLS"
echo "  Number of mutations: $MUTATIONS"
echo ""

# Build and run
echo "Building (this may take a moment)..."
if [ "$MODE" = "release" ]; then
    cargo build --example memory_test --release --quiet
    echo "Running memory test in release mode..."
    cargo run --example memory_test --release
else
    cargo build --example memory_test --quiet
    echo "Running memory test in debug mode..."
    cargo run --example memory_test
fi
