## Context
OT WASM 服务当前以 REST + Socket.IO + gRPC 广播为主，文档 CRUD 与快照相关逻辑仅在 REST 端口实现，gRPC `Editable` service 仍缺失。需要在保持已有服务结构的前提下新增统一业务层，避免 gRPC 与 REST 行为分叉。

## Goals / Non-Goals
- Goals:
  - 统一文档与快照业务逻辑到 DocumentService
  - gRPC/REST controller 仅做参数解析与错误映射
  - 按 proto 完整实现 Editable RPC
  - 支持 snapshot_id 或 revision 回退
- Non-Goals:
  - 不调整 OT pipeline 或 socketio 变更流
  - 不引入新的传输协议

## Decisions
- Decision: 将文档 CRUD、快照列表、回退、签名等作为 DocumentService 的业务能力聚合，gRPC/REST 仅适配协议与请求/响应结构。
  - Alternatives considered: 为 gRPC 重新实现一套业务逻辑（拒绝，重复与行为不一致风险）。
- Decision: 复用现有 StorageService/SnapshotService 作为 DocumentService 的下游依赖，避免重复存储流程。
  - Alternatives considered: 在 controller 中直接调用 StorageService（拒绝，导致业务逻辑分散）。
- Decision: 回退支持 snapshot_id 或 revision；revision 情况下在 DocumentService 内解析为目标 snapshot。
  - Alternatives considered: 仅 snapshot_id（不满足需求）。
- Decision: 快照列表采用 cursor + limit + desc 的分页形态，保持与 proto 对齐。

## Risks / Trade-offs
- 引入新字段会影响迁移与历史数据兼容 → 采用可空字段与默认值保证回滚安全。
- 回退到 revision 可能需要跨多张表查询 → 用 snapshot index + version 索引保障性能。

## Migration Plan
1. 新增 document/ snapshot 字段与索引迁移
2. 实现 DocumentService API 并适配 REST
3. 实现 gRPC Editable 全量接口
4. 添加测试与验证

## Open Questions
- 需确认 revision 回退的优先级与冲突处理（snapshot_id 与 revision 同时出现时的策略）。
