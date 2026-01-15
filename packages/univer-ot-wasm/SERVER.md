# OT Server

这是一个使用 Rust + Axum + Socket.IO + SeaORM + PostgreSQL 实现的 OT (Operational Transformation) 协同编辑服务器。

## 功能特性

- **Socket.IO 支持**：实时处理客户端 changeset，支持房间机制和 Ack 回调
- **OT 转换**：自动处理并发操作的冲突解决
- **文档快照**：定期保存文档完整状态
- **操作日志**：记录所有操作历史，支持版本追赶
- **REST API**：文档管理和查询接口
- **多文档订阅**：单个连接可订阅多个文档

## 环境要求

- Rust 1.70+
- PostgreSQL 12+
- 环境变量配置（见下方）

## 配置

**服务器支持通过环境变量或** `.env` 文件配置。优先顺序：环境变量 > `.env` 文件。

### 方式一：使用 .env 文件（推荐）

在项目根目录创建 `.env` 文件：

```bash
# 必需：PostgreSQL 连接字符串
DATABASE_URL=postgresql://user:password@localhost:5432/univer_ot

# 可选：服务器端口（默认 3000）
SERVER_PORT=3000

# 可选：快照更新间隔，每 N 个操作更新一次快照（默认 50）
SNAPSHOT_INTERVAL=50
```

### 方式二：使用环境变量

```bash
# 必需：PostgreSQL 连接字符串
export DATABASE_URL="postgresql://user:password@localhost:5432/univer_ot"

# 可选：服务器端口（默认 3000）
export SERVER_PORT=3000

# 可选：快照更新间隔，每 N 个操作更新一次快照（默认 50）
export SNAPSHOT_INTERVAL=50
```

## 构建和运行

### 构建服务器

```bash
# 使用 server feature 构建
cargo build --release --features server --bin ot-server

# 或者使用 pnpm（如果配置了脚本）
pnpm build:server
```

### 运行服务器

**使用 .env 文件（推荐）**：

```bash
# 创建 .env 文件（如果还没有）
# 然后直接运行
cargo run --release --features server --bin ot-server
```

**使用环境变量**：

```bash
# 设置环境变量
export DATABASE_URL="postgresql://user:password@localhost:5432/univer_ot"

# 运行服务器
cargo run --release --features server --bin ot-server
```

**注意**：如果 `.env` 文件不存在，服务器会忽略错误并继续运行，此时必须通过环境变量提供配置。

## API 文档

### Socket.IO API

**连接**: `http://localhost:3000` (默认命名空间 `/`)

服务器使用 Socket.IO 协议，支持房间机制和 Ack 回调。

#### 核心设计原则

1. **Ack/Broadcast 分离**：
   - 发送者通过 Socket.IO Ack 回调接收确认
   - 观察者通过房间广播接收转换后的变更（`changeset_pushed` 事件）
   - 发送者不会收到自己发送的消息

2. **房间机制**：
   - 每个文档对应一个房间：`doc:{doc_id}`
   - 客户端可以动态加入/退出多个文档房间

3. **版本管理**：
   - 所有消息包含版本信息（`baseRev`, `serverRev`）
   - 支持版本追赶（`fetch_ops`）

#### 客户端 -> 服务器事件

##### 1. `join_doc` - 加入文档房间

```typescript
socket.emit('join_doc', { docId: 'uuid' }, (ack) => {
  // ack: { status: 'ok', version: 10, content: {...} }
  // 或 { status: 'error', message: '...' }
})
```

**服务器处理**：
- 验证文档存在
- 将 socket 加入房间 `doc:{docId}`
- Ack 返回当前版本号和文档内容（快照）

##### 2. `leave_doc` - 离开文档房间

```typescript
socket.emit('leave_doc', { docId: 'uuid' })
```

**服务器处理**：
- 将 socket 从房间移除

##### 3. `changeset` - 发送变更集（使用 Ack）

```typescript
socket.emit('changeset', {
  docId: 'uuid',
  baseRev: 10,           // 必须：客户端当前版本
  clientMsgId: 'unique', // 必须：用于幂等性
  mutations: [...],      // 变更列表
  userId: 'user-123'     // 操作者 ID
}, (ack) => {
  // ack: { status: 'ok', serverRev: 11 }
  // 或 { status: 'error', message: '...' }
})
```

**服务器处理**：
- 验证 `baseRev` 是否匹配
- 检查 `clientMsgId` 幂等性
- 执行 OT 转换
- **Ack 返回**：`{ status: 'ok', serverRev: new_rev }` 或错误
- **房间广播**：`changeset_pushed` 事件（不发给发送者）

##### 4. `fetch_ops` - 获取操作历史（版本追赶）

```typescript
socket.emit('fetch_ops', {
  docId: 'uuid',
  startRev: 10  // 从哪个版本开始获取
}, (ack) => {
  // ack: { status: 'ok', operations: [...] }
  // operations: [{ rev, userId, mutations }, ...]
})
```

**服务器处理**：
- 查询 `startRev` 之后的所有操作
- 返回操作列表（按 rev 分组）

##### 5. `presence_update` - 光标/选择状态更新（可选）

```typescript
socket.emit('presence_update', {
  docId: 'uuid',
  cursor: { row: 1, col: 2 },
  selection: {...}
})
```

**服务器处理**：
- 广播给房间内其他客户端（高频，非关键）

#### 服务器 -> 客户端事件（房间广播）

##### 1. `changeset_pushed` - 变更推送（仅观察者）

```typescript
socket.on('changeset_pushed', ({
  docId,
  serverRev,    // 全局版本号
  userId,       // 操作者
  mutations     // 转换后的变更
}) => {
  // 更新本地视图
})
```

**特点**：
- 只发送给房间内**除发送者外**的其他客户端
- 使用 `socket.to(room).emit()` 实现

##### 2. `presence_update` - 光标/选择状态更新

```typescript
socket.on('presence_update', ({ docId, cursor, selection }) => {
  // 更新其他用户的光标位置
})
```

#### 数据流示例

```
客户端 A (发送者)          服务器              客户端 B (观察者)
     |                     |                      |
     |-- join_doc -------->|                      |
     |<-- Ack (v=10) ------|                      |
     |                     |                      |
     |-- join_doc ------------------------------>|
     |                     |<-- Ack (v=10) -------|
     |                     |                      |
     |-- changeset ------->|                      |
     |   (baseRev=10)      |                      |
     |                     |-- OT Transform       |
     |                     |-- Store to DB        |
     |<-- Ack (v=11) ------|                      |
     |                     |                      |
     |                     |-- changeset_pushed ->|
     |                     |   (to room,          |
     |                     |    exclude sender)   |
     |                     |                      |
     |                     |                      |<-- changeset_pushed
     |                     |                      |   (serverRev=11)
```

#### 客户端集成示例

```typescript
import { io } from 'socket.io-client';

const socket = io('http://localhost:3000');

// 1. 加入文档（获取当前版本和内容）
socket.emit('join_doc', { docId: 'doc-uuid-1' }, (ack) => {
  if (ack.status === 'ok') {
    console.log(`Joined doc, version: ${ack.version}`);
    // 加载内容: ack.content
  }
});

// 2. 发送 changeset（使用 Ack 接收确认）
socket.emit('changeset', {
  docId: 'doc-uuid-1',
  baseRev: 10,
  clientMsgId: 'client-msg-123',
  mutations: [...],
  userId: 'user-123'
}, (ack) => {
  if (ack.status === 'ok') {
    console.log(`Changeset applied, new version: ${ack.serverRev}`);
    // 更新本地版本号
  } else {
    console.error(`Error: ${ack.message}`);
  }
});

// 3. 接收其他客户端的变更（仅观察者）
socket.on('changeset_pushed', ({ docId, serverRev, userId, mutations }) => {
  console.log(`Doc ${docId} updated to rev ${serverRev} by ${userId}`);
  // 应用 mutations 到本地视图
});

// 4. 版本追赶（断线重连）
socket.emit('fetch_ops', {
  docId: 'doc-uuid-1',
  startRev: 10
}, (ack) => {
  if (ack.status === 'ok') {
    // 批量应用 ack.operations
    ack.operations.forEach(op => {
      // 应用操作
    });
  }
});

// 5. 离开文档
socket.emit('leave_doc', { docId: 'doc-uuid-1' });
```

### REST API

#### 创建文档

```bash
POST /api/documents
Content-Type: application/json

{
  "doc_id": "550e8400-e29b-41d4-a716-446655440000",
  "content": { ... }
}
```

#### 获取文档

```bash
GET /api/documents/:doc_id
```

#### 更新快照

```bash
POST /api/documents/:doc_id/snapshot
Content-Type: application/json

{
  "content": { ... },
  "version": 123
}
```

#### 获取操作日志

```bash
GET /api/documents/:doc_id/operations?from_rev=0&to_rev=100
```

#### 健康检查

```bash
GET /health
```

## 数据库结构

### document_snapshots 表

- `id` (UUID): 文档唯一标识
- `content` (JSONB): 文档完整内容
- `version` (BIGINT): 当前版本号
- `created_at` (TIMESTAMPTZ): 创建时间
- `updated_at` (TIMESTAMPTZ): 更新时间

### operation_logs 表

- `id` (BIGSERIAL): 自增主键
- `doc_id` (UUID): 文档 ID（外键）
- `rev` (BIGINT): 全局版本号（严格递增）
- `user_id` (VARCHAR): 操作者 ID
- `mutation_id` (VARCHAR): 操作类型
- `params` (JSONB): 操作参数
- `client_msg_id` (VARCHAR): 客户端消息 ID（用于幂等性）
- `created_at` (TIMESTAMPTZ): 创建时间

索引：
- `(doc_id, rev)` - 唯一索引
- `(doc_id, client_msg_id)` - 唯一索引（幂等性）
- `(doc_id)` - 普通索引

## 开发

### Migration 管理

Migration 使用独立的 `migration` crate 管理，遵循 SeaORM 2.0 的最佳实践。

#### 创建新的 Migration

```bash
# 在 migration 目录下创建新的 migration
cd migration
sea-orm-cli migrate generate <migration_name>
```

#### 运行 Migration

**方式一：通过服务器自动运行（推荐）**

服务器启动时会自动运行所有未应用的 migration：

```bash
cargo run --release --features server --bin ot-server
```

**方式二：通过 CLI 手动运行**

```bash
# 在 migration 目录下运行
cd migration
sea-orm-cli migrate up

# 或者回滚
sea-orm-cli migrate down
```

#### Migration 目录结构

```
migration/
├── Cargo.toml          # Migration crate 配置
├── README.md           # Migration 说明
└── src/
    ├── lib.rs          # Migrator API（用于集成到应用）
    ├── main.rs         # Migrator CLI（用于手动运行）
    └── m20250101_000001_create_tables.rs  # Migration 文件
```

### 测试

```bash
# 运行单元测试
cargo test --features server

# 运行集成测试（需要配置测试数据库）
cargo test --features server --test integration
```

## 注意事项

1. **版本控制**：服务器使用数据库事务确保版本号严格递增
2. **幂等性**：通过 `client_msg_id` 防止重复应用操作
3. **快照策略**：默认每 50 个操作更新一次快照，可通过环境变量配置
4. **并发安全**：使用数据库锁（SELECT FOR UPDATE）确保版本一致性

## 故障排查

### 数据库连接失败

检查 `DATABASE_URL` 环境变量是否正确设置，以及 PostgreSQL 服务是否运行。

### 版本冲突错误

如果客户端收到 "Version mismatch" 错误，说明客户端的 baseRev 已过期，需要重新同步文档状态。

### Socket.IO 连接失败

检查防火墙设置和服务器端口是否正确监听。确保客户端使用 Socket.IO 客户端库（如 `socket.io-client`）连接。
