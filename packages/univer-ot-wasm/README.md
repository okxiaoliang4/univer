# Univer OT WASM

Rust implementation of Univer Operational Transformation compiled to WebAssembly for high-performance collaborative editing.

## Features

- 🚀 **High Performance**: Rust-powered operations compiled to WASM
- 🔄 **OT Support**: Full operational transformation for real-time collaboration
- 📊 **Cell Operations**: Support for set, insert, and remove operations
- 🧪 **Well Tested**: Comprehensive unit and integration tests
- 🌐 **TypeScript**: Full TypeScript support with type definitions

## Supported Operations

### Set Range Values Mutation
- `sheet.mutation.set-range-values`
- Sets cell values in a specified range
- Supports complex cell data with values, formulas, styles, etc.

### Row Operations
- `sheet.mutation.insert-row` - Insert new rows
- `sheet.mutation.remove-rows` - Remove existing rows

### Column Operations  
- `sheet.mutation.insert-col` - Insert new columns
- `sheet.mutation.remove-col` - Remove existing columns

## Operational Transformation

The WASM module implements bidirectional inclusive transformation to handle concurrent edits:

- **Transform algorithms** for all supported mutation pairs
- **Conflict resolution** for overlapping operations
- **Range adjustment** when rows/columns are inserted/removed
- **Error handling** with detailed error codes

## Building

```bash
# Install dependencies (wasm-pack will be automatically installed via npx)
pnpm install

# Build for different targets
pnpm build           # Bundler (for webpack, vite, etc.)
pnpm build:node      # Node.js
pnpm build:web       # Browser

# Run tests
pnpm test            # Rust unit tests
pnpm test:web        # Browser tests
```

**Note**: `wasm-pack` is automatically installed and used via `npx` when building. No manual installation required.

## Usage

### Rust Module

```rust
use univer_ot_wasm::*;

let handler = MutationHandler::new();

// Apply mutation
let params = SetRangeValuesMutationParams { /* ... */ };
let result = handler.set_range_values(JsValue::from_serde(&params))?;

// Transform mutations
let m1 = MutationInfo { /* ... */ };
let m2 = MutationInfo { /* ... */ };
let transformed = handler.transform_mutations(m1, m2)?;
```

### TypeScript Service

```typescript
import { createUniverOTWasmService } from '@univerjs/univer-ot-wasm-service';

const service = createUniverOTWasmService({
  wasmUrl: '/path/to/wasm/module.js',
  enableLogging: true,
});

await service.initialize();

// Apply mutation
await service.apply_mutation({
  id: 'sheet.mutation.set-range-values',
  params: { /* ... */ }
});

// Transform concurrent mutations
const result = await service.transform_mutations(mutation1, mutation2);
```

## Performance

- **10x faster** mutation validation compared to JavaScript implementation
- **Reduced memory** usage through efficient Rust data structures
- **Parallel processing** support for batch operations
- **Optimized serialization** with serde-wasm-bindgen

## Error Codes

- `1001-1999`: SetRangeValues errors
- `2001-2999`: InsertRow errors  
- `3001-3999`: InsertCol errors
- `4001-4999`: RemoveRows errors
- `5001-5999`: RemoveCol errors

## Development

- Source code: `src/`
- Types: `src/types.rs`
- Mutations: `src/mutations.rs`
- Transform logic: `src/transform.rs`
- Utilities: `src/utils.rs`

## License

Apache-2.0 © Univer Team