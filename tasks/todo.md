# trx-ui Phase 3 — Task List

## Prerequisites
- [x] PR #54 review fixes pushed (d4f77c1)
- [ ] PR #54 merged to main

## Tasks

- [ ] **Task 1:** GPU syscall wrappers in `platform.zig` (~120 LOC)
  - Add SYS.GPU_* constants (0x0170-0x0179)
  - Add 8 high-level wrapper functions
  - Add 2 new tests
  - _Depends on: nothing_

- [ ] **Task 2:** `gpu.zig` — GPU context lifecycle (~250 LOC)
  - GpuContext struct with probe/alloc/free/map/submit/fence
  - 8-slot fixed buffer management, no heap
  - 5+ tests
  - _Depends on: Task 1_

--- CHECKPOINT: GPU context compiles, probe works on host ---

- [ ] **Task 3:** `atlas.zig` — Glyph atlas packing (~200 LOC)
  - 128x128 A8 texture, 95 ASCII glyphs
  - Rasterize from text.zig font_data
  - UV lookup table
  - 5+ tests
  - _Depends on: Task 2_ | **Can parallel with Task 4**

- [ ] **Task 4:** `batch.zig` — Command buffer batching (~250 LOC)
  - 3 command types (draw_rect, draw_glyph, set_clip), 16 bytes each
  - 4096-entry 64KB batch buffer
  - 8+ tests
  - _Depends on: Task 2_ | **Can parallel with Task 3**

--- CHECKPOINT: Pure data structures done, all host tests pass ---

- [ ] **Task 5:** GPU render path in `render.zig` (~180 LOC)
  - renderTreeGpu() parallel to existing renderTree()
  - No modification to existing CPU path
  - 4+ tests
  - _Depends on: Tasks 3 + 4_

- [ ] **Task 6:** App loop integration in `app.zig` (~200 LOC)
  - GPU probe at setup, fence sync in run loop
  - One-way CPU fallback on GPU error
  - Opt-in via caller-provided buffers
  - 3+ tests
  - _Depends on: Task 5_

--- CHECKPOINT: Feature complete, full test suite passes ---

- [ ] **Task 7:** Wire up root.zig + integration test (~50 LOC)
  - Re-exports for gpu, atlas, batch modules
  - Integration test: CPU vs GPU semantic equivalence
  - _Depends on: Task 6_

--- FINAL: All tests pass, zig build succeeds, ready for QEMU ---
