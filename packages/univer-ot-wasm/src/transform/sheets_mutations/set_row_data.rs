use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::impl_row_data_transform;

#[derive(Default)]
pub struct SetRowDataTransform;

impl_row_data_transform!(SetRowDataTransform, "sheet.mutation.set-row-data");
