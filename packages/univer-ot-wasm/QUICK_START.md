# Quick Start Guide

## Installation

Ensure you have Rust 1.70+ installed:

```bash
rustc --version
cargo --version
```

## Building

### For Browser (WASM)

```bash
# Development build (faster compile)
npm run dev

# Production build
npm run build

# Output: pkg/univer_ot_wasm.js
```

### For Server

```bash
# Build server binary
npm run build:server

# Output: target/release/ot-server
```

### For Core Library Only

```bash
# Build the pure Rust library
npm run build:core
```

## Running Tests

```bash
# Run all tests
npm run test

# Run only core tests
npm run test:core

# Run specific test file
cargo test -p ot-core test_insert_row
```

## Common Tasks

### Adding a New Transform

1. Create file: `crates/ot-core/src/transforms/sheets/my_feature.rs`
2. Implement transform functions
3. Register in `crates/ot-core/src/transforms/sheets/mod.rs`
4. Add tests in `crates/ot-core/tests/my_feature_tests.rs`
5. Run tests: `npm run test:core`

### Testing Conflict Resolution

The set_range_values transform has cell-level LWW:

```rust
// When m1 and m2 modify the same cell:
// - m2's value wins (Last-Write-Wins)
// - m1 gets that cell removed from m1_prime
// - m2 keeps the cell

// When they modify different cells:
// - Both succeed
// - No conflict
```

### Checking Transform Count

```bash
npm run test:core -- test_registry_coverage -- --nocapture
```

Output: `Total registered transforms: 137`

## File Structure

```
crates/ot-core/                    # Pure Rust library
├── src/
│   ├── lib.rs                      # Main API
│   ├── registry.rs                 # Transform registry
│   ├── types.rs                    # Core types
│   └── transforms/
│       ├── sheets/
│       │   ├── insert_row.rs
│       │   ├── set_range_values.rs (with LWW)
│       │   └── ...
│       ├── sheets_data_validation/
│       └── sheets_conditional_formatting/
└── tests/
    ├── insert_row_tests.rs
    ├── remove_rows_tests.rs
    ├── set_range_values_tests.rs
    └── registry_size_test.rs

crates/ot-wasm/                    # WASM bindings
├── src/
│   ├── lib.rs
│   ├── types.rs
│   └── service.rs
└── Cargo.toml

crates/ot-server/                  # Server binary
├── src/
│   ├── main.rs
│   ├── database/
│   ├── services/
│   └── handlers/
├── build.rs                        # gRPC proto build
└── Cargo.toml

migration/                         # Database migrations
└── src/

package.json                       # npm scripts
```

## Available npm Scripts

```bash
npm run build              # Build WASM (bundler target)
npm run build:node         # Build WASM (Node.js target)
npm run build:web          # Build WASM (web target)
npm run build:server       # Build server binary
npm run build:core         # Build core library
npm run build:workspace    # Build all crates
npm run test               # Run all tests
npm run test:core          # Run core tests
npm run test:server        # Run server tests
npm run test:web           # Run WASM browser tests
npm run test:coverage      # Generate coverage report
npm run dev                # Development WASM build
npm run clean              # Clean build artifacts
npm run lint               # Run clippy linter
npm run prepare            # Check Rust/Cargo versions
```

## Key Concepts

### Transform Service

```rust
use ot_core::{TransformService, MutationInfo};

let service = TransformService::new();
let result = service.transform(&m1, &m2);
```

### MutationInfo

```rust
pub struct MutationInfo {
    pub id: String,              // "sheet.mutation.insert-row"
    pub params: serde_json::Value,  // mutation parameters
}
```

### Transform Result

```rust
pub struct TransformResult {
    pub m1_prime: Option<MutationInfo>,  // transformed m1
    pub m2_prime: Option<MutationInfo>,  // transformed m2
    pub error: Option<String>,           // error if any
}
```

## Debugging

### Enable verbose output

```bash
RUST_LOG=debug cargo test
```

### Run single test

```bash
cargo test -p ot-core test_insert_row_vs_insert_row_same_position -- --nocapture
```

### Check for warnings

```bash
cargo clippy --workspace --all-targets
```

## Examples

### Using set-range-values with LWW

```rust
// Two clients modify the same cell
let m1 = MutationInfo {
    id: "sheet.mutation.set-range-values".to_string(),
    params: json!({
        "unitId": "doc1",
        "subUnitId": "sheet1",
        "cellValue": {
            "0": { "0": "Client A" }
        }
    }),
};

let m2 = MutationInfo {
    id: "sheet.mutation.set-range-values".to_string(),
    params: json!({
        "unitId": "doc1",
        "subUnitId": "sheet1",
        "cellValue": {
            "0": { "0": "Client B" }  // Same cell!
        }
    }),
};

let result = service.transform(&m1, &m2);
// Result: m1_prime has cell removed, m2_prime keeps it (m2 wins)
```

### Using insert-row with position shifting

```rust
let m1 = MutationInfo {
    id: "sheet.mutation.insert-row".to_string(),
    params: json!({
        "unitId": "doc1",
        "subUnitId": "sheet1",
        "range": { "startRow": 5, "endRow": 5, "startColumn": 0, "endColumn": 10 }
    }),
};

let m2 = MutationInfo {
    id: "sheet.mutation.set-range-values".to_string(),
    params: json!({
        "unitId": "doc1",
        "subUnitId": "sheet1",
        "cellValue": {
            "6": { "0": "value at row 6" }
        }
    }),
};

let result = service.transform(&m1, &m2);
// Result: m2's row reference shifts from 6 to 7 (after insert at 5)
```

## Troubleshooting

### Build fails with proto errors
Make sure the proto files are in the right location:
```bash
ls proto/ot_rpc.proto
```

### Tests fail with registry size
The registry should have 137+ transforms:
```bash
npm run test:core -- test_registry_coverage -- --nocapture
```

### WASM build fails
Try cleaning and rebuilding:
```bash
npm run clean
npm run build
```

### Linker errors on M1 Mac
Make sure you have the right target installed:
```bash
rustup target add wasm32-unknown-unknown
```

## Performance Tips

1. **Build Release Mode** for production WASM:
   ```bash
   npm run build  # uses release profile
   ```

2. **Use Transform Service** - registry lookups are O(1)

3. **Batch Transforms** - apply multiple at once instead of one-by-one

4. **Monitor Registry Size** - should be ~137 transforms

## Resources

- [ARCHITECTURE.md](ARCHITECTURE.md) - Full architecture details
- [OT_IMPLEMENTATION_COMPLETE.md](OT_IMPLEMENTATION_COMPLETE.md) - Implementation status
- [tests/](crates/ot-core/tests/) - Example tests
- [Rust Book](https://doc.rust-lang.org/book/)
- [wasm-bindgen Guide](https://rustwasm.org/docs/wasm-bindgen/)
