use crate::registry::TransformRegistry;

/// Register transforms for SetDrawingApplyMutation
///
/// Mutation ID: sheet.mutation.set-drawing-apply (or similar)
///
/// Drawing elements (shapes, images, charts) have complex properties:
/// - Position (row, col offsets)
/// - Size (width, height)
/// - Z-order
/// - Visual properties
///
/// Transform strategies likely needed:
/// - Position adjustments when rows/columns are inserted/removed
/// - Boundary clipping when anchor cells are removed
/// - Z-order conflict resolution
///
/// This is currently out of scope for the main OT implementation.
pub fn register_transforms(_registry: &mut TransformRegistry) {
    // TODO: Implement SetDrawingApplyMutation transforms:
    // - vs InsertRow/Col: Adjust position if anchored to affected cells
    // - vs RemoveRows/Cols: Remove or adjust if anchor cells removed
    // - vs MoveRows/Cols/Range: Adjust if anchor cells moved
    // - Self-transform: LWW or conflict based on drawing ID
    //
    // Note: Drawing positions are typically anchored to cell coordinates
    // with pixel offsets, requiring careful coordinate transformation
}
