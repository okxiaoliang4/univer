use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::impl_identity_transform;

#[derive(Default)]
pub struct SetWorkbookNameTransform;

impl_identity_transform!(SetWorkbookNameTransform, "sheet.mutation.set-workbook-name");
