use crate::registry::TransformRegistry;

/// Register transforms for SetFormulaCalculationMutation
///
/// Handles formula calculation state and results:
/// - Calculated values cache
/// - Dirty flags for recalculation
/// - Calculation order/dependencies
///
/// Transform strategies:
/// - Position-based (cell coordinates)
/// - Dependency chain awareness
/// - Invalidation on structural changes
pub fn register_transforms(_registry: &mut TransformRegistry) {
    // TODO: Implement SetFormulaCalculationMutation transforms:
    // - vs InsertRow/Col: Shift cell positions, invalidate dependent formulas
    // - vs RemoveRows/Cols: Remove/adjust, propagate invalidation
    // - vs SetRangeValues: May need recalculation trigger
    // - Self-transform: LWW at cell level
}
