use crate::mutations::types::InsertColMutationParams;
use crate::types::TransformError;

pub struct InsertColMutation;

impl InsertColMutation {
    pub fn apply(params: &InsertColMutationParams) -> Result<bool, TransformError> {
        // Validate range
        if params.range.start_column > params.range.end_column {
            return Err(TransformError {
                message: "Invalid range: start_column > end_column".to_string(),
                code: 3001,
            });
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mutations::types::InsertColMutationParams;
    use crate::types::{Range, SubUnitParams};

    #[test]
    fn test_insert_col_mutation_valid_params() {
        let params = InsertColMutationParams {
            sub_unit_params: SubUnitParams {
                unit_id: "test-unit".to_string(),
                sub_unit_id: "test-sheet".to_string(),
            },
            range: Range {
                start_row: 0,
                start_column: 1,
                end_row: 0,
                end_column: 2,
            },
            col_info: None,
        };

        let result = InsertColMutation::apply(&params);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}
