# Paper Immersive Reader - 优化计划 v2

> 基于 2026-08-08 产品验收审查报告，结合代码实际审查结果制定。
> 原 OPTIMIZATION_PLAN.md 中 Task 1-10、15-18 已完成，本次聚焦未完成项 + 审查新发现问题 + 后续阶段规划。
> 优先级: **P0** (阻断/必须) → **P1** (应该修复) → **P2** (体验改进) → **P3** (文档/一致性)
>
> **更新 2026-08-09**: 批次一、二、三全部完成。批次二额外修复: InMemoryStore::save_ai_record upsert,
> GatewayConfig::require_ai_key 移除 gateway_token 回退, GatewayAiClient HTTP 错误状态处理,
> util::current_time_ms wasm32 修复 (js_sys::Date::now), 消除重复 compute_anchors 和 current_time_ms。
> 批次四浏览器端冒烟测试: gateway + web-app 启动验证通过, 手动交互测试待执行。

---

## 批次一: 文档对齐 (P1)

> 原 OPTIMIZATION_PLAN Task 11-14，4 个文档全部未执行，是当前最大的未完成块。

### Task 1 - 重写 TECH_STACK.md

**问题**: 文档描述 Next.js/React/Tailwind/Fastify/Dexie/Vitest 栈，实际为 Rust/Leptos/Axum/web_sys/cargo test。

**涉及文件**: `TECH_STACK.md`

**实现要点**: 逐章重写，保留文档结构（§0-§13），替换内容:

1. §2 前端栈:
   - 框架: Leptos 0.7 (CSR mode) + wasm-bindgen + web_sys
   - UI: 手写 CSS（`crates/web-app/styles.css`），CSS 变量主题
   - 状态管理: `AppState` struct + Leptos `RwSignal<T>` + context API
   - 路由: `leptos_router` 0.7，client-side routing
2. §3 本地持久化:
   - IndexedDB via `web_sys::IdbDatabase`（非 Dexie）
   - `IndexedDbStore` (wasm32) + `InMemoryStore` (native/test)
   - DB schema v2，8 个 object store: papers, blocks, progress, annotations, vocabulary, ai_records, settings, reading_stats
3. §4 PDF 解析:
   - vendored pdf.js via `wasm-bindgen` extern (`pdfjs_extract_text` JS 全局函数)
   - `WasmPdfBridge` (wasm32) / `StubPdfBridge` (native test)
   - `filter_repeated_lines` 页眉页脚检测 + `segment_text` 分段
4. §5 标注锚定:
   - 混合锚定: quote + prefix/suffix (50 chars) + start/end offsets + `AnchoringStatus`
   - `find_offsets` 4 级 fallback: 精确匹配 → 前 20 字符前缀 → 首词 → None (Unanchored)
5. §6 AI 集成:
   - Gateway: Axum HTTP server，`POST /api/translate` + `POST /api/explain`
   - `OpenAiClient` 调用 OpenAI-compatible chat completions API
   - `StubAiClient` 用于开发/测试
   - 浏览器侧: `GatewayAiClient` via `gloo-net`，携带可选 Bearer token
6. §8 后端栈:
   - Axum 0.8 + tokio + reqwest
   - `GatewayConfig::from_env()` 读取环境变量
   - token middleware: `GATEWAY_TOKEN` 可选，设置后校验 `Authorization: Bearer`
   - `GET /health` 端点
7. §9 打包部署:
   - Trunk (`trunk serve crates/web-app/index.html`)
   - `cargo build --target wasm32-unknown-unknown --no-default-features`
   - Gateway: `cargo run -p paper-immersive-reader-gateway`
8. §10 测试:
   - `cargo test --workspace` (102 tests)
   - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
   - `cargo fmt --check`
   - WASM build check: `cargo build -p paper-immersive-reader-web-app --target wasm32-unknown-unknown --no-default-features`
9. §13 最终栈总结: 更新为实际 Rust 全栈

**验收**: 文档中不再出现 Next.js/React/Fastify/Dexie/Vitest 等无关技术名称

---

### Task 2 - 更新 BACKEND_STRUCTURE.md

**问题**: §8.5 AI 端点定义为 `/ai/explain-term`、`/ai/explain-sentence` 等，实际为统一 `/api/explain` + `/api/translate`。§8.1 `GET /health` 已实现但文档未标注。

**涉及文件**: `BACKEND_STRUCTURE.md`

**实现要点**:

1. §1.1 系统拓扑: 将 "Web App (PWA)" 改为 "Web App (Leptos CSR)"，描述更新为 Rust/WASM
2. §1.1 Gateway: 改为 "Axum HTTP gateway"
3. §8.1 Health: 标注 `GET /health` ✅ 已实现
4. §8.5 AI 端点改为:
   ```
   POST /api/translate  (body: text, target_language, context?)
   POST /api/explain    (body: text, explanation_type: "term"|"sentence", language, context?)
   ```
   删除 `/ai/explain-term`、`/ai/explain-sentence`、`/ai/summary`、`/ai/structure`、`/ai/qa`
5. §8.2-8.4 (Sync/Assets/Zotero): 标注 "Phase 5+ 未实现"
6. §8.6 OCR: 标注 "Phase 4+ 未实现"
7. §3.2 Authentication: 更新为实际 token middleware 实现 (`require_token` axum middleware)
8. §9.2 Secrets: 标注 "未实现，Phase 5+"

**验收**: API 端点与 `crates/gateway/src/routes/` 代码完全一致

---

### Task 3 - 更新 IMPLEMENTATION_PLAN.md 验收清单

**问题**: §6 验收清单全部为 `[ ]`，即使 MVP 已实现。

**涉及文件**: `IMPLEMENTATION_PLAN.md`

**实现要点**:

§6.1 Import & Parsing:
- [x] local PDF import works
- [x] paste text import works
- [ ] URL/DOI import (P1, 未实现)
- [x] main text extraction exists (pdf.js)
- [x] header/footer cleanup exists (`filter_repeated_lines`)
- [x] segmentation exists (`segment_text`)
- [ ] OCR exists (Phase 4, CLI 已实现但 web 端未接入)

§6.2 Reading:
- [x] dedicated reading page
- [x] fullscreen mode
- [x] progress display
- [x] next block
- [ ] adaptive chunking (FR-12, 当前为规则分段，AI chunking 未实现)

§6.3 Translation & AI:
- [x] selection translation
- [x] paragraph translation ("Translate Block" 按钮)
- [x] bilingual display (flex row, 左右并排)
- [x] term explanation
- [x] explanation language switching (Settings)
- [x] sentence explanation
- [ ] summary / structure / Q&A / notes / analysis (P1, Phase 3)

§6.4 Annotation & Vocabulary:
- [x] highlight / underline / note / tag
- [x] default tags (重点/疑问/术语/结论/复习)
- [x] custom tags (Settings)
- [x] auto vocabulary collection
- [x] vocabulary management (搜索/过滤/标记/笔记)
- [x] restore last location
- [x] reading time statistics (ReadingStats)

§6.5 Export / Sync / Zotero:
- [ ] export (Phase 2+)
- [ ] S3 sync (Phase 5)
- [ ] Zotero sync (Phase 6)

**验收**: 勾选状态与代码实际功能一致

---

### Task 4 - 更新 FRONTEND_GUIDELINES.md

**问题**: 文档描述 React/Next.js 组件结构和命名规范，实际为 Leptos。

**涉及文件**: `FRONTEND_GUIDELINES.md`

**实现要点**:

1. §4.1 组件目录结构: 更新为实际结构:
   ```
   crates/web-app/src/
   ├── components/    (BlockContent, BlockNavigator, ProgressIndicator,
   │                   SelectionPopover, TranslationPanel, AiPanel,
   │                   AnnotationRenderer)
   ├── routes/        (LibraryPage, ImportPage, ReadPage, VocabularyPage, SettingsPage)
   ├── selection/     (popover, TextSelection, SelectionAction, compute_anchors)
   ├── storage/       (AppStorage trait, InMemoryStore, IndexedDbStore, schema)
   ├── ai/            (WebAiClient, GatewayAiClient)
   ├── import/        (WebImportService, WasmPdfBridge, StubPdfBridge)
   ├── annotations/   (create_annotation, compute_anchors)
   ├── reading/       (NavigationAction, ReadingSession)
   └── state.rs       (AppState, use_app_state)
   ```
2. §4.2 命名: Leptos `#[component]` fn → `PascalCase`，信号 → `RwSignal<T>`，回调 → `Callback<T>`
3. §5 状态管理: `AppState` + Leptos context (`provide_context` / `expect_context`)，非 Zustand
4. §7 键盘支持: 添加实际实现的快捷键:
   - `ArrowRight` / `PageDown` / `Space` → 下一块
   - `ArrowLeft` / `PageUp` → 上一块
   - `F` → 切换全屏
   - 表单输入时不拦截
5. §3.2 双语布局: 标注已实现 (`.tl-result` flex row + `@media` 响应式)
6. §2.3 阅读页: 标注实际布局 (grid: 1fr 400px)，全屏模式 (`reading-fullscreen` class)

**验收**: 文档中不再出现 React/Next.js/Zustand 等无关名称

---

## 批次二: 代码健壮性修复 (P1)

> 验收审查中发现的代码质量问题，不影响当前功能但会在特定条件下触发。

### Task 5 - InMemoryStore::save_annotation 改为 upsert 语义

**问题**: `memory_store.rs:93` 中 `save_annotation` 直接 `push`，重复保存同一 ID 的标注会产生重复条目。IndexedDB 的 `put_with_key` 天然去重不受影响，但 InMemoryStore 与 IDB 行为不一致。

**涉及文件**: `crates/web-app/src/storage/memory_store.rs`

**实现要点**:
1. `save_annotation` 中先检查是否已存在相同 ID 的标注
2. 若存在: 替换（等价于 `update_annotation`）
3. 若不存在: `push`
```rust
fn save_annotation(&mut self, annotation: &Annotation) -> Result<(), StorageError> {
    let key = annotation.paper_id.0.clone();
    let list = self.annotations.entry(key).or_default();
    if let Some(existing) = list.iter_mut().find(|a| a.id == annotation.id) {
        *existing = annotation.clone();
    } else {
        list.push(annotation.clone());
    }
    Ok(())
}
```

**验收**:
- `cargo test --workspace` 全部通过
- 新增测试: 连续两次 `save_annotation` 同一 ID → `load_annotations` 返回 1 条而非 2 条

---

### Task 6 - InMemoryStore::upsert_vocabulary 统一使用 term_lower 字段

**问题**: `memory_store.rs:142` 用 `item.term.to_lowercase()` 做 key，而 `VocabularyItem` 已有 `term_lower` 字段。两者逻辑等价但不 DRY，且如果未来 `term_lower` 的计算方式改变（如 Unicode 归一化），InMemoryStore 会与 IndexedDB 不一致。

**涉及文件**: `crates/web-app/src/storage/memory_store.rs`

**实现要点**:
1. `upsert_vocabulary`: `let key = item.term_lower.clone();`（替代 `item.term.to_lowercase()`）
2. `update_vocabulary`: 同理改为 `item.term_lower.clone()`

**验收**: `cargo test --workspace` 全部通过

---

### Task 7 - 补充 gateway token middleware 测试

**问题**: gateway token 校验中间件 `require_token` 已实现但没有专门的集成测试，只在代码层面存在。当前 `gateway_translate_contract.rs` 和 `gateway_explain_contract.rs` 测试不涉及 token。

**涉及文件**: 新增 `tests/gateway_token_auth.rs`

**实现要点**:
1. 测试 1: `expected_token = None` → 请求无 Authorization header → 200 OK
2. 测试 2: `expected_token = Some("secret")` + 请求携带 `Authorization: Bearer secret` → 200 OK
3. 测试 3: `expected_token = Some("secret")` + 请求无 header → 401
4. 测试 4: `expected_token = Some("secret")` + 错误 token → 401
5. 使用 `StubAiClient` + `build_router` 构建测试 router
6. 使用 `tokio::test` + `reqwest` 或 `axum::TestServer`

**验收**: `cargo test --test gateway_token_auth` 4 个测试全部通过

---

### Task 8 - 补充 /health 端点测试

**问题**: `GET /health` 已实现但无测试覆盖。

**涉及文件**: 新增 `tests/gateway_health.rs`

**实现要点**:
1. 构建 router with `StubAiClient`
2. `GET /health` → 200 + `{"status":"ok"}`
3. 可与 Task 7 合并到同一测试文件

**验收**: `cargo test --test gateway_health` 通过

---

## 批次三: 功能增强 (P2)

> 非 MVP 阻塞项，但属于 PRD 已定义功能，提升产品完整度。

### Task 9 - 导出功能 (FR-35/FR-36)

**问题**: PRD §4.8 定义导出功能，当前完全缺失。PRD 明确: 默认包含原文+标注+摘要+词汇，默认排除翻译+术语解释+AI 问答日志。

**涉及文件**:
- `crates/web-app/src/routes/library.rs` - 添加导出按钮
- 新增 `crates/web-app/src/export/mod.rs` - 导出逻辑
- `crates/web-app/src/state.rs` - AppState 方法

**实现要点**:
1. 数据模型: 导出为 JSON 或 Markdown 格式（MVP 先做 JSON）
2. 导出内容:
   ```rust
   pub struct ExportBundle {
       pub paper: StoredPaper,
       pub blocks: Vec<Block>,
       pub annotations: Vec<Annotation>,
       pub vocabulary: Vec<VocabularyItem>,
       pub reading_stats: Option<ReadingStats>,
       // 不包含: translations, ai_records
   }
   ```
3. Library paper card 添加 "Export" 按钮
4. 点击后生成 JSON，通过 `web_sys::Blob` + `URL::create_object_url` 触发下载
5. 文件名: `{paper_title_or_untitled}_{date}.json`

**验收**:
- 导入论文 → 添加标注 → 查询词汇 → 导出 → 下载的 JSON 包含 paper/blocks/annotations/vocabulary
- JSON 中不包含 ai_records
- `cargo test --workspace` 通过

---

### Task 10 - 标注搜索/过滤 (FR-27)

**问题**: PRD §4.6 FR-27 要求标注搜索，当前阅读页只列出当前论文的标注，无法跨论文搜索/过滤。

**涉及文件**:
- 新增 `crates/web-app/src/routes/annotations.rs` - 标注浏览页
- `crates/web-app/src/app.rs` - 添加路由
- `crates/web-app/src/state.rs` - 添加 `all_annotations()` 方法

**实现要点**:
1. 新增 `/annotations` 路由页面
2. 遍历所有 papers，收集标注
3. 过滤器:
   - 按论文 (dropdown)
   - 按类型 (highlight/underline/note)
   - 按标签 (dropdown，含默认 + 自定义)
   - 搜索框 (搜索 quote/note 文本)
4. 列表展示: 标注引用文本 + 类型图标 + 标签 + 论文标题 + "跳转阅读" 链接
5. 导航栏添加 "Annotations" 入口（当前导航只有 Library/Import/Vocabulary/Settings）

**验收**:
- 创建多篇论文的标注 → /annotations 页面显示全部
- 按类型/标签/搜索词过滤 → 结果正确
- 点击"跳转阅读" → 导航到对应论文阅读页

---

### Task 11 - AI 面板可折叠

**问题**: FRONTEND_GUIDELINES §2.3 要求 "AI panel can be collapsed; collapsed state persists per user"。当前阅读页的侧边栏始终显示。

**涉及文件**:
- `crates/web-app/src/routes/read.rs` - 添加折叠状态
- `crates/web-app/src/storage/schema.rs` - Settings 添加 `sidebar_collapsed: bool`
- `crates/web-app/styles.css` - 折叠样式

**实现要点**:
1. `Settings` 添加 `sidebar_collapsed: bool`（默认 false）
2. 阅读页添加折叠/展开按钮
3. 折叠时侧边栏 `display: none`，阅读列扩展为全宽
4. 状态变更时保存到 Settings（持久化到 IndexedDB）

**验收**:
- 点击折叠 → 侧边栏隐藏，阅读区域扩展
- 刷新页面 → 折叠状态保持
- 点击展开 → 恢复双语布局

---

## 批次四: 浏览器端验证准备 (P2)

> 当前所有验证均在 cargo 层面完成，浏览器端从未实际走过完整流程。

### Task 12 - 浏览器端冒烟测试清单与修复

**问题**: prd.json US-005 中 5 个 `[PENDING]` 浏览器验证项从未执行。可能存在 WASM 编译通过但运行时失败的问题。

**涉及文件**: 无代码修改（验证任务），发现的问题另行建 Task

**实现要点**:
1. 启动 gateway: `GATEWAY_AI_USE_STUB=true ./scripts/run_gateway.sh`
2. 启动 web-app: `./scripts/run_web_app.sh`
3. 冒烟测试清单:
   - [ ] Library 页面正常渲染，空状态显示正确
   - [ ] Import 粘贴文本 → 跳转阅读页
   - [ ] Import PDF 上传 → 提取文本 → 跳转阅读页
   - [ ] 阅读: 下一块/上一块导航正常
   - [ ] 阅读: 进度条更新
   - [ ] 阅读: 键盘快捷键 (ArrowRight/Space/PageUp/F)
   - [ ] 选词翻译 → 翻译面板左右并排显示
   - [ ] 选词解释 (term/sentence) → AI 面板显示结果
   - [ ] 选词标注 → 创建标注 → 内联装饰显示
   - [ ] 标注删除 → 列表和内联装饰同步更新
   - [ ] Vocabulary 页面 → 查询的词出现 → 搜索/过滤正常
   - [ ] Settings 页面 → 修改语言/gateway URL/token → 保存
   - [ ] 刷新页面 → 数据从 IndexedDB 恢复
   - [ ] 全屏模式 → 导航栏隐藏 → 退出全屏恢复
   - [ ] Library 显示进度条 + 阅读时间
   - [ ] Library 删除论文 → 确认 → 论文消失

4. 记录所有失败项，按严重程度分类

**验收**: 清单全部勾选，或失败项已记录为新 Task

---

## 批次五: 后续阶段规划 (P3)

> PRD 中 Phase 2-6 的功能规划，非本次优化范围，仅记录作为路线图参考。

### Phase 2 - 阅读质量改进
- 连字符断词修复 (hyphenation cleanup)
- 多栏 PDF 检测与阅读顺序修正
- block 重新分段版本管理
- 导入诊断 UI (显示提取质量评分)

### Phase 3 - P1 知识功能
- 段落摘要 (FR-19)
- 全文摘要
- 结构提取: 目的/方法/结果/结论 (FR-20)
- 论文问答 (FR-21)
- 笔记生成 (FR-22)
- 关键词提取 (FR-34)
- 关键句标记 (FR-33)
- 标注按类别导出 (FR-28)

### Phase 4 - OCR + PDF 高级容错
- 扫描 PDF 检测
- OCR fallback pipeline (浏览器端 Tesseract.js 或 Gateway PaddleOCR)
- OCR 置信度阈值
- 大文档渐进式 OCR
- 未锚定标注的重新锚定 UI

### Phase 5 - S3 同步
- Settings 同步配置 UI (FR-38)
- 实体清单模型
- push/pull 同步流程
- 冲突处理
- 全量同步: papers/progress/annotations/vocabulary/AI records

### Phase 6 - Zotero 双向同步
- Zotero 账户连接
- 导入元数据 + PDF + 标注/笔记
- 推送标注/笔记回 Zotero
- 映射表维护

---

## 执行顺序与依赖关系

```
批次一 (P1 文档 - 全部可并行):
  Task 1 (TECH_STACK)       ── 独立
  Task 2 (BACKEND_STRUCT)   ── 独立
  Task 3 (IMPL_PLAN)        ── 独立
  Task 4 (FRONTEND_GUIDE)   ── 独立

批次二 (P1 健壮性 - 部分有依赖):
  Task 5 (ann upsert)       ── 独立
  Task 6 (vocab term_lower) ── 独立
  Task 7 (token test)       ── 独立
  Task 8 (health test)      ── 可与 Task 7 合并到同一文件

批次三 (P2 功能 - 有依赖):
  Task 9  (export)          ── 独立
  Task 10 (ann search)      ── 独立
  Task 11 (sidebar collapse)── 独立

批次四 (P2 验证):
  Task 12 (browser smoke)   ── 依赖批次一/二全部完成
```

## 每批次完成后的验证检查

每个 Task 完成后:
1. `cargo fmt --check`
2. `cargo clippy --workspace --all-targets --all-features -- -D warnings`
3. `cargo test --workspace`
4. `cargo build -p paper-immersive-reader-web-app --target wasm32-unknown-unknown --no-default-features`

批次二/三完成后:
5. 启动 gateway (`GATEWAY_AI_USE_STUB=true ./scripts/run_gateway.sh`)
6. 启动 web-app (`./scripts/run_web_app.sh`)
7. 浏览器端走查该批次相关功能

批次四完成后:
8. 完整浏览器端冒烟测试清单全部通过
