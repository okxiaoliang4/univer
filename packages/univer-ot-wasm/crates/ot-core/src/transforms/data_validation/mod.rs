pub mod validation;

use crate::registry::TransformRegistry;

/// Register all data validation transforms
pub fn register_transforms(registry: &mut TransformRegistry) {
    validation::register_transforms(registry);
}
