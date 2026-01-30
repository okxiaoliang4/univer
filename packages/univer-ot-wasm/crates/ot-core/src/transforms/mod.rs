pub mod constants;
pub mod sheets;
pub mod sheets_data_validation;
pub mod sheets_conditional_formatting;
pub mod sheets_filter;
pub mod sheets_hyper_link;
pub mod sheets_note;
pub mod sheets_table;
pub mod sheets_pivot_table;
pub mod thread_comment;

use crate::registry::TransformRegistry;

/// Register all transforms into the registry
///
/// This is the single entry point for registering all OT transforms.
/// Each module registers its own transforms following the pattern:
/// - Bidirectional: Single implementation with automatic swap
/// - Symmetric: Same-type transforms
/// - Identity: Non-interfering mutations
pub fn register_all(registry: &mut TransformRegistry) {
    // Register sheets core mutations
    sheets::register_transforms(registry);

    // Register sheets-data-validation mutations
    sheets_data_validation::register_transforms(registry);

    // Register sheets-conditional-formatting mutations
    sheets_conditional_formatting::register_transforms(registry);

    // Register sheets-filter mutations
    sheets_filter::register_transforms(registry);

    // Register sheets-hyper-link mutations
    sheets_hyper_link::register_transforms(registry);

    // Register sheets-note mutations
    sheets_note::register_transforms(registry);

    // Register sheets-table mutations
    sheets_table::register_transforms(registry);

    // Register sheets-pivot-table mutations
    sheets_pivot_table::register_transforms(registry);

    // Register thread-comment mutations
    thread_comment::register_transforms(registry);
}
