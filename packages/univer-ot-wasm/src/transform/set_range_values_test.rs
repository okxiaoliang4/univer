#[cfg(test)]
mod tests {
    use crate::types::*;
    use crate::transform::set_range_values::SetRangeValuesTransform;
    use crate::transform::mutation_transform::MutationTransform;
    use crate::transform::test_utils::test_utils::create_mutation_info;
    use serde_json;
    use crate::transform::TransformService;

    #[test]
    fn test_set_range_values_transform_trait() {
        let transform = SetRangeValuesTransform::default();
        assert_eq!(SetRangeValuesTransform::mutation_id(), "sheet.mutation.set-range-values");
    }

    // SetRangeValues × SetRangeValues (identity transform)
    #[test]
    fn test_transform_set_range_values_with_set_range_values() {
        let service = TransformService::new();

        let mut cell_data1 = serde_json::Map::new();
        let mut row_data1 = serde_json::Map::new();
        row_data1.insert("0".to_string(), serde_json::json!({"v": "value1"}));
        cell_data1.insert("0".to_string(), serde_json::Value::Object(row_data1));

        let mut cell_data2 = serde_json::Map::new();
        let mut row_data2 = serde_json::Map::new();
        row_data2.insert("1".to_string(), serde_json::json!({"v": "value2"}));
        cell_data2.insert("1".to_string(), serde_json::Value::Object(row_data2));

        let m1 = create_mutation_info(
            "sheet.mutation.set-range-values".to_string(),
            serde_json::to_value(SetRangeValuesMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                cell_value: Some(ObjectMatrixPrimitiveType { data: cell_data1 }),
            }).unwrap(),
        );

        let m2 = create_mutation_info(
            "sheet.mutation.set-range-values".to_string(),
            serde_json::to_value(SetRangeValuesMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                cell_value: Some(ObjectMatrixPrimitiveType { data: cell_data2 }),
            }).unwrap(),
        );

        let result = service.transform_internal(&m1, &m2);
        assert!(result.error.is_none());
        // Identity transform - should remain unchanged
        assert_eq!(result.m1_prime.id, m1.id);
        assert_eq!(result.m2_prime.id, m2.id);
    }

    // SetRangeValues × InsertRow
    #[test]
    fn test_transform_set_range_values_insert_row() {
        let service = TransformService::new();

        let mut cell_data = serde_json::Map::new();
        let mut row_data = serde_json::Map::new();
        row_data.insert("2".to_string(), serde_json::json!({"v": "test"}));
        cell_data.insert("2".to_string(), serde_json::Value::Object(row_data));

        let m1 = create_mutation_info(
            "sheet.mutation.set-range-values".to_string(),
            serde_json::to_value(SetRangeValuesMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                cell_value: Some(ObjectMatrixPrimitiveType { data: cell_data }),
            }).unwrap(),
        );

        let m2 = create_mutation_info(
            "sheet.mutation.insert-row".to_string(),
            serde_json::to_value(InsertRowMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                range: Range {
                    start_row: 1,
                    start_column: 0,
                    end_row: 1,
                    end_column: 0,
                },
                row_info: None,
            }).unwrap(),
        );

        let result = service.transform_internal(&m1, &m2);
        assert!(result.error.is_none());

        let transformed_params: SetRangeValuesMutationParams = serde_json::from_value(result.m1_prime.params.clone()).unwrap();
        assert!(transformed_params.cell_value.is_some());

        let cell_value = transformed_params.cell_value.unwrap();
        assert!(cell_value.data.contains_key("3"));
        assert!(!cell_value.data.contains_key("2"));
    }

    // SetRangeValues × InsertCol
    #[test]
    fn test_transform_set_range_values_insert_col() {
        let service = TransformService::new();

        let mut cell_data = serde_json::Map::new();
        let mut row_data = serde_json::Map::new();
        row_data.insert("1".to_string(), serde_json::json!({"v": "test"}));
        cell_data.insert("0".to_string(), serde_json::Value::Object(row_data));

        let m1 = create_mutation_info(
            "sheet.mutation.set-range-values".to_string(),
            serde_json::to_value(SetRangeValuesMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                cell_value: Some(ObjectMatrixPrimitiveType { data: cell_data }),
            }).unwrap(),
        );

        let m2 = create_mutation_info(
            "sheet.mutation.insert-col".to_string(),
            serde_json::to_value(InsertColMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                range: Range {
                    start_row: 0,
                    start_column: 1,
                    end_row: 0,
                    end_column: 1,
                },
                col_info: None,
            }).unwrap(),
        );

        let result = service.transform_internal(&m1, &m2);
        assert!(result.error.is_none());
        // Currently returns identity transform (column transform not yet implemented)
        assert_eq!(result.m1_prime.id, m1.id);
    }

    // SetRangeValues × RemoveRows
    #[test]
    fn test_transform_set_range_values_remove_rows() {
        let service = TransformService::new();

        let mut cell_data = serde_json::Map::new();
        let mut row_data = serde_json::Map::new();
        row_data.insert("5".to_string(), serde_json::json!({"v": "test"}));
        cell_data.insert("5".to_string(), serde_json::Value::Object(row_data));

        let m1 = create_mutation_info(
            "sheet.mutation.set-range-values".to_string(),
            serde_json::to_value(SetRangeValuesMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                cell_value: Some(ObjectMatrixPrimitiveType { data: cell_data }),
            }).unwrap(),
        );

        let m2 = create_mutation_info(
            "sheet.mutation.remove-rows".to_string(),
            serde_json::to_value(RemoveRowsMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                range: Range {
                    start_row: 1,
                    start_column: 0,
                    end_row: 3,
                    end_column: 0,
                },
            }).unwrap(),
        );

        let result = service.transform_internal(&m1, &m2);
        assert!(result.error.is_none());

        let transformed_params: SetRangeValuesMutationParams = serde_json::from_value(result.m1_prime.params.clone()).unwrap();
        assert!(transformed_params.cell_value.is_some());

        let cell_value = transformed_params.cell_value.unwrap();
        // Row 5 should become row 2 (5 - 3 = 2)
        assert!(cell_value.data.contains_key("2"));
        assert!(!cell_value.data.contains_key("5"));
    }

    // SetRangeValues × RemoveCol
    #[test]
    fn test_transform_set_range_values_remove_col() {
        let service = TransformService::new();

        let mut cell_data = serde_json::Map::new();
        let mut row_data = serde_json::Map::new();
        row_data.insert("2".to_string(), serde_json::json!({"v": "test"}));
        cell_data.insert("0".to_string(), serde_json::Value::Object(row_data));

        let m1 = create_mutation_info(
            "sheet.mutation.set-range-values".to_string(),
            serde_json::to_value(SetRangeValuesMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                cell_value: Some(ObjectMatrixPrimitiveType { data: cell_data }),
            }).unwrap(),
        );

        let m2 = create_mutation_info(
            "sheet.mutation.remove-col".to_string(),
            serde_json::to_value(RemoveColMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                range: Range {
                    start_row: 0,
                    start_column: 1,
                    end_row: 0,
                    end_column: 3,
                },
            }).unwrap(),
        );

        let result = service.transform_internal(&m1, &m2);
        assert!(result.error.is_none());
        // Currently returns identity transform (column transform not yet implemented)
        assert_eq!(result.m1_prime.id, m1.id);
    }
}
