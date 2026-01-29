//! OT WASM - WebAssembly Bindings for Operational Transformation
//!
//! This crate provides WASM bindings for the ot-core library.
//! It handles the boundary conversion between JavaScript (JsValue) and
//! Rust (serde_json::Value) types.
//!
//! # Architecture
//!
//! - **ot-core**: Pure Rust OT logic (no WASM dependencies)
//! - **ot-wasm**: Thin WASM binding layer (this crate)
//!
//! The separation ensures:
//! 1. Core logic can be tested without WASM
//! 2. Core logic can be reused in server environments
//! 3. Minimal WASM overhead - only at the boundary
//!
//! # Usage from JavaScript
//!
//! ```javascript
//! import init, { TransformService } from './pkg/ot_wasm.js';
//!
//! await init();
//!
//! const service = new TransformService();
//!
//! const m1 = {
//!     id: 'sheet.mutation.insert-row',
//!     params: {
//!         unitId: 'workbook1',
//!         subUnitId: 'sheet1',
//!         range: { startRow: 5, startColumn: 0, endRow: 5, endColumn: 10 }
//!     }
//! };
//!
//! const m2 = {
//!     id: 'sheet.mutation.set-range-values',
//!     params: {
//!         unitId: 'workbook1',
//!         subUnitId: 'sheet1',
//!         cellValue: {}
//!     }
//! };
//!
//! const result = service.transform(m1, m2);
//! console.log(result.m1_prime, result.m2_prime);
//! ```

use wasm_bindgen::prelude::*;

mod types;
mod service;

pub use types::*;
pub use service::*;

// When the `wee_alloc` feature is enabled, use lol_alloc as the global allocator
#[cfg(target_arch = "wasm32")]
use lol_alloc::{FreeListAllocator, LockedAllocator};

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOCATOR: LockedAllocator<FreeListAllocator> =
    LockedAllocator::new(FreeListAllocator::new());

// Define console_error_panic_hook for better error messages in development
#[cfg(feature = "console_error_panic_hook")]
pub use console_error_panic_hook::set_once as set_panic_hook;

#[cfg(not(feature = "console_error_panic_hook"))]
#[inline]
pub fn set_panic_hook() {}

/// Initialize the WASM module (optional, for setup tasks)
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        init();
    }
}
