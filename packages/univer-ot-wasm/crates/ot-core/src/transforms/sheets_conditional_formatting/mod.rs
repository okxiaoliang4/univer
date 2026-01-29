pub mod conditional_rule;

use crate::registry::TransformRegistry;

/// Register all conditional formatting transforms
pub fn register_transforms(registry: &mut TransformRegistry) {
    conditional_rule::register_transforms(registry);
}
