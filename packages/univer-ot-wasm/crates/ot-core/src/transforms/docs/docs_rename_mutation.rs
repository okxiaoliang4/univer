use crate::registry::TransformRegistry;

/// Register transforms for DocsRenameMutation
///
/// Mutation ID: docs.mutation.rename (or similar)
///
/// This mutation handles document rename operations.
/// Since it's a document-level property (not position-based),
/// transforms are likely simpler than rich text editing.
///
/// Expected pattern: Last-Write-Wins (LWW) at document level
pub fn register_transforms(_registry: &mut TransformRegistry) {
    // TODO: Implement DocsRenameMutation transforms:
    // - Self-transform: LWW (m2 wins)
    // - vs other mutations: Likely identity (independent operations)
}
