---
active: true
iteration: 1
max_iterations: 10
completion_promise: "DONE"
started_at: "2026-01-30T07:07:09Z"
---

将 @packages/univer-ot-wasm/crates/ot-core/src/params.rs  中的类型改为 mutations 中的 params，移除掉旧的 params.rs，然后将所有 mutation 用字面量写死的mutation id改为对应的 mutation.id 属性，并保证 ot-core, ot-wasm, ot-server 包编译成功
