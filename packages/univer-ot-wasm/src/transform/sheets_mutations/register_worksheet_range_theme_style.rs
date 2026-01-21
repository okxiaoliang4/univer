use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::impl_identity_transform;

#[derive(Default)]
pub struct RegisterWorksheetRangeThemeStyleTransform;

impl_identity_transform!(
    RegisterWorksheetRangeThemeStyleTransform,
    "sheet.mutation.register-worksheet-range-theme-style"
);
