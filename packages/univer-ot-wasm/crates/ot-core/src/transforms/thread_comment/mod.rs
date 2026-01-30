mod comment;

use crate::registry::TransformRegistry;

pub fn register_transforms(registry: &mut TransformRegistry) {
    comment::register_transforms(registry);
}
