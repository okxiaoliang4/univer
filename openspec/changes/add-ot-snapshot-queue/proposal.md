## Why

当前 snapshot 生成逻辑直接在 op 写入时同步执行，存在以下问题：
1. **性能瓶颈**：snapshot 计算阻塞 op 写入响应
2. **无法水平扩展**：snapshot 服务无法独立扩展
3. **故障隔离差**：snapshot 失败会影响正常 op 处理

需要引入异步 snapshot 队列机制，将 op 写入与 snapshot 生成解耦，并支持 Node.js 端 BullMQ 消费。

## What Changes

### 1. etcd 服务注册
- 服务启动时在 etcd 注册 lease，key 为 `ot-collaboration/{UUID}`，值为 `{IP}:{PORT}`
- 支持健康检查和自动续租
- 服务关闭时自动注销

### 2. gRPC 服务发现与通信
- 实现 `get_grpc_client(service_name)` 函数，从 etcd 获取服务地址并创建 gRPC client
- 服务地址格式：`{服务名}/{UUID}` → `{IP}:{PORT}`
- 新增 `snapshot_queue.proto` 定义 EnqueueSnapshotJob RPC
- 添加 proto build command

### 3. Snapshot 队列机制
- Op 写入时：落库 + 写入 Redis 待处理队列
- 定时（每分钟）触发 snapshot job（通过 gRPC 调用 Node BullMQ）
- **容错保证**：
  - gRPC 调用失败时写入 outbox 表/Redis
  - Node 侧补偿扫描机制确保不丢 job
  - 幂等处理防止重复 snapshot

### 4. Redis 数据结构
- `ops:{doc_id}` - ZSET，存储待处理 op（score=rev）
- `op:{op_id}` - HASH，存储 op payload
- `snapshot:checkpoint:{doc_id}` - STRING，记录已处理到的 rev
- `snapshot:lock:{doc_id}` - 分布式锁防止并发处理

## Impact

- **Affected specs**: `ot-wasm`
- **Affected code**:
  - `packages/univer-ot-wasm/Cargo.toml` - 新增依赖
  - `packages/univer-ot-wasm/src/server/config.rs` - etcd 配置
  - `packages/univer-ot-wasm/src/server/services/` - 新增 etcd、grpc、queue 服务
  - `packages/univer-ot-wasm/src/bin/ot-server.rs` - 启动时注册 etcd
  - `packages/univer-ot-wasm/proto/` - 新增 proto 文件
  - `packages/univer-ot-wasm/build.rs` - proto 编译
