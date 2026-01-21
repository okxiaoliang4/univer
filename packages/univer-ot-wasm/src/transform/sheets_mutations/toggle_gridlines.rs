use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::impl_identity_transform;

#[derive(Default)]
pub struct ToggleGridlinesTransform;

impl_identity_transform!(ToggleGridlinesTransform, "sheet.mutation.toggle-gridlines");
