use crate::registry::TransformRegistry;

/// Register transforms for SetImageFormulaDataMutation
///
/// Handles formulas that return images (e.g., IMAGE() function):
/// - Image URL/data from formula
/// - Cell position
/// - Display properties
///
/// Transform strategies:
/// - Similar to SetFormulaDataMutation (position + reference adjustment)
/// - Additional consideration for image display area
pub fn register_transforms(_registry: &mut TransformRegistry) {
    // TODO: Implement SetImageFormulaDataMutation transforms:
    // - Similar patterns to SetFormulaDataMutation
    // - Additional transforms for image display properties
}
