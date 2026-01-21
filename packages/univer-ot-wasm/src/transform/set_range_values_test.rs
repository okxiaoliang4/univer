#[cfg(test)]
mod tests {
    use crate::mutations::types::{
        InsertColMutationParams, InsertRowMutationParams, RemoveColMutationParams,
        RemoveRowsMutationParams, SetRangeValuesMutationParams,
    };
    use crate::transform::mutation_transform::MutationTransform;
    use crate::transform::set_range_values::SetRangeValuesTransform;
    use crate::transform::test_utils::test_utils::create_mutation_info;
    use crate::transform::TransformService;
    use crate::types::{ObjectMatrixPrimitiveType, Range, SubUnitParams};
    use serde_json;

    #[test]
    fn test_set_range_values_transform_trait() {
        let _transform = SetRangeValuesTransform::default();
        assert_eq!(
            SetRangeValuesTransform::mutation_id(),
            "sheet.mutation.set-range-values"
        );
    }

    // SetRangeValues × SetRangeValues (LWW - Last Writer Wins)
    // Test conflict: both modify the same cell
    #[test]
    fn test_transform_set_range_values_with_set_range_values_conflict() {
        let service = TransformService::new();

        // Both operations modify cell (0, 0)
        let mut cell_data1 = serde_json::Map::new();
        let mut row_data1 = serde_json::Map::new();
        row_data1.insert("0".to_string(), serde_json::json!({"v": "value1"}));
        cell_data1.insert("0".to_string(), serde_json::Value::Object(row_data1));

        let mut cell_data2 = serde_json::Map::new();
        let mut row_data2 = serde_json::Map::new();
        row_data2.insert("0".to_string(), serde_json::json!({"v": "value2"}));
        cell_data2.insert("0".to_string(), serde_json::Value::Object(row_data2));

        let m1 = create_mutation_info(
            "sheet.mutation.set-range-values".to_string(),
            serde_json::to_value(SetRangeValuesMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                cell_value: Some(ObjectMatrixPrimitiveType { data: cell_data1 }),
            })
            .unwrap(),
        );

        let m2 = create_mutation_info(
            "sheet.mutation.set-range-values".to_string(),
            serde_json::to_value(SetRangeValuesMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                cell_value: Some(ObjectMatrixPrimitiveType { data: cell_data2 }),
            })
            .unwrap(),
        );

        let result = service.transform_internal(&m1, &m2);
        assert!(result.error.is_none());

        // LWW: m1 is later and should win on conflicts
        // m1_prime should keep its original value
        let m1_prime_params: SetRangeValuesMutationParams =
            serde_json::from_value(result.m1_prime.unwrap().params.clone()).unwrap();
        assert!(m1_prime_params.cell_value.is_some());
        let m1_cell_value = m1_prime_params.cell_value.unwrap();

        // Cell (0, 0) should keep m1's value
        let row = m1_cell_value.data.get("0").expect("Row 0 should exist");
        let row_obj = row.as_object().expect("Row should be an object");
        let cell_value = row_obj.get("0").expect("Cell (0,0) should exist");
        assert_eq!(
            cell_value.get("v").unwrap(),
            "value1",
            "Conflicting cell should keep m1 value"
        );

        // m2_prime should be dropped because it conflicts with m1
        assert!(result.m2_prime.is_none());
    }

    // SetRangeValues × SetRangeValues (no conflict - different cells)
    #[test]
    fn test_transform_set_range_values_with_set_range_values_no_conflict() {
        let service = TransformService::new();

        // m1 modifies cell (0, 0)
        let mut cell_data1 = serde_json::Map::new();
        let mut row_data1 = serde_json::Map::new();
        row_data1.insert("0".to_string(), serde_json::json!({"v": "value1"}));
        cell_data1.insert("0".to_string(), serde_json::Value::Object(row_data1));

        // m2 modifies cell (1, 1)
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
            })
            .unwrap(),
        );

        let m2 = create_mutation_info(
            "sheet.mutation.set-range-values".to_string(),
            serde_json::to_value(SetRangeValuesMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                cell_value: Some(ObjectMatrixPrimitiveType { data: cell_data2 }),
            })
            .unwrap(),
        );

        let result = service.transform_internal(&m1, &m2);
        assert!(result.error.is_none());

        // No conflict: both operations should remain unchanged
        assert_eq!(result.m1_prime.unwrap().params, m1.params);
        assert_eq!(result.m2_prime.unwrap().params, m2.params);
    }

    // SetRangeValues × SetRangeValues (partial conflict)
    #[test]
    fn test_transform_set_range_values_with_set_range_values_partial_conflict() {
        let service = TransformService::new();

        // m1 modifies cells (0, 0) and (0, 1)
        let mut cell_data1 = serde_json::Map::new();
        let mut row_data1 = serde_json::Map::new();
        row_data1.insert("0".to_string(), serde_json::json!({"v": "value1"}));
        row_data1.insert("1".to_string(), serde_json::json!({"v": "value2"}));
        cell_data1.insert("0".to_string(), serde_json::Value::Object(row_data1));

        // m2 modifies cell (0, 0)
        let mut cell_data2 = serde_json::Map::new();
        let mut row_data2 = serde_json::Map::new();
        row_data2.insert("0".to_string(), serde_json::json!({"v": "value3"}));
        cell_data2.insert("0".to_string(), serde_json::Value::Object(row_data2));

        let m1 = create_mutation_info(
            "sheet.mutation.set-range-values".to_string(),
            serde_json::to_value(SetRangeValuesMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                cell_value: Some(ObjectMatrixPrimitiveType { data: cell_data1 }),
            })
            .unwrap(),
        );

        let m2 = create_mutation_info(
            "sheet.mutation.set-range-values".to_string(),
            serde_json::to_value(SetRangeValuesMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                cell_value: Some(ObjectMatrixPrimitiveType { data: cell_data2 }),
            })
            .unwrap(),
        );

        let result = service.transform_internal(&m1, &m2);
        assert!(result.error.is_none());

        // Partial conflict: m1_prime should keep both cells intact
        let m1_prime_params: SetRangeValuesMutationParams =
            serde_json::from_value(result.m1_prime.unwrap().params.clone()).unwrap();
        assert!(m1_prime_params.cell_value.is_some());
        let m1_cell_value = m1_prime_params.cell_value.unwrap();
        let row = m1_cell_value.data.get("0").unwrap().as_object().unwrap();

        // Cell (0, 0) should keep m1's value due to LWW
        assert!(row.contains_key("0"));
        assert_eq!(row.get("0").unwrap().get("v").unwrap(), "value1");

        // Cell (0, 1) should remain unchanged (no conflict)
        assert!(row.contains_key("1"));
        assert_eq!(row.get("1").unwrap().get("v").unwrap(), "value2");

        // m2_prime should be dropped because it conflicts with m1
        assert!(result.m2_prime.is_none());
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
            })
            .unwrap(),
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
            })
            .unwrap(),
        );

        let result = service.transform_internal(&m1, &m2);
        assert!(result.error.is_none());

        let transformed_params: SetRangeValuesMutationParams =
            serde_json::from_value(result.m1_prime.unwrap().params.clone()).unwrap();
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
            })
            .unwrap(),
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
            })
            .unwrap(),
        );

        let result = service.transform_internal(&m1, &m2);
        assert!(result.error.is_none());

        let transformed_params: SetRangeValuesMutationParams =
            serde_json::from_value(result.m1_prime.unwrap().params.clone()).unwrap();
        assert!(transformed_params.cell_value.is_some());

        let cell_value = transformed_params.cell_value.unwrap();
        let row = cell_value.data.get("0").unwrap().as_object().unwrap();
        assert!(row.contains_key("2"));
        assert!(!row.contains_key("1"));
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
            })
            .unwrap(),
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
            })
            .unwrap(),
        );

        let result = service.transform_internal(&m1, &m2);
        assert!(result.error.is_none());

        let transformed_params: SetRangeValuesMutationParams =
            serde_json::from_value(result.m1_prime.unwrap().params.clone()).unwrap();
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
            })
            .unwrap(),
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
                    start_column: 0,
                    end_row: 0,
                    end_column: 0,
                },
            })
            .unwrap(),
        );

        let result = service.transform_internal(&m1, &m2);
        assert!(result.error.is_none());

        let transformed_params: SetRangeValuesMutationParams =
            serde_json::from_value(result.m1_prime.unwrap().params.clone()).unwrap();
        assert!(transformed_params.cell_value.is_some());

        let cell_value = transformed_params.cell_value.unwrap();
        assert!(cell_value.data.contains_key("0"));
    }

    #[test]
    fn test_compose_set_range_values_merge_cells() {
        let service = TransformService::new();

        let mut cell_data1 = serde_json::Map::new();
        let mut row_data1 = serde_json::Map::new();
        row_data1.insert("0".to_string(), serde_json::json!({"v": "a"}));
        cell_data1.insert("0".to_string(), serde_json::Value::Object(row_data1));

        let mut cell_data2 = serde_json::Map::new();
        let mut row_data2 = serde_json::Map::new();
        row_data2.insert("1".to_string(), serde_json::json!({"v": "b"}));
        cell_data2.insert("0".to_string(), serde_json::Value::Object(row_data2));

        let m1 = create_mutation_info(
            "sheet.mutation.set-range-values".to_string(),
            serde_json::to_value(SetRangeValuesMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                cell_value: Some(ObjectMatrixPrimitiveType { data: cell_data1 }),
            })
            .unwrap(),
        );

        let m2 = create_mutation_info(
            "sheet.mutation.set-range-values".to_string(),
            serde_json::to_value(SetRangeValuesMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                cell_value: Some(ObjectMatrixPrimitiveType { data: cell_data2 }),
            })
            .unwrap(),
        );

        let result = service.compose_internal(&m1, &m2);
        assert_eq!(result.len(), 1);
        let composed_params: SetRangeValuesMutationParams =
            serde_json::from_value(result[0].params.clone()).unwrap();
        let composed_value = composed_params.cell_value.unwrap();
        let row = composed_value.data.get("0").unwrap();
        let row_map = row.as_object().unwrap();
        assert!(row_map.contains_key("0"));
        assert!(row_map.contains_key("1"));
    }

    #[test]
    fn test_compose_set_range_values_different_sheet() {
        let service = TransformService::new();

        let m1 = create_mutation_info(
            "sheet.mutation.set-range-values".to_string(),
            serde_json::to_value(SetRangeValuesMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "sheet-1".to_string(),
                },
                cell_value: None,
            })
            .unwrap(),
        );

        let m2 = create_mutation_info(
            "sheet.mutation.set-range-values".to_string(),
            serde_json::to_value(SetRangeValuesMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "sheet-2".to_string(),
                },
                cell_value: None,
            })
            .unwrap(),
        );

        let result = service.compose_internal(&m1, &m2);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_compose_list_set_range_values() {
        let service = TransformService::new();

        let m1 = create_mutation_info(
            "sheet.mutation.set-range-values".to_string(),
            serde_json::to_value(SetRangeValuesMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                cell_value: None,
            })
            .unwrap(),
        );

        let m2 = create_mutation_info(
            "sheet.mutation.set-range-values".to_string(),
            serde_json::to_value(SetRangeValuesMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                cell_value: None,
            })
            .unwrap(),
        );

        let m3 = create_mutation_info(
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
            })
            .unwrap(),
        );

        let list = vec![m1.clone(), m2.clone(), m3.clone()];
        let result = service.compose_list_internal(&list);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, "sheet.mutation.set-range-values");
        assert_eq!(result[1].id, "sheet.mutation.insert-row");
    }

    // Test OT consistency: verify that client and server reach the same final state
    // Scenario: User A and User B both modify the same cell concurrently
    #[test]
    fn test_set_range_values_ot_consistency() {
        let service = TransformService::new();

        // Initial state: Cell A1 = "X"
        // User A: SetRangeValues(A1, "Hello")
        // User B: SetRangeValues(A1, "World")

        let mut cell_data_a = serde_json::Map::new();
        let mut row_data_a = serde_json::Map::new();
        row_data_a.insert("0".to_string(), serde_json::json!({"v": "Hello"}));
        cell_data_a.insert("0".to_string(), serde_json::Value::Object(row_data_a));

        let mut cell_data_b = serde_json::Map::new();
        let mut row_data_b = serde_json::Map::new();
        row_data_b.insert("0".to_string(), serde_json::json!({"v": "World"}));
        cell_data_b.insert("0".to_string(), serde_json::Value::Object(row_data_b));

        let m_a = create_mutation_info(
            "sheet.mutation.set-range-values".to_string(),
            serde_json::to_value(SetRangeValuesMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                cell_value: Some(ObjectMatrixPrimitiveType { data: cell_data_a }),
            })
            .unwrap(),
        );

        let m_b = create_mutation_info(
            "sheet.mutation.set-range-values".to_string(),
            serde_json::to_value(SetRangeValuesMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                cell_value: Some(ObjectMatrixPrimitiveType { data: cell_data_b }),
            })
            .unwrap(),
        );

        // Server receives m_A first
        // Server state: $State + m_A
        // Server broadcasts m_A to others

        // Server then receives m_B with base_rev before m_A
        // Server performs: transform(m_B, m_A) → (m_B', m_A')
        let result = service.transform_internal(&m_b, &m_a);
        assert!(result.error.is_none());

        let m_b_prime = result.m1_prime;
        let m_a_prime = result.m2_prime;

        // Server applies m_B' and stores it
        // Server state: $State + m_A + m_B'
        // Server broadcasts m_B' to others

        // Verify OT property: m_B' should keep its value as the later writer
        let m_b_prime_params: SetRangeValuesMutationParams =
            serde_json::from_value(m_b_prime.unwrap().params.clone()).unwrap();

        if let Some(cell_value) = m_b_prime_params.cell_value {
            let row = cell_value.data.get("0").expect("Row 0 should exist");
            let row_obj = row.as_object().expect("Row should be an object");
            let cell = row_obj.get("0").expect("Cell (0,0) should exist");
            assert_eq!(
                cell.get("v").unwrap(),
                "World",
                "m_B' should keep conflicting cell value"
            );
        }

        // Client A perspective:
        // 1. Sends m_A, applies locally → A1 = "Hello"
        // 2. Receives Ack(rev 11)
        // 3. Receives broadcast(rev 12, m_B') where m_B' keeps A1 = "World"
        // 4. Applies m_B' directly (no pending) → A1 = "World"
        // Final: $State + m_A + m_B' = $State + m_B (m_B's "World" wins)

        // Client B perspective:
        // 1. Sends m_B, applies locally → A1 = "World", pending = [m_B]
        // 2. Receives broadcast(rev 11, m_A) where m_A sets A1 = "Hello"
        // 3. Performs transform([m_B], [m_A]) → ([m_B'], [m_A'])
        //    - m_B' keeps A1 = "World" (later writer wins)
        //    - m_A' = m_A (no change needed)
        // 4. Applies m_A' to UI → A1 = "Hello"
        // 5. Updates pending = [m_B'] (A1 = "World")
        // 6. Receives Ack(rev 12), clears pending
        // Final: $State + m_B + m_A' = $State + m_B (A1 = "World" wins)

        // Verify m_A' should be dropped because it conflicts with m_B
        assert!(m_a_prime.is_none());

        // Both clients end up with: $State + m_B (Cell A1 = "World")
        // The LWW strategy ensures the later writer (m1) overwrites earlier values
    }
}
