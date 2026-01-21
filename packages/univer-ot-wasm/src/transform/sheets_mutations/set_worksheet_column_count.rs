use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::impl_identity_transform;

#[derive(Default)]
pub struct SetWorksheetColumnCountTransform;

impl_identity_transform!(
    SetWorksheetColumnCountTransform,
    "sheet.mutation.set-worksheet-column-count"
);
