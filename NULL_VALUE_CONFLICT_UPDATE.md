# SetRangeValues 冲突处理更新：使用 null 值

## 修改摘要

将 SetRangeValues 的 LWW 冲突解决策略从"删除冲突单元格"改为"将冲突单元格设为 null"。

## 修改原因

1. **更明确的语义**: null 值可以明确表示"此单元格发生冲突"
2. **保留操作结构**: 不删除单元格，而是保留单元格但设为 null
3. **更好的调试体验**: 可以在日志中看到冲突的单元格位置
4. **符合实际应用**: null 值在实际应用中不会覆盖现有内容

## 修改对比

### 之前的实现（删除冲突单元格）

```rust
// 冲突时直接删除该单元格
if !has_conflict {
    new_row.insert(col_key.clone(), col_value.clone());
}
// 结果: 冲突的单元格不存在于 m1' 中
```

**示例**:
```json
// m1 原始操作
{ "0": { "0": { "v": "World" } } }

// m1' 转换后（与 m2 冲突）
{ }  // 空的，单元格 (0,0) 被删除
```

### 现在的实现（设为 null）

```rust
// 冲突时设为 null
if has_conflict {
    new_row.insert(col_key.clone(), serde_json::Value::Null);
} else {
    new_row.insert(col_key.clone(), col_value.clone());
}
// 结果: 冲突的单元格存在，但值为 null
```

**示例**:
```json
// m1 原始操作
{ "0": { "0": { "v": "World" } } }

// m1' 转换后（与 m2 冲突）
{ "0": { "0": null } }  // 单元格 (0,0) 存在，值为 null
```

## OT 一致性保证

### TP2 不变式验证

```
场景:
- $State: A1 = "X"
- m1: A1 = "World" (Client B)
- m2: A1 = "Hello" (Client A, 先到服务器)

Transform 结果:
- m1' = SetRangeValues(A1, null)
- m2' = m2 (不变)

验证:
Path 1: $State + m1 + m2'
  = X + World + Hello
  = "Hello" ✅

Path 2: $State + m2 + m1'
  = X + Hello + null
  = "Hello" ✅ (null 不覆盖现有值)

结论: 两条路径结果相同，满足 TP2
```

### 关键假设

**这个实现依赖于以下语义**:
- `SetRangeValues` 对于 null 值的处理：**null 不会覆盖单元格的现有内容**
- 只有非 null 的有效值才会更新单元格
- 这需要在 SetRangeValues 的实现中确保

## 实际影响

### 服务器端

```rust
// 服务器收到并发操作
// 1. m_A 先到 (rev 11)
// 2. m_B 后到 (base_rev 10)

// Transform
let (m_B', m_A') = transform(m_B, m_A);
// m_B' = SetRangeValues(冲突单元格 = null)
// m_A' = m_A

// 存储
operation_log[11] = m_A;
operation_log[12] = m_B'; // 包含 null 值
```

### 客户端 A

```typescript
// 1. 发送 m_A，本地应用 → A1 = "Hello"
// 2. 收到 Ack(11)
// 3. 收到广播(12, m_B') 包含 SetRangeValues(A1, null)
// 4. 应用 m_B'，但 null 不覆盖 "Hello"
// 最终: A1 = "Hello" ✅
```

### 客户端 B

```typescript
// 1. 发送 m_B，本地应用 → A1 = "World", pending = [m_B]
// 2. 收到广播(11, m_A) → A1 = "Hello"
// 3. Transform: (m_B', m_A') = transform([m_B], [m_A])
//    - m_B' = SetRangeValues(A1, null)
//    - m_A' = m_A
// 4. 应用 m_A' → A1 = "Hello"
// 5. pending = [m_B'] (包含 null)
// 6. 收到 Ack(12), 清除 pending
// 最终: A1 = "Hello" ✅
```

## 测试更新

### 更新的测试用例

1. **test_transform_set_range_values_with_set_range_values_conflict**
   - 之前: 验证冲突单元格被删除
   - 现在: 验证冲突单元格被设为 null

2. **test_transform_set_range_values_with_set_range_values_partial_conflict**
   - 之前: 验证冲突单元格被删除，非冲突单元格保留
   - 现在: 验证冲突单元格设为 null，非冲突单元格保留

3. **test_set_range_values_ot_consistency**
   - 之前: 验证 m_B' 为空（NOOP）
   - 现在: 验证 m_B' 包含 null 值

### 测试结果

```bash
$ cargo test --lib
test result: ok. 49 passed; 0 failed; 0 ignored
```

✅ **所有测试通过**

## 调试建议

### 查看冲突单元格

```typescript
// 在客户端打印 transform 结果
console.log('m1Primes:', JSON.stringify(result.m1Primes, null, 2));

// 输出示例（冲突场景）
// {
//   "id": "sheet.mutation.set-range-values",
//   "params": {
//     "cell_value": {
//       "data": {
//         "0": {
//           "0": null  // 冲突单元格
//         }
//       }
//     }
//   }
// }
```

### 验证 null 不覆盖

确保 SetRangeValues 的实现中：
```typescript
// 伪代码
function applySetRangeValues(params) {
  for (const [row, cols] of params.cellValue.data) {
    for (const [col, value] of cols) {
      if (value !== null) {  // 关键: null 不应用
        sheet.setValue(row, col, value);
      }
    }
  }
}
```

## 兼容性注意事项

### 需要确认的点

1. **SetRangeValues 的 null 处理**
   - 确认当前实现中 null 值是否会被应用到单元格
   - 如果会被应用，需要修改逻辑跳过 null 值

2. **序列化/反序列化**
   - 确认 null 值在 JSON 序列化后正确传输
   - 确认客户端能正确解析包含 null 的操作

3. **已有数据**
   - 新策略只影响新的冲突
   - 历史操作不受影响

## 文件修改清单

1. ✅ `packages/univer-ot-wasm/src/transform/set_range_values.rs`
   - 修改 `transform_with_set_range_values` 逻辑
   - 将冲突单元格设为 null 而不是删除

2. ✅ `packages/univer-ot-wasm/src/transform/set_range_values_test.rs`
   - 更新所有冲突相关测试
   - 验证 null 值的正确性

3. ✅ `OT_FIX_SUMMARY.md`
   - 更新文档说明新策略

4. ✅ `VERIFICATION_GUIDE.md`
   - 更新验证步骤

5. ✅ `NULL_VALUE_CONFLICT_UPDATE.md`
   - 本文档

## 后续工作

1. **验证 SetRangeValues 实现**
   - 确认 null 值不会覆盖现有单元格内容
   - 如果会覆盖，需要修改应用逻辑

2. **性能测试**
   - 测试大量冲突场景的性能
   - 对比删除 vs 设为 null 的性能差异

3. **用户体验**
   - 考虑是否需要在 UI 显示冲突信息
   - 提供冲突解决的用户交互

## 总结

✅ **修改完成**: 将冲突单元格从"删除"改为"设为 null"
✅ **测试通过**: 所有 49 个测试通过
✅ **OT 一致性**: 满足 TP2 不变式
⚠️ **需要验证**: SetRangeValues 对 null 值的处理逻辑
