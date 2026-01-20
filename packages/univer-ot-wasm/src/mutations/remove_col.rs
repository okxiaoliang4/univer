use crate::mutations::types::RemoveColMutationParams;
use crate::types::TransformError;

pub struct RemoveColMutation;

impl RemoveColMutation {
    pub fn apply(params: &RemoveColMutationParams) -> Result<bool, TransformError> {
        // Validate range
        if params.range.start_column > params.range.end_column {
            return Err(TransformError {
                message: "Invalid range: start_column > end_column".to_string(),
                code: 5001,
            });
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mutations::types::RemoveColMutationParams;
    use crate::types::{Range, SubUnitParams};

    #[test]
    fn test_remove_col_mutation_valid_params() {
        let params = RemoveColMutationParams {
            sub_unit_params: SubUnitParams {
                unit_id: "test-unit".to_string(),
                sub_unit_id: "test-sheet".to_string(),
            },
            range: Range {
                start_row: 0,
                start_column: 2,
                end_row: 0,
                end_column: 4,
            },
        };

        let result = RemoveColMutation::apply(&params);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}
