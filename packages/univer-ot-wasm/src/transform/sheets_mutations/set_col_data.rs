use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::impl_col_data_transform;

#[derive(Default)]
pub struct SetColDataTransform;

impl_col_data_transform!(SetColDataTransform, "sheet.mutation.set-col-data");
