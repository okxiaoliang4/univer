use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::impl_identity_transform;

#[derive(Default)]
pub struct SetWorksheetRowCountTransform;

impl_identity_transform!(
    SetWorksheetRowCountTransform,
    "sheet.mutation.set-worksheet-row-count"
);
