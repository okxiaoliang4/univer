use serde::{Deserialize, Serialize};
      use serde_json::json;
      use std::time::Instant;

      #[derive(Debug, Clone, Serialize, Deserialize)]
      #[serde(rename_all = "camelCase")]
      struct RangeParams {
          unit_id: String,
          sub_unit_id: String,
          ranges: Vec<Range>,
      }

      #[derive(Debug, Clone, Serialize, Deserialize)]
      #[serde(rename_all = "camelCase")]
      struct Range {
          start_row: i32,
          end_row: i32,
          start_column: i32,
          end_column: i32,
      }

      #[test]
      fn bench_full_transform_flow() {
          // 模拟真实场景: MutationInfo.params 是 serde_json::Value
          let original_value: serde_json::Value = json!({
              "unitId": "workbook1",
              "subUnitId": "sheet1",
              "ranges": [
                  {"startRow": 5, "endRow": 10, "startColumn": 0, "endColumn": 5},
                  {"startRow": 15, "endRow": 20, "startColumn": 0, "endColumn": 5},
                  {"startRow": 25, "endRow": 30, "startColumn": 0, "endColumn": 5}
              ]
          });

          const ITERATIONS: usize = 10000;

          println!("\n=== 完整 Transform 流程对比 ({} 次) ===\n", ITERATIONS);

          // ========== 方案 A: 类型化 (解析 -> 修改 -> 序列化) ==========
          let start = Instant::now();
          for _ in 0..ITERATIONS {
              // 1. Clone 原始 Value
              let cloned = original_value.clone();

              // 2. 解析为具体类型
              let mut typed: RangeParams = serde_json::from_value(cloned).unwrap();

              // 3. 修改
              for range in typed.ranges.iter_mut() {
                  if range.start_row >= 3 {
                      range.start_row += 2;
                      range.end_row += 2;
                  }
              }

              // 4. 序列化回 Value
              let _result: serde_json::Value = serde_json::to_value(typed).unwrap();
          }
          let typed_time = start.elapsed();

          // ========== 方案 B: 动态 JSON (直接操作 Value) ==========
          let start = Instant::now();
          for _ in 0..ITERATIONS {
              // 1. Clone 原始 Value
              let mut cloned = original_value.clone();

              // 2. 直接操作 JSON
              if let Some(arr) = cloned.get_mut("ranges").and_then(|r| r.as_array_mut()) {
                  for range in arr.iter_mut() {
                      if let Some(obj) = range.as_object_mut() {
                          if let Some(sr) = obj.get("startRow").and_then(|v| v.as_i64()) {
                              if sr >= 3 {
                                  obj.insert("startRow".to_string(), json!(sr + 2));
                              }
                          }
                          if let Some(er) = obj.get("endRow").and_then(|v| v.as_i64()) {
                              if er >= 3 {
                                  obj.insert("endRow".to_string(), json!(er + 2));
                              }
                          }
                      }
                  }
              }

              // 3. 已经是 Value，无需序列化
              let _result = cloned;
          }
          let json_time = start.elapsed();

          println!("方案 A (类型化: 解析->修改->序列化): {:?}", typed_time);
          println!("方案 B (动态 JSON: 直接操作):        {:?}", json_time);

          let ratio = typed_time.as_nanos() as f64 / json_time.as_nanos() as f64;
          if ratio > 1.0 {
              println!("\n动态 JSON 快 {:.2}x ✓", ratio);
          } else {
              println!("\n静态类型快 {:.2}x", 1.0 / ratio);
          }

          println!("\n=== 内存分配对比 ===\n");

          // 方案 A 分配: clone + 解析堆对象 + 序列化堆对象
          // 方案 B 分配: clone only

          println!("方案 A 分配: Value.clone() + 类型解析 + Value 序列化 = 3次主要分配");
          println!("方案 B 分配: Value.clone() + 少量字符串 key = 1+N次分配");
      }