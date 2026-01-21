use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::impl_ranges_transform;

#[derive(Default)]
pub struct SetRowVisibleTransform;

impl_ranges_transform!(SetRowVisibleTransform, "sheet.mutation.set-row-visible");
