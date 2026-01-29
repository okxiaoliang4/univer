use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(all(feature = "wee_alloc", target_arch = "wasm32"))]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Define console_error_panic_hook for better error messages in development
#[cfg(feature = "console_error_panic_hook")]
pub use console_error_panic_hook::set_once as set_panic_hook;

#[cfg(not(feature = "console_error_panic_hook"))]
#[inline]
pub fn set_panic_hook() {}

// Export a `greet` function from Rust to JavaScript, that alerts a hello message.
#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

mod mutations;
mod transform;
mod types;
mod utils;

#[cfg(feature = "server")]
mod server;

pub use mutations::types::{
    ColumnData, InsertColMutationParams, InsertRowMutationParams, RemoveColMutationParams,
    RemoveRowsMutationParams, RowData, SetRangeValuesMutationParams,
};
pub use mutations::{
    InsertColMutation, InsertRowMutation, RemoveColMutation, RemoveRowsMutation,
    SetRangeValuesMutation,
};
pub use transform::TransformService;
pub use types::*;
pub use utils::*;
