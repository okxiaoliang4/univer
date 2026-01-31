use crate::registry::TransformRegistry;

/// Register transforms for SetArrayFormulaDataMutation
///
/// Array formulas span multiple cells and have special behavior:
/// - Single formula produces array of results
/// - Affects a range of cells
/// - References need adjustment when structure changes
///
/// Transform strategies:
/// - Range shifting with insert/remove operations
/// - Cell reference adjustments (A1:B10 → shifted range)
/// - Conflict resolution when array ranges overlap
pub fn register_transforms(_registry: &mut TransformRegistry) {
    // TODO: Implement SetArrayFormulaDataMutation transforms:
    // - vs InsertRow/Col: Shift array range and adjust references
    // - vs RemoveRows/Cols: Adjust or remove if range affected
    // - vs MoveRows/Cols/Range: Complex reference adjustments
    // - Self-transform: Conflict resolution by range overlap
}
