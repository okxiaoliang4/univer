use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::impl_identity_transform;

#[derive(Default)]
pub struct SetWorksheetOrderTransform;

impl_identity_transform!(
    SetWorksheetOrderTransform,
    "sheet.mutation.set-worksheet-order"
);
