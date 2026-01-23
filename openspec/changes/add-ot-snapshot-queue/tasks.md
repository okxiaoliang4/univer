## 1. 基础设施

- [ ] 1.1 更新 Cargo.toml，添加依赖：`etcd-client`, `tonic`, `prost`, `prost-build`
- [ ] 1.2 创建 `proto/` 目录和 proto 文件
- [ ] 1.3 创建 `build.rs` 实现 proto 编译
- [ ] 1.4 更新 `package.json` 添加 proto build script

## 2. etcd 服务注册

- [ ] 2.1 扩展 Config 结构，添加 etcd 配置项
- [ ] 2.2 实现 `EtcdService`：连接、lease 创建、KV 注册
- [ ] 2.3 实现服务注册逻辑：启动时注册 `ot-collaboration/{UUID}` → `{IP}:{PORT}`
- [ ] 2.4 实现 lease keepalive 自动续租
- [ ] 2.5 实现优雅关闭时自动注销
- [ ] 2.6 集成到 `ot-server.rs` 启动流程

## 3. gRPC 服务发现

- [ ] 3.1 实现 `get_grpc_client(service_name)` 函数：从 etcd 获取服务地址
- [ ] 3.2 实现地址选择策略（随机/轮询）
- [ ] 3.3 实现连接缓存和失效重建

## 4. gRPC Snapshot Queue Client

- [ ] 4.1 定义 `snapshot_queue.proto`：EnqueueSnapshotJob RPC
- [ ] 4.2 创建 `build.rs` 实现 proto 编译
- [ ] 4.3 生成 Rust gRPC client 代码
- [ ] 4.4 实现 `SnapshotQueueClient`：封装 enqueue 调用

## 5. Redis Op 队列

- [ ] 5.1 扩展 Redis service，添加 op 队列操作方法
- [ ] 5.2 实现 `add_pending_op(doc_id, op_id, rev, payload)` - 写入 ZSET + HASH
- [ ] 5.3 实现 `get_pending_ops(doc_id, from_rev, limit)` - 读取待处理 op
- [ ] 5.4 实现 `remove_processed_ops(doc_id, up_to_rev)` - 删除已处理 op
- [ ] 5.5 实现 `get_checkpoint(doc_id)` / `set_checkpoint(doc_id, rev)`
- [ ] 5.6 实现 `acquire_lock(doc_id)` / `release_lock(doc_id)` - 分布式锁

## 6. Snapshot Queue 集成

- [ ] 6.1 实现 `SnapshotQueueService`：封装 enqueue 逻辑
- [ ] 6.2 修改 `OTService.apply_changeset`：写入 op 后调用 enqueue
- [ ] 6.3 实现 Outbox 机制：gRPC 失败时写入 `snapshot:outbox`
- [ ] 6.4 实现 Outbox 后台处理任务：定期重试失败的 enqueue
- [ ] 6.5 添加 feature flag 控制新旧逻辑切换

## 7. 定时触发机制

- [ ] 7.1 实现定时扫描任务：每分钟检查有新 op 的 doc
- [ ] 7.2 实现批量 enqueue 逻辑
- [ ] 7.3 集成到服务启动流程

## 8. 测试与文档

- [ ] 8.1 编写 etcd 服务注册单元测试
- [ ] 8.2 编写 gRPC client 单元测试
- [ ] 8.3 编写 Redis op 队列单元测试
- [ ] 8.4 编写集成测试：完整 enqueue 流程
- [ ] 8.5 更新 README 文档
- [ ] 8.6 添加环境变量配置说明
