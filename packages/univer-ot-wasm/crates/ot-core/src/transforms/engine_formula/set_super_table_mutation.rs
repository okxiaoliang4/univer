use crate::registry::TransformRegistry;

/// Register transforms for SetSuperTableMutation
///
/// Super tables (structured references) provide Excel-like table functionality:
/// - Table name and range
/// - Column headers with structured references
/// - Formulas using [@ColumnName] syntax
///
/// Transform strategies:
/// - Range-based shifting (table boundaries)
/// - Structural reference adjustments
/// - Column add/remove handling
pub fn register_transforms(_registry: &mut TransformRegistry) {
    // TODO: Implement SetSuperTableMutation transforms:
    // - vs InsertRow: Expand table if insert within table boundaries
    // - vs InsertCol: Add column if insert within table
    // - vs RemoveRows/Cols: Shrink table or adjust
    // - vs MoveRange: Complex - may need to move entire table
    // - Self-transform: Conflict by table ID or range overlap
    //
    // CRITICAL: Structured references in formulas need special handling
}
