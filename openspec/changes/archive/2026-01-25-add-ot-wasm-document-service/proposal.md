## Why
现有 OT WASM 服务器只支持 BroadcastOp 的 gRPC 接口以及有限的 REST 文档接口，缺少按 `ot_rpc.proto` 定义的完整文档生命周期与快照/回退能力，导致 gRPC 与 REST 行为不一致、业务逻辑重复。

## What Changes
- 新增 `DocumentService` 作为业务层，统一文档增删改查、快照列表/回退、签名与映射查询能力
- 在 gRPC 与 REST controller 层复用业务逻辑，按 `Editable` service 的全部 RPC 接口实现
- 文档与快照新增字段（creator_id、doc_type、create_type、snapshot 名称/大小/users/restore_from）并补齐存储/时间信息
- 支持按 snapshot_id 或 revision 回退文档

## Impact
- Affected specs: ot-wasm
- Affected code:
  - packages/univer-ot-wasm/proto/ot_rpc.proto
  - packages/univer-ot-wasm/src/server/grpc.rs
  - packages/univer-ot-wasm/src/server/handlers/api.rs
  - packages/univer-ot-wasm/src/server/services/document.rs
  - packages/univer-ot-wasm/src/server/services/snapshot.rs
  - packages/univer-ot-wasm/src/server/services/storage.rs
  - packages/univer-ot-wasm/src/server/database/entities/*
  - packages/univer-ot-wasm/migration/*
