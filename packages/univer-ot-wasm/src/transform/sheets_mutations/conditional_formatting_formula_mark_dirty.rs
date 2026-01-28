use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::impl_identity_transform;

#[derive(Default)]
pub struct ConditionalFormattingFormulaMarkDirtyTransform;

impl_identity_transform!(
    ConditionalFormattingFormulaMarkDirtyTransform,
    "sheet.mutation.conditional-formatting-formula-mark-dirty"
);
