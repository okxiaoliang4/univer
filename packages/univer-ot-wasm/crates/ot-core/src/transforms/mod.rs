pub mod sheets;
pub mod sheets_data_validation;
pub mod sheets_conditional_formatting;

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
}
