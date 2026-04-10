# trx-ui Phase 3: GPU Rendering Path

See full plan: `~/.claude/plans/fluttering-juggling-rabbit.md`

## Summary

Add an optional GPU rendering path to trx-ui. The CPU software renderer remains the default fallback. GPU path batches draw commands (rects, glyphs, clips) into a 64KB command buffer and submits once per frame via `gpu_submit`, with fence-based sync.

## Architecture

```
platform.zig → gpu.zig → atlas.zig (parallel) → render.zig → app.zig → root.zig
                        → batch.zig (parallel) ↗
```

## Scope

- 7 tasks, ~1,400 LOC (3 new files, 4 modified)
- ~25 new tests (host-testable, no QEMU required)
- Tasks 3 and 4 can run in parallel
- Checkpoints after tasks 2, 3+4, 5, and 7
