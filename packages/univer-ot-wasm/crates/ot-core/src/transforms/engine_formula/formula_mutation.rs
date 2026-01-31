use crate::registry::TransformRegistry;

/// Register transforms for FormulaMutation
///
/// This is a base formula mutation handler.
///
/// Note: Formula OT requires special handling due to:
/// - Cell reference adjustments (relative vs absolute)
/// - Dependency chain recalculation
/// - Circular reference detection
/// - Named range references
pub fn register_transforms(_registry: &mut TransformRegistry) {
    // TODO: Implement FormulaMutation transforms
}
