# Paper Immersive Reader — MVP 优化计划

> 基于 2026-08-08 产品验收审查报告，按优先级分批执行。
> 每个任务标注: **P0** (必须修复/阻塞) → **P1** (应该修复) → **P2** (体验改进) → **P3** (文档/一致性)

---

## 批次一: Bug 修复与功能补全 (P0)

### Task 1 — GatewayAiClient 请求携带 GATEWAY_TOKEN

**问题**: `GatewayAiClient::translate_async` / `explain_async` 不发送 `GATEWAY_TOKEN` header。若 gateway 配置了 token 校验，所有 AI 请求会被拒绝。同时 gateway 侧也没有实际校验 token 的中间件——两端都需要补。

**涉及文件**:
- `crates/web-app/src/ai/client.rs` — `GatewayAiClient` 请求添加 `Authorization` header
- `crates/web-app/src/storage/schema.rs` — `Settings.ai_gateway_token_ref` 已存在但未使用
- `crates/web-app/src/routes/settings.rs` — 添加 gateway token 输入框
- `crates/web-app/src/state.rs` — 从 settings 恢复 token 到运行时状态
- `crates/gateway/src/routes/mod.rs` — 添加 token 校验 axum middleware
- `crates/gateway/src/config.rs` — `gateway_token` 已存在，middleware 读取它

**实现要点**:
1. `GatewayAiClient::new` 增加 `token: Option<String>` 参数
2. `translate_async` / `explain_async` 的 `gloo_net::http::Request` 调用 `.header("Authorization", &format!("Bearer {}", token))` (当 token 存在时)
3. Settings 页面添加 "Gateway Token (optional)" 输入框，存入 `Settings.ai_gateway_token_ref`
4. `AppState` 添加 `gateway_token: RwSignal<Option<String>>`，`init_persistence` 时从 settings 恢复
5. Gateway 侧添加 axum middleware: 若 `GATEWAY_TOKEN` 环境变量存在，校验请求 `Authorization: Bearer <token>` header；不存在则放行
6. `/api/translate` 和 `/api/explain` 路由应用该 middleware

**验收**: 
- 设置 `GATEWAY_TOKEN=secret` 启动 gateway，web-app settings 填入 token → AI 请求成功
- 不填 token → gateway 返回 401
- 不设置 `GATEWAY_TOKEN` 环境变量 → 无需 token，直接放行

---

### Task 2 — IndexedDB vocabulary upsert 保留原始 term 大小写

**问题**: `indexed_db_store.rs:405` 中 `new_item.term = term_lower` 将新词强制小写存储，原始大小写丢失（如 "DNA" 存为 "dna"）。

**涉及文件**:
- `crates/web-app/src/storage/indexed_db_store.rs`

**实现要点**:
1. `upsert_vocabulary` 中新建词时保留 `item.term` 原值，不覆盖为 `term_lower`
2. `term_lower` 仅用于索引查找，不回写到 `term` 字段
3. 删除 `new_item.term = term_lower;` 这一行

**验收**:
- 查询 "DNA" → vocabulary 列表显示 "DNA" 而非 "dna"
- 再次查询 "dna" → lookup_count 递增，term 保持首次存储的 "DNA"

---

### Task 3 — 阅读时间统计 (FR-32)

**问题**: PRD 将阅读时间统计列为 P0 功能，代码中完全缺失。

**涉及文件**:
- `crates/core/src/domain/reading_progress.rs` — 扩展 `ReadingProgress` 或新增 `ReadingStats` 结构
- `crates/web-app/src/storage/schema.rs` — 新增 `ReadingStats` schema
- `crates/web-app/src/storage/mod.rs` — `AppStorage` trait 新增 `save_reading_stats` / `load_reading_stats`
- `crates/web-app/src/storage/memory_store.rs` — 实现
- `crates/web-app/src/storage/indexed_db_store.rs` — 实现 + 新增 object store
- `crates/web-app/src/state.rs` — 持久化方法
- `crates/web-app/src/routes/read.rs` — 记录会话开始时间，离开页面时计算 elapsed 并保存
- `crates/web-app/src/routes/library.rs` — paper card 显示总阅读时间

**实现要点**:
1. 数据模型:
   ```rust
   pub struct ReadingStats {
       pub paper_id: PaperId,
       pub total_reading_ms: u64,
       pub last_session_started_ms: Option<u64>,
       pub session_count: u32,
   }
   ```
2. `ReadPage` 挂载时记录 `session_start_ms`（存入 `ReadingStats.last_session_started_ms` 并持久化，防崩溃丢失）
3. `on_cleanup` (组件卸载) 时计算 `elapsed = now - session_start_ms`，累加到 `total_reading_ms`，`session_count += 1`，清空 `last_session_started_ms`
4. IndexedDB 新增 `reading_stats` object store (DB_VERSION 升至 2)
5. Library paper card 显示: `"阅读 2h 15m"`
6. 进度指示器旁可选显示当前会话时长

**验收**:
- 打开论文阅读 30 秒后离开 → 返回 library 看到 "阅读 30s"
- 再次打开阅读 30 秒 → "阅读 1m"
- 刷新页面不丢失（IndexedDB 持久化）

---

### Task 4 — 添加标注删除功能

**问题**: 可创建标注但无删除入口，用户无法修正误标注。

**涉及文件**:
- `crates/web-app/src/storage/mod.rs` — `AppStorage` trait 新增 `delete_annotation`
- `crates/web-app/src/storage/memory_store.rs` — 实现
- `crates/web-app/src/storage/indexed_db_store.rs` — 实现 (IDB `delete()`)
- `crates/web-app/src/state.rs` — `delete_annotation` 方法 + 持久化
- `crates/web-app/src/components/annotation_renderer.rs` — 每条标注添加删除按钮
- `crates/web-app/src/routes/read.rs` — 处理删除回调

**实现要点**:
1. `AppStorage::delete_annotation(&mut self, id: &AnnotationId) -> Result<(), StorageError>`
2. `InMemoryStore`: 从 `annotations` vec 中 retain 过滤
3. `IndexedDbStore`: `store.delete(&JsValue::from_str(&id.0))`
4. `AnnotationRenderer` 新增 `on_delete: Callback<AnnotationId>` prop，每条标注渲染删除按钮
5. `ReadPage` 传入 `on_delete` 回调，调用 `app.delete_annotation(&id)`

**验收**:
- 创建标注 → 出现删除按钮 → 点击删除 → 标注从列表和内联装饰中消失
- 刷新页面 → 标注确实已从 IndexedDB 删除

---

## 批次二: 健壮性与一致性修复 (P1)

### Task 5 — Gateway 添加 /health 端点

**问题**: `BACKEND_STRUCTURE.md §8.1` 定义了 `GET /health`，但 gateway 未实现。

**涉及文件**:
- `crates/gateway/src/routes/mod.rs` — 添加 `/health` 路由

**实现要点**:
1. 添加 `GET /health` 返回 `200 OK` + JSON `{"status": "ok"}`
2. 不需要 AI client state，直接挂在 Router 根上

**验收**: `curl http://127.0.0.1:3000/health` → `{"status":"ok"}`

---

### Task 6 — Library paper card 显示阅读进度

**问题**: `FRONTEND_GUIDELINES §2.1` 要求 paper card 显示阅读进度和最后打开时间，当前缺失。

**涉及文件**:
- `crates/web-app/src/routes/library.rs` — 查询 progress 并渲染

**实现要点**:
1. 遍历 papers 时调用 `app.progress(&sp.paper.id)` 获取 `ReadingProgress`
2. 计算 `progress_percent = (current_block_index + 1) * 100 / total_blocks`（需查 blocks 数量或从 ReadingProgress 扩展）
3. card 渲染: 进度条 + 百分比 + "最后阅读 2h ago" (用 `format_relative_time`)
4. 若无 progress 记录，显示 "未开始"

**验收**:
- 导入论文 → library 显示 "未开始"
- 阅读至第 5/20 块 → library 显示 "25%" 进度条
- 离开后返回 → 进度和时间正确

---

### Task 7 — 词汇表搜索/过滤

**问题**: `APP_FLOW §1.4` 和 `FR-30` 要求词汇管理 UI 含搜索，当前无搜索框。

**涉及文件**:
- `crates/web-app/src/routes/vocabulary.rs` — 添加搜索输入框 + 过滤逻辑

**实现要点**:
1. 添加 `search_query: RwSignal<String>` 信号
2. 搜索输入框 `on:input` 更新信号
3. `items()` 闭包中按 `term.contains(query)` (case-insensitive) 过滤
4. 可选: 按 learned/unlearned 状态过滤的下拉框

**验收**:
- 输入 "gen" → 只显示包含 "gen" 的词项
- 清空搜索框 → 恢复完整列表

---

### Task 8 — 翻译面板双语左右并排布局

**问题**: `FRONTEND_GUIDELINES §3.2` 要求"英文左，中文右"，当前为上下堆叠。

**涉及文件**:
- `crates/web-app/src/components/translation_panel.rs` — 调整 HTML 结构
- `crates/web-app/styles.css` — `.tl-result` 改为 flex row

**实现要点**:
1. `.tl-result` 改为 `display: flex; gap: 1rem`，两个子元素各占 50%
2. 左侧 `<div class="tl-source">` 标注 "English"，右侧 `<div class="tl-target">` 标注 "中文"
3. 响应式: `@media (max-width: 600px)` 时堆叠为 column
4. 翻译面板标题旁添加语言标签

**验收**:
- 选词翻译 → 英文在左，中文在右
- 缩小窗口至手机宽度 → 切换为上下排列

---

### Task 9 — 修正 IndexedDB Closure::forget 内存泄漏

**问题**: `indexed_db_store.rs` 中 `await_val` 和 `await_idb_db` 使用 `Closure::forget()`，每次 IDB 操作都会泄漏内存。

**涉及文件**:
- `crates/web-app/src/storage/indexed_db_store.rs`

**实现要点**:
1. 将 `await_val` 重构为不使用 `Closure::forget` 的模式
2. 方案: 用 `wasm_bindgen::closure::Closure::once_into_js` 创建一次性回调，或用 `RefCell` 持有 Closure 在 promise resolve 后 drop
3. `await_idb_db` 同理
4. `import.rs:160` 的 `onload.forget()` 也需修正（改用 `Closure::once_into_js` 或存入结构体）

**验收**:
- 连续执行 100 次 IDB 操作 → 浏览器内存无显著增长
- `cargo test --workspace` 仍全部通过

---

### Task 10 — 修正 find_offsets fallback 逻辑

**问题**: `read.rs:743` 中 `find_offsets` 在精确匹配失败时 fallback 到 `(0, text.chars().count())`，导致标注覆盖整个块。

**涉及文件**:
- `crates/web-app/src/routes/read.rs`

**实现要点**:
1. fallback 策略改为: 对 block_text 和 selected text 做 whitespace 归一化后再查找
2. 若仍找不到，用 selected text 的前 20 字符在 block_text 中搜索定位 start
3. 若全部失败，返回 `(0, 0)` 并标记 `AnchoringStatus::Unanchored` 而非覆盖整块

**验收**:
- 选中文本中含多余空格 → 标注仍能正确定位
- 选中跨节点文本 → 标注位置尽可能接近实际选择

---

## 批次三: 文档对齐 (P1)

### Task 11 — 更新 TECH_STACK.md 反映实际 Rust 架构

**问题**: 文档描述 Next.js/React/Fastify/Dexie，实际为 Rust/Leptos/Axum/web_sys。

**涉及文件**:
- `TECH_STACK.md`

**实现要点**:
重写以下章节:
1. §2 前端栈: Leptos 0.7 (CSR) + wasm-bindgen + web_sys + 手写 CSS
2. §3 本地持久化: IndexedDB via web_sys (非 Dexie)
3. §4 PDF 解析: vendored pdf.js via wasm-bindgen extern
4. §8 后端栈: Axum + reqwest (非 Fastify)
5. §9 打包部署: Trunk (非 Vercel/next-pwa)
6. §10 测试: cargo test + 集成测试 (非 Vitest/Playwright)
7. §13 最终推荐栈总结: 更新为实际栈

---

### Task 12 — 更新 BACKEND_STRUCTURE.md API 端点

**问题**: 文档定义 `/ai/explain-term`、`/ai/explain-sentence` 等端点，实际为统一 `/api/explain` + `/api/translate`。

**涉及文件**:
- `BACKEND_STRUCTURE.md`

**实现要点**:
1. §8.5 AI 端点改为: `POST /api/translate`, `POST /api/explain` (explanation_type 区分 term/sentence)
2. 添加 `GET /health` (Task 5 实现后)
3. §8.2-8.4 标注为 "Phase 5+ 未实现"
4. §1 系统拓扑更新为 Rust 实现

---

### Task 13 — 更新 IMPLEMENTATION_PLAN.md 验收清单

**问题**: §6 验收清单全部为 `[ ]`，即使 MVP 已实现。

**涉及文件**:
- `IMPLEMENTATION_PLAN.md`

**实现要点**:
1. §6.1 导入与解析: 勾选 PDF 导入、粘贴文本、文本提取、页眉页脚清理、分段
2. §6.2 阅读: 勾选全部 (专用页面、全屏、进度、下一块)
3. §6.3 翻译与AI: 勾选选词翻译、段落翻译、双语显示、术语解释、语言切换、句子解释
4. §6.4 标注与词汇: 勾选高亮/下划线/笔记/标签、默认标签、自定义标签、自动词汇收集、词汇管理、恢复位置
5. 未实现项保留 `[ ]`: OCR、阅读时间统计 (Task 3 完成后勾选)

---

### Task 14 — 更新 FRONTEND_GUIDELINES.md 反映 Leptos 架构

**问题**: 文档描述 React/Next.js 组件结构和命名规范，实际为 Leptos。

**涉及文件**:
- `FRONTEND_GUIDELINES.md`

**实现要点**:
1. §4 组件目录结构: 更新为实际 `components/`、`routes/`、`selection/`、`storage/` 模块结构
2. §4.2 命名: Leptos `#[component]` fn → snake_case，信号 → `RwSignal<T>`
3. §5 状态管理: `AppState` + Leptos context (非 Zustand)
4. §7 键盘支持: 添加实际实现的快捷键 (ArrowRight/PageDown/Space/ArrowLeft/PageUp/F)

---

## 批次四: 体验改进 (P2)

### Task 15 — Library paper card 添加删除论文功能

**涉及文件**:
- `crates/web-app/src/storage/mod.rs` — `AppStorage` 新增 `delete_paper` + 关联数据级联删除
- `crates/web-app/src/storage/memory_store.rs`
- `crates/web-app/src/storage/indexed_db_store.rs`
- `crates/web-app/src/state.rs`
- `crates/web-app/src/routes/library.rs` — 删除按钮 + 确认对话框

**实现要点**:
1. `delete_paper` 删除 paper + blocks + progress + annotations + ai_records
2. IndexedDB 需要在单个事务中删除多个 store 的数据
3. UI: paper card 添加删除按钮，点击弹出确认

---

### Task 16 — 标注编辑功能

**涉及文件**:
- `crates/web-app/src/components/annotation_renderer.rs` — 编辑按钮
- `crates/web-app/src/routes/read.rs` — 编辑表单复用 annotation form

**实现要点**:
1. 标注列表每项添加 "编辑" 按钮
2. 点击后打开 annotation form，预填现有 type/note/tag
3. 保存时 `save_annotation` 覆盖原记录 (相同 ID)

---

### Task 17 — 导入处理进度反馈

**问题**: `FRONTEND_GUIDELINES §2.2` 要求导入显示分阶段进度，当前仅显示 "Importing…"。

**涉及文件**:
- `crates/web-app/src/routes/import.rs`

**实现要点**:
1. 添加 `import_stage: RwSignal<&str>` 信号
2. PDF 导入流程更新阶段: "Extracting text..." → "Cleaning headers..." → "Segmenting..." → "Done"
3. paste 导入: "Segmenting..." → "Done"
4. UI 显示当前阶段文字 + 简单动画

---

### Task 18 — 词汇表跳回原文

**问题**: `APP_FLOW §2.8` 提及 "jump back to source context"，当前词汇表无此功能。

**涉及文件**:
- `crates/web-app/src/routes/vocabulary.rs`

**实现要点**:
1. 词汇项添加 "查看出处" 链接
2. 链接跳转到 `/read/{paper_id_first_seen}` 
3. 可选: 通过 URL query param 传递 block_id，ReadPage 读取后跳转到对应块

---

## 执行顺序与依赖关系

```
批次一 (P0 - 并行执行):
  Task 1 (gateway token)     ─┐
  Task 2 (vocab case fix)    ─┤
  Task 3 (reading time)      ─┤── 无相互依赖，可并行
  Task 4 (annotation delete) ─┘

批次二 (P1 - 部分有依赖):
  Task 5  (health endpoint)  ── 独立
  Task 6  (library progress) ── 依赖 Task 3 (显示阅读时间)
  Task 7  (vocab search)     ── 独立
  Task 8  (bilingual layout) ── 独立
  Task 9  (closure leak)     ── 独立
  Task 10 (offset fix)       ── 独立

批次三 (P1 文档 - 可并行):
  Task 11 (TECH_STACK)       ── 独立
  Task 12 (BACKEND_STRUCT)   ── 依赖 Task 5 (health endpoint)
  Task 13 (IMPL_PLAN)        ── 依赖 Task 3 (阅读时间)
  Task 14 (FRONTEND_GUIDE)   ── 独立

批次四 (P2 体验 - 按需执行):
  Task 15 (delete paper)     ── 独立
  Task 16 (edit annotation)  ── 依赖 Task 4 (annotation delete)
  Task 17 (import progress)  ── 独立
  Task 18 (vocab jump back)  ── 独立
```

## 每批次完成后的验证检查

每个 Task 完成后:
1. `cargo fmt --check`
2. `cargo clippy --workspace --all-targets --all-features -- -D warnings`
3. `cargo test --workspace`
4. `cargo build -p paper-immersive-reader-web-app --target wasm32-unknown-unknown --no-default-features`

每批次完成后:
5. 启动 gateway (`GATEWAY_AI_USE_STUB=true ./scripts/run_gateway.sh`)
6. 启动 web-app (`./scripts/run_web_app.sh`)
7. 浏览器端走查该批次相关功能
