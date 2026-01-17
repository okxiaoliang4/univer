# OT 修复验证指南

## 快速验证步骤

### 1. 运行 Rust 测试
```bash
cd packages/univer-ot-wasm
cargo test --lib

# 预期结果：49 passed
```

### 2. 手动测试场景

**场景**: 两个客户端同时修改同一单元格

**步骤**:
1. 打开两个浏览器窗口（A 和 B）
2. 断开 B 的网络（或暂停 B 的 Socket 连接）
3. A 在单元格 A1 输入 "Hello"
4. B 在单元格 A1 输入 "World"
5. 恢复 B 的网络连接

**预期结果**:
- 两个客户端最终都显示 "Hello"（A 的操作先到服务器）
- B 的 "World" 被覆盖，但 B 的 UI 不会闪烁
- 服务器日志显示 B 的操作被转换为 SetRangeValues(A1, null)

**验证要点**:
- ✅ 最终状态一致
- ✅ 无 UI 闪烁（不撤销本地操作）
- ✅ 服务器日志显示 transform 操作

## 测试覆盖率

### SetRangeValues × SetRangeValues
1. ✅ 完全冲突（同一单元格）
2. ✅ 无冲突（不同单元格）
3. ✅ 部分冲突（多个单元格，部分冲突）
4. ✅ OT 一致性（完整服务器-客户端交互）

### 其他 Transform 组合
- ✅ SetRangeValues × InsertRow
- ✅ SetRangeValues × InsertCol
- ✅ SetRangeValues × RemoveRows
- ✅ SetRangeValues × RemoveCol
- ✅ InsertRow × InsertRow
- ✅ RemoveRows × RemoveRows
- ✅ (其他 40+ 测试)

## 调试技巧

### 服务器端日志
```rust
// 在 ot.rs 中查看 transform 结果
info!("Transform result: m1_prime={:?}, m2_prime={:?}", m1_prime, m2_prime);
```

### 客户端日志
```typescript
// 在 collaboration.service.ts 中查看 OT 结果
this._logger.log(`OT result: m1Primes=${result.m1Primes.length}, m2Primes=${result.m2Primes.length}`);
```

### 检查 null 值操作
LWW 策略会将冲突单元格设为 null：
```typescript
// m1_prime 的冲突单元格会被设为 null
// 例如: { "0": { "0": null } } 表示 A1 单元格冲突
// null 值不会覆盖现有单元格内容，确保 LWW 语义
```

## 已知限制

1. **LWW 策略**: 当前是服务器先到优先，未来可考虑其他策略
2. **无冲突通知**: 用户不知道自己的操作被覆盖了
3. **单元格粒度**: 冲突检测按单元格，未来可按属性（value, style）

## 预期行为

### 场景 A: 无冲突
```
Client A: 修改 A1 = "Hello"
Client B: 修改 B1 = "World"
结果: A1 = "Hello", B1 = "World" ✅
```

### 场景 B: 完全冲突
```
Client A: 修改 A1 = "Hello" (先到)
Client B: 修改 A1 = "World" (后到)
结果: A1 = "Hello" (A 的操作保留) ✅
```

### 场景 C: 部分冲突
```
Client A: 修改 A1 = "Hello", A2 = "Foo" (先到)
Client B: 修改 A1 = "World", B1 = "Bar" (后到)
结果: A1 = "Hello", A2 = "Foo", B1 = "Bar" ✅
     (A1 冲突，A 保留；A2 和 B1 无冲突，都保留)
```

## 回滚方案

如果发现问题，可以回滚到之前的版本：

```bash
# 查看修改的文件
git diff --name-only

# 回滚特定文件
git checkout HEAD~1 -- packages/univer-ot-wasm/src/transform/set_range_values.rs
git checkout HEAD~1 -- packages/univer-ot-wasm/src/transform/set_range_values_test.rs
git checkout HEAD~1 -- packages/collaboration/src/services/collaboration.service.ts

# 重新测试
cd packages/univer-ot-wasm && cargo test --lib
```

## 联系信息

如有问题，请参考：
- OT 算法文档：`packages/univer-ot-wasm/README.md`
- 协作服务：`packages/collaboration/README.md`
- 详细修复总结：`OT_FIX_SUMMARY.md`
