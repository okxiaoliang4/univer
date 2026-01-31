pub mod numfmt;

use crate::registry::TransformRegistry;

/// Register all number format transforms
///
/// This module handles transforms for number format mutations:
/// - SetNumfmtMutation: Sets number format for cell ranges
/// - RemoveNumfmtMutation: Removes number format from ranges
///
/// See: .claude/skills/univer-ot-dev/references/mutations/numfmt_mutations.md
pub fn register_transforms(registry: &mut TransformRegistry) {
    numfmt::register_transforms(registry);
}
