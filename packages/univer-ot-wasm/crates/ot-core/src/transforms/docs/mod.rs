pub mod core_editing_mutation;
pub mod docs_rename_mutation;

use crate::registry::TransformRegistry;

/// Register all docs transforms
///
/// This module handles transforms for document-level mutations:
/// - RichTextEditingMutation: Core editing operations
/// - DocsRenameMutation: Document rename operations
///
/// Note: Document OT has different requirements than sheets OT.
/// These mutations are currently out of scope for the main OT implementation.
pub fn register_transforms(registry: &mut TransformRegistry) {
    core_editing_mutation::register_transforms(registry);
    docs_rename_mutation::register_transforms(registry);
}
