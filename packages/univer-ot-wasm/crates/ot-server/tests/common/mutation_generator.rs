use ot_core::MutationInfoWithOpId;
use serde_json::{json, Map, Value};

/// Generate a large setRangeValues mutation with specified number of cells
pub fn generate_set_range_values_mutation(
    unit_id: &str,
    sub_unit_id: &str,
    cell_count: usize,
    op_id: String,
) -> MutationInfoWithOpId {
    println!(
        "🔧 Generating setRangeValues mutation with {} cells",
        cell_count
    );

    // Calculate grid dimensions (try to make it roughly square)
    let cols = (cell_count as f64).sqrt().ceil() as usize;
    let rows = (cell_count + cols - 1) / cols;

    println!(
        "   Grid dimensions: {}x{} (rows x cols)",
        rows, cols
    );

    // Build the cellValue object
    let mut cell_value_map = Map::new();

    let mut generated_count = 0;
    for row in 0..rows {
        let mut row_map = Map::new();

        for col in 0..cols {
            if generated_count >= cell_count {
                break;
            }

            // Create cell value with rich data
            let cell_data = json!({
                "v": format!("Cell_R{}C{}", row, col),
                "t": 1, // CellValueType.STRING
            });

            row_map.insert(col.to_string(), cell_data);
            generated_count += 1;
        }

        if !row_map.is_empty() {
            cell_value_map.insert(row.to_string(), Value::Object(row_map));
        }
    }

    let params = json!({
        "unitId": unit_id,
        "subUnitId": sub_unit_id,
        "cellValue": cell_value_map,
    });

    let mutation = MutationInfoWithOpId {
        id: "sheet.mutation.set-range-values".to_string(),
        params,
        op_id,
    };

    println!("✅ Generated mutation with {} cells", generated_count);

    mutation
}

/// Generate multiple setRangeValues mutations, each with specified cell count
pub fn generate_multiple_set_range_values(
    unit_id: &str,
    sub_unit_id: &str,
    mutation_count: usize,
    cells_per_mutation: usize,
    client_id: &str,
) -> Vec<MutationInfoWithOpId> {
    println!(
        "🔧 Generating {} mutations with {} cells each",
        mutation_count, cells_per_mutation
    );

    let mut mutations = Vec::with_capacity(mutation_count);

    for i in 0..mutation_count {
        let op_id = format!("{}_op_{}", client_id, i);
        let mutation = generate_set_range_values_mutation(
            unit_id,
            sub_unit_id,
            cells_per_mutation,
            op_id,
        );
        mutations.push(mutation);
    }

    println!(
        "✅ Generated {} mutations (total {} cells)",
        mutations.len(),
        mutation_count * cells_per_mutation
    );

    mutations
}

/// Generate a mutation that modifies a specific range of cells
pub fn generate_range_mutation(
    unit_id: &str,
    sub_unit_id: &str,
    start_row: usize,
    start_col: usize,
    end_row: usize,
    end_col: usize,
    value_prefix: &str,
    op_id: String,
) -> MutationInfoWithOpId {
    let mut cell_value_map = Map::new();

    for row in start_row..=end_row {
        let mut row_map = Map::new();

        for col in start_col..=end_col {
            let cell_data = json!({
                "v": format!("{}_R{}C{}", value_prefix, row, col),
                "t": 1,
            });

            row_map.insert(col.to_string(), cell_data);
        }

        cell_value_map.insert(row.to_string(), Value::Object(row_map));
    }

    let params = json!({
        "unitId": unit_id,
        "subUnitId": sub_unit_id,
        "cellValue": cell_value_map,
    });

    MutationInfoWithOpId {
        id: "sheet.mutation.set-range-values".to_string(),
        params,
        op_id,
    }
}

/// Calculate the size of a mutation in bytes (approximate)
pub fn calculate_mutation_size(mutation: &MutationInfoWithOpId) -> usize {
    serde_json::to_string(mutation)
        .map(|s| s.len())
        .unwrap_or(0)
}

/// Print statistics about a mutation
pub fn print_mutation_stats(mutations: &[MutationInfoWithOpId]) {
    println!("\n📊 Mutation Statistics:");
    println!("   Count: {}", mutations.len());

    if mutations.is_empty() {
        return;
    }

    let total_size: usize = mutations.iter().map(|m| calculate_mutation_size(m)).sum();
    let avg_size = total_size / mutations.len();

    println!("   Total size: {:.2} KB", total_size as f64 / 1024.0);
    println!("   Average size: {:.2} KB", avg_size as f64 / 1024.0);

    if let Some(first) = mutations.first() {
        println!("   Mutation ID: {}", first.id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_small_mutation() {
        let mutation = generate_set_range_values_mutation(
            "test_unit",
            "test_sheet",
            100,
            "op_1".to_string(),
        );

        assert_eq!(mutation.id, "sheet.mutation.set-range-values");
        assert_eq!(mutation.op_id, "op_1");

        // Verify params structure
        let params = &mutation.params;
        assert_eq!(params["unitId"], "test_unit");
        assert_eq!(params["subUnitId"], "test_sheet");
        assert!(params["cellValue"].is_object());
    }

    #[test]
    fn test_generate_multiple() {
        let mutations =
            generate_multiple_set_range_values("unit", "sheet", 3, 50, "client1");

        assert_eq!(mutations.len(), 3);

        for (i, mutation) in mutations.iter().enumerate() {
            assert_eq!(mutation.op_id, format!("client1_op_{}", i));
        }
    }

    #[test]
    fn test_generate_range() {
        let mutation = generate_range_mutation(
            "unit",
            "sheet",
            0,
            0,
            2,
            2,
            "TEST",
            "op_range".to_string(),
        );

        let cell_value = &mutation.params["cellValue"];

        // Should have 3 rows (0, 1, 2)
        assert!(cell_value["0"].is_object());
        assert!(cell_value["1"].is_object());
        assert!(cell_value["2"].is_object());

        // Each row should have 3 columns (0, 1, 2)
        assert!(cell_value["0"]["0"].is_object());
        assert!(cell_value["0"]["1"].is_object());
        assert!(cell_value["0"]["2"].is_object());
    }

    #[test]
    fn test_calculate_size() {
        let mutation = generate_set_range_values_mutation(
            "unit",
            "sheet",
            10,
            "op_1".to_string(),
        );

        let size = calculate_mutation_size(&mutation);
        assert!(size > 0);
        println!("Mutation size: {} bytes", size);
    }
}
