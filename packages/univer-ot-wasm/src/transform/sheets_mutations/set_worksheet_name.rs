use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::impl_identity_transform;

#[derive(Default)]
pub struct SetWorksheetNameTransform;

impl_identity_transform!(
    SetWorksheetNameTransform,
    "sheet.mutation.set-worksheet-name"
);
