use crate::types::{SetRangeValuesMutationParams, TransformError};

pub struct SetRangeValuesMutation;

impl SetRangeValuesMutation {
    pub fn apply(params: &SetRangeValuesMutationParams) -> Result<bool, TransformError> {
        // Validate row keys are numeric
        if let Some(ref cell_value) = params.cell_value {
            for row_key in cell_value.data.keys() {
                if row_key.parse::<u32>().is_err() {
                    return Err(TransformError {
                        message: format!("Invalid row key: {}", row_key),
                        code: 1001,
                    });
                }
            }
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{SetRangeValuesMutationParams, SubUnitParams, ObjectMatrixPrimitiveType};
    use serde_json;

    #[test]
    fn test_set_range_values_mutation_valid_params() {
        let params = SetRangeValuesMutationParams {
            sub_unit_params: SubUnitParams {
                unit_id: "test-unit".to_string(),
                sub_unit_id: "test-sheet".to_string(),
            },
            cell_value: None,
        };

        let result = SetRangeValuesMutation::apply(&params);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_set_range_values_mutation_with_cell_data() {
        let mut cell_data = serde_json::Map::new();
        let mut row_data = serde_json::Map::new();
        row_data.insert("0".to_string(), serde_json::json!({"v": "test"}));
        cell_data.insert("0".to_string(), serde_json::Value::Object(row_data));

        let params = SetRangeValuesMutationParams {
            sub_unit_params: SubUnitParams {
                unit_id: "test-unit".to_string(),
                sub_unit_id: "test-sheet".to_string(),
            },
            cell_value: Some(ObjectMatrixPrimitiveType { data: cell_data }),
        };

        let result = SetRangeValuesMutation::apply(&params);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_set_range_values_mutation_invalid_row_key() {
        let mut cell_data = serde_json::Map::new();
        let mut row_data = serde_json::Map::new();
        row_data.insert("0".to_string(), serde_json::json!({"v": "test"}));
        cell_data.insert("invalid".to_string(), serde_json::Value::Object(row_data));

        let params = SetRangeValuesMutationParams {
            sub_unit_params: SubUnitParams {
                unit_id: "test-unit".to_string(),
                sub_unit_id: "test-sheet".to_string(),
            },
            cell_value: Some(ObjectMatrixPrimitiveType { data: cell_data }),
        };

        let result = SetRangeValuesMutation::apply(&params);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, 1001);
    }
}
