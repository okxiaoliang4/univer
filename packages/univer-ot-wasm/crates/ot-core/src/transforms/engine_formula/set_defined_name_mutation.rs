use crate::registry::TransformRegistry;

/// Register transforms for SetDefinedNameMutation
///
/// Defined names (named ranges) map a name to a cell reference or formula:
/// - Example: "SalesData" → A1:B100
/// - Used in formulas instead of direct references
///
/// Transform strategies:
/// - Name-based identification (LWW by name)
/// - Range adjustments when referred cells are affected
/// - Workbook or worksheet scope
pub fn register_transforms(_registry: &mut TransformRegistry) {
    // TODO: Implement SetDefinedNameMutation transforms:
    // - Self-transform: LWW by name
    // - vs InsertRow/Col: Adjust referenced ranges
    // - vs RemoveRows/Cols: Adjust or invalidate if referenced cells removed
    // - vs other mutations: Identity (name-based, independent)
}
