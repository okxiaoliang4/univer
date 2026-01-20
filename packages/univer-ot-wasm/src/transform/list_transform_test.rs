#[cfg(test)]
mod tests {
    use crate::transform::test_utils::test_utils::create_mutation_info;
    use crate::transform::TransformService;
    use crate::mutations::types::{
        InsertColMutationParams, SetRangeValuesMutationParams,
    };
    use crate::types::{
        ObjectMatrixPrimitiveType, Range, SubUnitParams,
    };
    use serde_json;

    fn build_insert_col() -> crate::types::MutationInfoInternal {
        create_mutation_info(
            "sheet.mutation.insert-col".to_string(),
            serde_json::to_value(InsertColMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                range: Range {
                    start_row: 0,
                    start_column: 0,
                    end_row: 0,
                    end_column: 0,
                },
                col_info: None,
            })
            .unwrap(),
        )
    }

    fn build_set_range_values(col: &str) -> crate::types::MutationInfoInternal {
        let mut cell_data = serde_json::Map::new();
        let mut row_data = serde_json::Map::new();
        row_data.insert(col.to_string(), serde_json::json!({"v": 1}));
        cell_data.insert("0".to_string(), serde_json::Value::Object(row_data));

        create_mutation_info(
            "sheet.mutation.set-range-values".to_string(),
            serde_json::to_value(SetRangeValuesMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                cell_value: Some(ObjectMatrixPrimitiveType { data: cell_data }),
            })
            .unwrap(),
        )
    }

    #[test]
    fn test_transform_list_insert_col_then_set_range_values() {
        let service = TransformService::new();
        let m1_list = vec![build_insert_col()];
        let m2_list = vec![build_set_range_values("0")];

        let (m1_primes, m2_primes, error) =
            service.transform_list_internal(&m1_list, &m2_list);
        assert!(error.is_none());
        assert_eq!(m1_primes.len(), 1);
        assert_eq!(m2_primes.len(), 1);

        let transformed_params: SetRangeValuesMutationParams =
            serde_json::from_value(m2_primes[0].params.clone()).unwrap();
        let cell_value = transformed_params.cell_value.unwrap();
        let row = cell_value.data.get("0").unwrap().as_object().unwrap();
        assert!(row.contains_key("1"));
        assert!(!row.contains_key("0"));
    }

    #[test]
    fn test_transform_list_set_range_values_then_insert_col() {
        let service = TransformService::new();
        let m1_list = vec![build_set_range_values("0")];
        let m2_list = vec![build_insert_col()];

        let (m1_primes, m2_primes, error) =
            service.transform_list_internal(&m1_list, &m2_list);
        assert!(error.is_none());
        assert_eq!(m1_primes.len(), 1);
        assert_eq!(m2_primes.len(), 1);

        let transformed_params: SetRangeValuesMutationParams =
            serde_json::from_value(m1_primes[0].params.clone()).unwrap();
        let cell_value = transformed_params.cell_value.unwrap();
        let row = cell_value.data.get("0").unwrap().as_object().unwrap();
        assert!(row.contains_key("1"));
        assert!(!row.contains_key("0"));
    }
}
