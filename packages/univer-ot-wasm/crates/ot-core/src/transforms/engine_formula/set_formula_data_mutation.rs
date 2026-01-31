use crate::registry::TransformRegistry;

/// Register transforms for SetFormulaDataMutation
///
/// Sets formula content in cells:
/// - Formula string (e.g., "=SUM(A1:A10)")
/// - Cell position
/// - Formula type (normal, array, etc.)
///
/// Transform strategies:
/// - Position-based shifting
/// - Reference adjustment (relative references shift, absolute don't)
/// - Conflict resolution at cell level (LWW)
pub fn register_transforms(_registry: &mut TransformRegistry) {
    // TODO: Implement SetFormulaDataMutation transforms:
    // - vs InsertRow/Col: Shift position AND adjust references in formula
    // - vs RemoveRows/Cols: Remove if in range, adjust references
    // - vs MoveRows/Cols/Range: Complex reference adjustments
    // - Self-transform: LWW at cell level
    // - vs SetRangeValues: Conflict (formula vs value)
    //
    // CRITICAL: Reference adjustment is complex:
    // - Relative references (A1) shift
    // - Absolute references ($A$1) don't shift
    // - Mixed references ($A1, A$1) partially shift
}
