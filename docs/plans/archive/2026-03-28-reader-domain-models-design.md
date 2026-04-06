# Reader Domain Models Design (Minimal Reading Core)

**Status:** Approved
**Date:** 2026-03-28
**Project:** Paper Immersive Reader

**Scope:** Milestone M1.1 — Minimal reading domain types in `crates/core`

**Goal:** Define the smallest stable, persistence-agnostic domain model needed to support the Phase 0/Phase 1 reading loop: paper identity and lifecycle, ordered reading blocks, and persisted reading position.

**Non-goals (explicit):**
- No persistence layer (IndexedDB/Dexie or any DB abstraction)
- No PDF parsing, segmentation, OCR, or import pipeline logic
- No annotation anchoring, tags, vocabulary, AI records, sync metadata
- No commitment to a serialized schema (JSON/protobuf) beyond what tests require

---

## 1. Design principles

1. **Local-first & offline-friendly**: domain types must not assume network availability.
2. **Readability-first**: keep types easy to reason about; avoid premature complexity.
3. **Low-regret foundations**: define types that remain valid as parsing/chunking improves.
4. **Opaque identifiers**: do not spread raw `String` IDs everywhere.

---

## 2. Modules and file layout

Target layout in `crates/core`:

- `crates/core/src/domain/mod.rs`
- `crates/core/src/domain/error.rs`
- `crates/core/src/domain/paper.rs`
- `crates/core/src/domain/block.rs`
- `crates/core/src/domain/reading_progress.rs`

`crates/core/src/lib.rs` should re-export the domain surface:

- `pub mod domain;`
- `pub use domain::{ ... }` (or re-export submodules) so downstream crates can import from a single place.

---

## 3. Types

### 3.1 Newtype IDs

- `PaperId(String)`
- `BlockId(String)`

Notes:
- IDs are opaque wrappers to prevent mixing IDs across entities.
- Generation strategy is intentionally not decided here (later: UUIDv7 or similar).

### 3.2 Enums

#### Paper lifecycle

```rust
enum PaperStatus {
    Created,
    Ingesting,
    ReadyText,
    ReadyOcr,
    Failed,
}
```

#### Paper import/source type

```rust
enum PaperSourceType {
    Pdf,
    Paste,
    Url,
    Doi,
    Zotero,
}
```

### 3.3 Structs

#### Paper

```rust
struct Paper {
    id: PaperId,
    title: Option<String>,
    source_type: PaperSourceType,
    source_ref: Option<String>,
    status: PaperStatus,
    parse_version: u32,
}
```

Notes:
- `title` optional because metadata extraction can be deferred.
- `parse_version` supports future re-segmentation and migration.

#### Block

```rust
struct Block {
    id: BlockId,
    paper_id: PaperId,
    order_index: u32,
    text: String,
    block_hash: Option<String>,
}
```

Notes:
- `order_index` defines immersive reading order.
- `block_hash` optional placeholder for future resilience mapping.

#### ReadingProgress

```rust
struct ReadingProgress {
    paper_id: PaperId,
    current_block_index: u32,
    current_block_id: Option<BlockId>,
    updated_at_ms: u64,
}
```

Notes:
- `progress_percent` is **derived** at higher layer from `blocks.len()`.
- Use millisecond epoch integer for now to avoid introducing a time dependency.

---

## 4. Invariants & validation

### 4.1 Minimal invariants

- `Block.text` must be non-empty after trimming.
- `ReadingProgress.current_block_index` must be within the block list bounds (cannot be validated here without blocks; validate at service layer).

### 4.2 Constructors

Expose constructors enforcing invariants (at least for `Block`):

- `Block::try_new(...) -> Result<Block, DomainError>`

Optionally do the same for `Paper` if we decide to enforce any invariants immediately.

### 4.3 Errors

Define a small domain error type:

- `DomainError` with variants like `EmptyBlockText`.

---

## 5. Testing strategy (TDD)

Integration tests under `tests/` (root-level, per AGENTS.md):

- `tests/domain_paper_smoke.rs`:
  - create a minimal `Paper` and assert fields
- `tests/domain_block_validation.rs`:
  - block with empty/whitespace text fails
  - block with non-empty text succeeds
- `tests/domain_progress_smoke.rs`:
  - create a progress value and assert fields

The tests should use the public exports of `paper_immersive_reader_core` (and avoid reaching into private modules).

---

## 6. Compatibility with existing docs

This design matches the PRD and BACKEND_STRUCTURE canonical entities by implementing the minimal subset:
- Paper
- Block
- ReadingProgress

Everything else remains explicitly deferred.
