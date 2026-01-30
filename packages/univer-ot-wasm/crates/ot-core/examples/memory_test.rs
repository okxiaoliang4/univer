use ot_core::{MutationInfo, TransformService};
use serde_json::json;
use std::time::{Duration, Instant};

#[cfg(target_os = "linux")]
fn get_memory_usage() -> Option<(usize, usize)> {
    use std::fs;

    let status = fs::read_to_string("/proc/self/status").ok()?;
    let mut vm_rss = 0;
    let mut vm_size = 0;

    for line in status.lines() {
        if line.starts_with("VmRSS:") {
            vm_rss = line
                .split_whitespace()
                .nth(1)?
                .parse::<usize>()
                .ok()?;
        } else if line.starts_with("VmSize:") {
            vm_size = line
                .split_whitespace()
                .nth(1)?
                .parse::<usize>()
                .ok()?;
        }
    }

    Some((vm_rss, vm_size))
}

#[cfg(target_os = "macos")]
fn get_memory_usage() -> Option<(usize, usize)> {
    use std::process::Command;

    let output = Command::new("ps")
        .args(&["-o", "rss=,vsz=", "-p", &std::process::id().to_string()])
        .output()
        .ok()?;

    let output_str = String::from_utf8(output.stdout).ok()?;
    let parts: Vec<&str> = output_str.trim().split_whitespace().collect();

    if parts.len() >= 2 {
        let rss = parts[0].parse::<usize>().ok()?;
        let vsz = parts[1].parse::<usize>().ok()?;
        Some((rss, vsz))
    } else {
        None
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn get_memory_usage() -> Option<(usize, usize)> {
    None
}

fn format_memory(kb: usize) -> String {
    if kb < 1024 {
        format!("{} KB", kb)
    } else if kb < 1024 * 1024 {
        format!("{:.2} MB", kb as f64 / 1024.0)
    } else {
        format!("{:.2} GB", kb as f64 / (1024.0 * 1024.0))
    }
}

/// Generate a set-range-values mutation with specified number of cells
fn generate_set_range_values(doc_id: &str, sheet_id: &str, num_cells: usize) -> MutationInfo {
    let rows = (num_cells as f64).sqrt().ceil() as usize;
    let cols = (num_cells + rows - 1) / rows;

    let mut cell_value = serde_json::Map::new();

    for r in 0..rows {
        let mut row_map = serde_json::Map::new();
        for c in 0..cols {
            if r * cols + c >= num_cells {
                break;
            }
            let mut cell = serde_json::Map::new();
            cell.insert("v".to_string(), json!(format!("Cell_{}", r * cols + c)));
            cell.insert("t".to_string(), json!(1)); // CellValueType.STRING
            row_map.insert(c.to_string(), json!(cell));
        }
        if !row_map.is_empty() {
            cell_value.insert(r.to_string(), json!(row_map));
        }
    }

    MutationInfo {
        id: "sheet.mutation.set-range-values".to_string(),
        params: json!({
            "unitId": doc_id,
            "subUnitId": sheet_id,
            "cellValue": cell_value,
        }),
    }
}

fn print_memory_stats(label: &str, start_mem: Option<(usize, usize)>) {
    if let Some((rss, vsz)) = get_memory_usage() {
        if let Some((start_rss, start_vsz)) = start_mem {
            let rss_diff = rss as i64 - start_rss as i64;
            let vsz_diff = vsz as i64 - start_vsz as i64;
            println!(
                "[{}] RSS: {} ({:+}), VSZ: {} ({:+})",
                label,
                format_memory(rss),
                format_memory(rss_diff.abs() as usize),
                format_memory(vsz),
                format_memory(vsz_diff.abs() as usize)
            );
        } else {
            println!(
                "[{}] RSS: {}, VSZ: {}",
                label,
                format_memory(rss),
                format_memory(vsz)
            );
        }
    } else {
        println!("[{}] Memory tracking not available on this platform", label);
    }
}

fn main() {
    println!("=== OT Core Memory Test ===");
    println!("Testing set-range-values with 20,000 cells\n");

    // Record initial memory
    let start_mem = get_memory_usage();
    print_memory_stats("INITIAL", None);
    println!();

    // Create transform service
    println!("Creating TransformService...");
    let service = TransformService::new();
    print_memory_stats("AFTER_SERVICE_INIT", start_mem);
    println!();

    // Generate test mutations
    println!("Generating 5 mutations with 20,000 cells each...");
    let mutations: Vec<MutationInfo> = (0..5)
        .map(|i| {
            generate_set_range_values(
                "workbook1",
                &format!("sheet{}", i),
                20_000,
            )
        })
        .collect();

    print_memory_stats("AFTER_MUTATION_GEN", start_mem);
    println!("Generated {} mutations", mutations.len());
    println!();

    // Perform transformations
    println!("Starting transformations...");
    let transform_start = Instant::now();
    let mut transform_count = 0;

    for i in 0..mutations.len() {
        for j in (i + 1)..mutations.len() {
            let m1 = &mutations[i];
            let m2 = &mutations[j];

            let result = service.transform(m1, m2);
            transform_count += 1;

            // Verify transform succeeded
            if result.error.is_some() {
                println!("Transform error: {:?}", result.error);
            }

            // Print progress every 2 transforms
            if transform_count % 2 == 0 {
                println!("Completed {} transforms...", transform_count);
                print_memory_stats(&format!("TRANSFORM_{}", transform_count), start_mem);
            }
        }
    }

    let transform_duration = transform_start.elapsed();
    println!();
    println!("Completed {} transforms in {:?}", transform_count, transform_duration);
    print_memory_stats("AFTER_TRANSFORMS", start_mem);
    println!();

    // Drop mutations to free memory
    println!("Dropping mutations...");
    drop(mutations);
    print_memory_stats("AFTER_DROP", start_mem);
    println!();

    // Monitor memory usage every second
    println!("Monitoring memory usage (press Ctrl+C to exit)...");
    println!("Time | RSS | VSZ | RSS Δ | VSZ Δ");
    println!("{}", "-".repeat(70));

    let monitor_start = Instant::now();
    let mut last_mem = start_mem;

    loop {
        std::thread::sleep(Duration::from_secs(1));

        let elapsed = monitor_start.elapsed().as_secs();

        if let Some((rss, vsz)) = get_memory_usage() {
            let (rss_delta, vsz_delta) = if let Some((last_rss, last_vsz)) = last_mem {
                (
                    rss as i64 - last_rss as i64,
                    vsz as i64 - last_vsz as i64,
                )
            } else {
                (0, 0)
            };

            println!(
                "{:4}s | {} | {} | {:+} | {:+}",
                elapsed,
                format_memory(rss),
                format_memory(vsz),
                format_memory(rss_delta.abs() as usize),
                format_memory(vsz_delta.abs() as usize)
            );

            last_mem = Some((rss, vsz));
        } else {
            println!("{}s | Memory tracking not available", elapsed);
        }
    }
}
