use crate::mutations::sheets::UnregisterWorksheetRangeThemeStyleMutation;
use crate::registry::{MutationId, TransformRegistry};

pub const MUTATION_ID: MutationId = UnregisterWorksheetRangeThemeStyleMutation::ID;

pub fn register_transforms(_registry: &mut TransformRegistry) {


    // NOTE: No register_identity calls needed - registry falls back to identity automatically
}
