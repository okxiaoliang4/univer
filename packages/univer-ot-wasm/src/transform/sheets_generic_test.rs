#[cfg(test)]
mod tests {
    use crate::transform::sheets_generic::{build_generic_transform, SheetTransformKind};
    use crate::transform::test_utils::test_utils::create_mutation_info;
    use serde_json;

    #[test]
    fn test_range_transform_with_insert_row() {
        let transform = build_generic_transform("sheet.mutation.add-range-theme", SheetTransformKind::Range);
        let range_params = serde_json::json!({
            "unitId": "unit",
            "subUnitId": "sheet",
            "range": { "startRow": 1, "startColumn": 0, "endRow": 2, "endColumn": 1 }
        });
        let insert_params = serde_json::json!({
            "unitId": "unit",
            "subUnitId": "sheet",
            "range": { "startRow": 0, "startColumn": 0, "endRow": 0, "endColumn": 0 }
        });
        let m1 = create_mutation_info(transform.id.to_string(), range_params);
        let m2 = create_mutation_info("sheet.mutation.insert-row".to_string(), insert_params);
        let result = transform.transform_with_insert_row(&m1, &m2);
        let updated: serde_json::Value = result.m1_prime.unwrap().params;
        let range = updated.get("range").unwrap();
        assert_eq!(range.get("startRow").unwrap().as_u64().unwrap(), 2);
    }

    #[test]
    fn test_range_transform_with_remove_row_drops() {
        let transform = build_generic_transform("sheet.mutation.add-range-theme", SheetTransformKind::Range);
        let range_params = serde_json::json!({
            "unitId": "unit",
            "subUnitId": "sheet",
            "range": { "startRow": 1, "startColumn": 0, "endRow": 1, "endColumn": 1 }
        });
        let remove_params = serde_json::json!({
            "unitId": "unit",
            "subUnitId": "sheet",
            "range": { "startRow": 1, "startColumn": 0, "endRow": 1, "endColumn": 0 }
        });
        let m1 = create_mutation_info(transform.id.to_string(), range_params);
        let m2 = create_mutation_info("sheet.mutation.remove-rows".to_string(), remove_params);
        let result = transform.transform_with_remove_rows(&m1, &m2);
        assert!(result.m1_prime.is_none());
    }
}
