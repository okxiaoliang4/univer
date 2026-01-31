use crate::registry::TransformRegistry;

/// Register transforms for RichTextEditingMutation
///
/// Mutation ID: docs.mutation.rich-text-editing (or similar)
///
/// Note: Document OT typically uses a different algorithm than sheets OT.
/// Rich text editing requires character-level OT, often based on CRDT algorithms
/// like Yjs, Automerge, or operational transformation specifically designed for text.
///
/// This is currently out of scope for the sheets-focused OT implementation.
pub fn register_transforms(_registry: &mut TransformRegistry) {
    // TODO: Implement document-specific OT transforms
    // Consider using established document OT libraries or algorithms:
    // - Operational Transformation for rich text (similar to Google Docs)
    // - CRDT-based approaches (Yjs, Automerge)
    // - Differential synchronization
}
