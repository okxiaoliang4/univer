# package.json Updates

## Summary

Updated `package.json` to reflect the new workspace architecture with separate crates (`ot-core`, `ot-wasm`, `ot-server`). All feature flags have been eliminated through crate separation.

## Changes Made

### Build Scripts

#### Old Structure
```json
{
  "build": "wasm-pack build --target bundler --out-dir pkg",
  "build:native": "cargo build --release",
  "build:proto": "cargo build --features server"
}
```

#### New Structure
```json
{
  "build": "wasm-pack build crates/ot-wasm --target bundler --out-dir ../../pkg",
  "build:node": "wasm-pack build crates/ot-wasm --target nodejs --out-dir ../../pkg",
  "build:web": "wasm-pack build crates/ot-wasm --target web --out-dir ../../pkg",
  "build:server": "cargo build -p ot-server --release",
  "build:core": "cargo build -p ot-core --release",
  "build:workspace": "cargo build --workspace --release"
}
```

**Key Changes:**
- Explicitly specify `crates/ot-wasm` path for WASM builds
- Removed feature-flag-based `build:proto` command
- Added `build:server` for standalone server binary
- Added `build:core` for pure Rust library
- Added `build:workspace` to build all crates together

### Test Scripts

#### Old Structure
```json
{
  "test": "cargo test"
}
```

#### New Structure
```json
{
  "test": "cargo test --workspace",
  "test:core": "cargo test -p ot-core",
  "test:server": "cargo test -p ot-server",
  "test:web": "wasm-pack test crates/ot-wasm --headless --firefox",
  "test:coverage": "cargo tarpaulin --workspace"
}
```

**Key Changes:**
- Explicit `--workspace` flag for all tests
- Added granular test commands for each crate
- Added coverage testing with tarpaulin
- Separated WASM browser tests

### New Scripts

Added new scripts for better developer experience:

```json
{
  "lint": "cargo clippy --workspace --all-targets"
}
```

## Usage Examples

### Building WASM for Different Targets

```bash
# Browser/Bundler (webpack, etc.)
npm run build

# Browser directly
npm run build:web

# Node.js
npm run build:node

# Development (faster compile, debug info)
npm run dev
```

### Testing Specific Components

```bash
# Test everything
npm run test

# Test core transforms only
npm run test:core

# Test server functionality
npm run test:server

# WASM browser tests
npm run test:web

# Generate coverage report
npm run test:coverage
```

### Building for Different Use Cases

```bash
# Just the core library (no WASM)
npm run build:core

# Standalone server binary
npm run build:server

# Everything
npm run build:workspace

# Check code quality
npm run lint
```

## Benefits

1. **Clear Separation** - Each command targets a specific crate
2. **No Feature Flags** - `--features server` is no longer needed
3. **Better IDE Integration** - IDEs can identify the specific crate being built
4. **Faster Builds** - Can build just what's needed
5. **Granular Testing** - Can test specific components
6. **Better Documentation** - Each command has a clear purpose

## Backward Compatibility

The main `build` and `test` commands still work as before:

```bash
npm run build  # Still builds WASM
npm run test   # Still runs all tests
```

## Migration Guide

If you have scripts that use the old commands:

**Old Way:**
```bash
npm run build:proto  # Error! Command not found
```

**New Way:**
```bash
npm run build:server  # For server binary
npm run build:workspace  # For everything
```

## File Paths

Note that output paths have changed due to new workspace structure:

| Build | Old Output | New Output |
|-------|-----------|-----------|
| WASM | `pkg/` | `pkg/` (via `../../pkg`) |
| Server | `target/release/ot-server` | Same |
| Core | N/A | `target/release/ot_core.rlib` |

## Environment Variables

All standard Cargo environment variables still work:

```bash
# Verbose output
RUST_LOG=debug npm run test:core

# Parallel jobs
CARGO_BUILD_JOBS=4 npm run build:workspace

# Specific Rust version
rustup override set 1.70
```

## CI/CD Integration

These commands are CI/CD friendly:

```yaml
# GitHub Actions example
- name: Test
  run: npm run test

- name: Build WASM
  run: npm run build

- name: Build Server
  run: npm run build:server
```

## Documentation

See also:
- [QUICK_START.md](QUICK_START.md) - Quick reference
- [ARCHITECTURE.md](ARCHITECTURE.md) - Detailed architecture
- [OT_IMPLEMENTATION_COMPLETE.md](OT_IMPLEMENTATION_COMPLETE.md) - Implementation details
