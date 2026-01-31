pub mod hyper_link_mutation;

use crate::registry::TransformRegistry;

/// Register all docs hyperlink transforms
///
/// This module handles transforms for document hyperlink mutations.
///
/// Note: Document hyperlinks are different from sheets hyperlinks.
/// These mutations are currently out of scope for the main OT implementation.
pub fn register_transforms(registry: &mut TransformRegistry) {
    hyper_link_mutation::register_transforms(registry);
}
