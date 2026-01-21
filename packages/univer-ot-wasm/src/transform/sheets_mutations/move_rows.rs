use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::impl_move_rows_cols_transform;

#[derive(Default)]
pub struct MoveRowsTransform;

impl_move_rows_cols_transform!(MoveRowsTransform, "sheet.mutation.move-rows", true);
