use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::impl_identity_transform;

#[derive(Default)]
pub struct SetWorksheetPermissionPointsTransform;

impl_identity_transform!(
    SetWorksheetPermissionPointsTransform,
    "sheet.mutation.set-worksheet-permission-points"
);
