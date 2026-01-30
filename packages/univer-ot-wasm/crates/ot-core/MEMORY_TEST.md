# OT Core 内存测试

## 概述

这个测试程序用于评估 `set-range-values` 操作在处理大量单元格时的内存占用和回收情况。

## 测试场景

1. **生成数据**：创建 5 个 `set-range-values` mutation，每个包含 20,000 个单元格
2. **Transform 操作**：对所有 mutation 进行两两 transform（共 10 次 transform）
3. **内存监控**：
   - 记录初始内存
   - 记录每个阶段的内存变化
   - 在操作完成后持续监控内存，观察垃圾回收行为

## 运行测试

### 基本运行

```bash
cd /Volumes/data/projects/univer/packages/univer-ot-wasm
cargo run --example memory_test --release
```

### 使用 Debug 模式（更详细的信息）

```bash
cargo run --example memory_test
```

### 使用 jemalloc（更好的内存统计）

如果想要更精确的内存统计，可以使用 jemalloc：

```bash
# 首先添加 jemalloc 依赖（可选）
cargo run --example memory_test --release --features jemalloc
```

## 输出说明

### 阶段标记

- `INITIAL`: 程序启动时的初始内存
- `AFTER_SERVICE_INIT`: 创建 TransformService 后
- `AFTER_MUTATION_GEN`: 生成所有 mutation 后
- `TRANSFORM_N`: 完成 N 次 transform 后
- `AFTER_TRANSFORMS`: 所有 transform 完成后
- `AFTER_DROP`: 显式 drop mutation 后

### 内存指标

- **RSS (Resident Set Size)**: 实际物理内存占用（更重要）
- **VSZ (Virtual Memory Size)**: 虚拟内存大小
- **Δ (Delta)**: 相对于初始状态的变化量

### 示例输出

```
=== OT Core Memory Test ===
Testing set-range-values with 20,000 cells

[INITIAL] RSS: 2.45 MB, VSZ: 4.20 GB

Creating TransformService...
[AFTER_SERVICE_INIT] RSS: 2.67 MB (+0.22 MB), VSZ: 4.20 GB (+0.00 MB)

Generating 5 mutations with 20,000 cells each...
[AFTER_MUTATION_GEN] RSS: 15.23 MB (+12.78 MB), VSZ: 4.20 GB (+0.00 MB)
Generated 5 mutations

Starting transformations...
Completed 2 transforms...
[TRANSFORM_2] RSS: 17.89 MB (+15.44 MB), VSZ: 4.20 GB (+0.00 MB)
...

Monitoring memory usage (press Ctrl+C to exit)...
Time | RSS | VSZ | RSS Δ | VSZ Δ
----------------------------------------------------------------------
   1s | 18.45 MB | 4.20 GB | +0.56 MB | +0.00 MB
   2s | 18.45 MB | 4.20 GB | +0.00 MB | +0.00 MB
   3s | 12.34 MB | 4.20 GB | -6.11 MB | +0.00 MB  <- 垃圾回收发生
...
```

## 评估标准

### 内存效率

1. **Transform 峰值内存**
   - 理想：< 50 MB for 20,000 cells × 5 mutations
   - 可接受：< 100 MB
   - 需要优化：> 100 MB

2. **内存回收**
   - 理想：在 drop 后 5 秒内回收到接近初始水平
   - 可接受：在 10 秒内回收
   - 需要优化：> 30 秒或持续增长

3. **Transform 性能**
   - 理想：10 次 transform < 100ms
   - 可接受：< 500ms
   - 需要优化：> 1s

## 问题诊断

### 内存持续增长

如果内存在监控阶段持续增长：
- 可能存在内存泄漏
- 检查是否有循环引用
- 检查是否有静态变量持有数据

### 内存未回收

如果 drop 后内存未明显下降：
- Rust 的分配器可能不会立即归还内存给 OS
- 这是正常的，内存已被 deallocate 但保留在进程中供复用
- 使用 jemalloc 可以看到更准确的 allocated 内存

### Transform 太慢

如果 transform 性能不佳：
- 检查是否有不必要的 clone
- 查看计划文档中的优化方案
- 考虑实现 zero-copy transform

## 相关文档

- 优化计划：`/Users/jelf/.claude/plans/reflective-toasting-wilkinson.md`
- Transform 实现：`src/transforms/`
- 核心类型：`src/types.rs`

## 扩展测试

### 测试不同数据量

修改 `memory_test.rs` 中的参数：

```rust
// 测试 50,000 cells
generate_set_range_values("workbook1", "sheet1", 50_000)

// 生成更多 mutations
let mutations: Vec<MutationInfo> = (0..10).map(...)
```

### 添加其他 mutation 类型

```rust
// 测试 insert-row
let mutation = MutationInfo {
    id: "sheet.mutation.insert-row".to_string(),
    params: json!({
        "unitId": "workbook1",
        "subUnitId": "sheet1",
        "range": {
            "startRow": 100,
            "endRow": 200,
            "startColumn": 0,
            "endColumn": 100,
        }
    }),
};
```

## 性能基准

在 MacBook Pro (M1, 16GB) 上的预期结果：

| 指标 | 预期值 |
|------|--------|
| 初始内存 | ~2-3 MB |
| 生成 mutation 后 | ~15-20 MB |
| Transform 峰值 | ~25-30 MB |
| Drop 后 | ~5-10 MB |
| Transform 总时间 | < 200ms |

## 持续监控建议

对于生产环境的内存监控：

1. **使用 Prometheus + Grafana**
   - 导出内存指标
   - 设置告警阈值

2. **添加内存统计到日志**
   - 定期记录内存使用
   - 关联到操作类型和数据量

3. **集成到 CI/CD**
   - 运行基准测试
   - 检测性能退化
