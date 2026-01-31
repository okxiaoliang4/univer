pub mod constants;

// Core sheets transforms
pub mod sheets;

// Feature plugin transforms (aligned with mutations directory)
pub mod data_validation;
pub mod docs;
pub mod docs_hyper_link;
pub mod engine_formula;
pub mod sheets_conditional_formatting;
pub mod sheets_drawing;
pub mod sheets_filter;
pub mod sheets_hyper_link;
pub mod sheets_note;
pub mod sheets_numfmt;
pub mod sheets_pivot_table;
pub mod sheets_table;
pub mod thread_comment;

use crate::registry::TransformRegistry;

/// Register all transforms into the registry
///
/// This is the single entry point for registering all OT transforms.
/// Each module registers its own transforms following the pattern:
/// - Bidirectional: Single implementation with automatic swap
/// - Symmetric: Same-type transforms
/// - Identity: Non-interfering mutations
///
/// Module order matches mutations directory structure for consistency.
pub fn register_all(registry: &mut TransformRegistry) {
    // Core sheets mutations
    sheets::register_transforms(registry);

    // Feature plugin mutations (in alphabetical order, matching mutations/mod.rs)
    data_validation::register_transforms(registry);
    docs::register_transforms(registry);
    docs_hyper_link::register_transforms(registry);
    engine_formula::register_transforms(registry);
    sheets_conditional_formatting::register_transforms(registry);
    sheets_drawing::register_transforms(registry);
    sheets_filter::register_transforms(registry);
    sheets_hyper_link::register_transforms(registry);
    sheets_note::register_transforms(registry);
    sheets_numfmt::register_transforms(registry);
    sheets_pivot_table::register_transforms(registry);
    sheets_table::register_transforms(registry);
    thread_comment::register_transforms(registry);
}
