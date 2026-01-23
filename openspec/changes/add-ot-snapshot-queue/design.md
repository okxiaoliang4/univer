## Context

当前 Univer OT Server 的 snapshot 机制是同步的：每写入 N 个 op 后立即计算 snapshot。这导致：
- 写入延迟高（snapshot 计算可能很慢）
- 无法独立扩展 snapshot 处理能力
- snapshot 服务故障会阻塞正常协作

目标是将 snapshot 生成异步化，由独立的 Node.js BullMQ worker 处理，同时保证数据一致性和容错。

**约束**：
- Rust 无成熟 BullMQ 客户端，需通过 gRPC 与 Node 通信
- 需要服务发现机制（etcd）支持多实例部署
- 必须保证 op 不丢失，即使 enqueue 失败

## Goals / Non-Goals

### Goals
- 实现 etcd 服务注册与发现
- 实现 gRPC 通信层，支持调用 Node BullMQ 服务
- 实现异步 snapshot 队列，每分钟触发一次
- 保证服务边界稳定：enqueue 失败不丢数据
- 支持幂等处理和分布式锁

### Non-Goals
- 不实现 Rust 原生 BullMQ 协议（复杂度高、兼容性风险）
- 不修改现有 op 处理主流程（仅新增队列逻辑）
- 不实现 snapshot 计算逻辑（由 Node worker 处理）

## Decisions

### Decision 1: 使用 etcd 进行服务注册
**选择**: etcd lease + KV  
**原因**:
- etcd 是成熟的分布式 KV 存储，适合服务发现
- lease 机制天然支持健康检查和自动清理
- Rust 生态有成熟的 etcd-client

**Key 格式**: `ot-collaboration/{UUID}` → `{IP}:{PORT}`

**替代方案**:
- Consul: 功能更重，项目未使用
- Redis: 不支持 lease，需自己实现 TTL 续期
- Kubernetes Service: 依赖 K8s 环境

### Decision 2: 使用 gRPC 与 Node BullMQ 服务通信
**选择**: tonic (gRPC) + prost (protobuf)  
**原因**:
- gRPC 性能好、类型安全
- 支持双向流、重试、超时
- Rust/Node 生态都有成熟支持

**服务发现**:
- 实现 `get_grpc_client(service_name)` 函数
- 从 etcd 读取 `{service_name}/*` 前缀下的所有 key
- 随机/轮询选择一个地址创建 gRPC client
- 支持连接缓存和失败重试

**Proto 定义** (`snapshot_queue.proto`):
```protobuf
service SnapshotQueueService {
  rpc EnqueueSnapshotJob(EnqueueRequest) returns (EnqueueResponse);
}

message EnqueueRequest {
  string doc_id = 1;
  int64 target_rev = 2;
  int64 timestamp = 3;
}

message EnqueueResponse {
  bool success = 1;
  string job_id = 2;
  string error = 3;
}
```

### Decision 3: Op 先持久化，再 enqueue
**选择**: 写入 Redis ZSET + DB 后再调 gRPC  
**原因**:
- 保证 op 不丢失
- enqueue 只是触发信号，不是数据传输
- 失败可重试或补偿

**流程**:
```
Rust OT Server                    Node BullMQ Service
     │                                    │
     ├─① 写入 DB (operation_log)          │
     ├─② 写入 Redis (ops:{doc_id} ZSET)   │
     ├─③ gRPC EnqueueSnapshotJob ────────►│
     │   (如果失败，写入 outbox)            ├─④ BullMQ queue.add()
     │                                    │
```

### Decision 4: 容错保证 - Outbox + 补偿扫描
**选择**: 双保险机制

**机制 A - Rust 侧 Outbox**:
- gRPC 调用失败时，写入 `snapshot_outbox` 表/Redis list
- 后台任务定期重试 outbox 中的 job

**机制 B - Node 侧补偿扫描**:
- 每分钟扫描 Redis `ops:*` keys
- 发现有未处理 op 且无 pending job 的 doc → 强制 enqueue
- 使用 `snapshot:lock:{doc_id}` 防止重复

**幂等保证**:
- job payload 包含 `target_rev`
- worker 处理时校验 `checkpoint:{doc_id}` 
- 如果 `target_rev <= checkpoint`，跳过执行

### Decision 5: Redis 数据结构设计
```
# 待处理 op 队列（按 doc 分桶）
ops:{doc_id}            ZSET    score=rev, member=op_id
op:{op_id}              HASH    {mutation_id, params, user_id, created_at}

# Snapshot 状态
snapshot:checkpoint:{doc_id}    STRING  已处理到的最大 rev
snapshot:lock:{doc_id}          STRING  分布式锁（带 TTL）

# Outbox（gRPC 失败时写入）
snapshot:outbox                 LIST    {doc_id, target_rev, timestamp}
```

### Decision 6: 每分钟触发（时间调度）
**选择**: BullMQ repeatable job  
**原因**:
- BullMQ 原生支持 repeatable job
- 自动处理 cron 调度、失败重试
- 无需 Rust 侧维护定时器

**实现**:
- Node 服务启动时注册一个 repeatable job（每 60 秒）
- Job handler 扫描有新 op 的 doc 并处理

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| gRPC 服务不可用导致 job 积压 | Outbox + Node 补偿扫描双保险 |
| etcd 不可用导致服务发现失败 | 本地缓存 + 降级到配置文件 |
| Redis 数据丢失 | Redis 持久化 + DB 作为最终数据源 |
| 重复处理同一 snapshot | checkpoint 版本校验 + 分布式锁 |
| Snapshot 计算失败 | BullMQ retry 机制 + 死信队列 |

## Migration Plan

1. **Phase 1**: 实现 etcd 服务注册（不影响现有功能）
2. **Phase 2**: 实现 gRPC 服务发现和 proto build
3. **Phase 3**: 实现 op 写入 Redis 队列（与现有逻辑并行）
4. **Phase 4**: 实现 gRPC enqueue + outbox
5. **Phase 5**: Node 侧实现 BullMQ worker + 补偿扫描
6. **Phase 6**: 移除旧的同步 snapshot 逻辑

**回滚**: 每个 phase 可独立回滚，feature flag 控制新旧逻辑切换

## Open Questions

1. **etcd 集群地址如何配置？** - 建议环境变量 `ETCD_ENDPOINTS`
2. **Node BullMQ 服务的 gRPC 端口？** - 需要与 Node 侧协调
3. **Outbox 重试策略？** - 建议指数退避，最大重试 10 次
4. **Op payload 存 Redis 还是只存 ID？** - 建议只存 ID，payload 从 DB 取（节省内存）
