use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::impl_identity_transform;

#[derive(Default)]
pub struct SetWorksheetProtectionTransform;

impl_identity_transform!(
    SetWorksheetProtectionTransform,
    "sheet.mutation.set-worksheet-protection"
);
