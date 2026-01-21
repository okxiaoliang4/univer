use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::impl_ranges_transform;

#[derive(Default)]
pub struct SetColVisibleTransform;

impl_ranges_transform!(SetColVisibleTransform, "sheet.mutation.set-col-visible");
