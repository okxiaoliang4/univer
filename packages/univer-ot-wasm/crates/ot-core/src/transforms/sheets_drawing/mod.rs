pub mod set_drawing_apply;

use crate::registry::TransformRegistry;

/// Register all sheets drawing transforms
///
/// This module handles transforms for drawing-related mutations:
/// - SetDrawingApplyMutation: Complex drawing operations
///
/// Note: Drawing operations involve complex visual elements and are
/// currently out of scope for the main OT implementation.
pub fn register_transforms(registry: &mut TransformRegistry) {
    set_drawing_apply::register_transforms(registry);
}
