use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::impl_ranges_transform;

#[derive(Default)]
pub struct SetRowHiddenTransform;

impl_ranges_transform!(SetRowHiddenTransform, "sheet.mutation.set-row-hidden");
