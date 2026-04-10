/// App lifecycle — init, run (event loop), shutdown.
///
/// Supports optional GPU rendering: caller provides static buffers
/// for the glyph atlas and command batch. If GPU is unavailable or
/// buffers are not provided, falls back to CPU software rendering.
const node_mod = @import("node.zig");
const layout_mod = @import("layout.zig");
const render_mod = @import("render.zig");
const damage_mod = @import("damage.zig");
const input_mod = @import("input.zig");
const platform = @import("platform.zig");
const color_mod = @import("color.zig");
const gpu_mod = @import("gpu.zig");
const atlas_mod = @import("atlas.zig");
const batch_mod = @import("batch.zig");

pub const Node = node_mod.Node;
pub const Color = color_mod.Color;
pub const Framebuffer = render_mod.Framebuffer;
pub const InputEvent = input_mod.InputEvent;
pub const EventType = input_mod.EventType;
pub const FocusState = input_mod.FocusState;
pub const DamageTracker = damage_mod.DamageTracker;

/// Builder function type — called each frame to produce the UI tree.
pub const BuildFn = *const fn () *Node;

/// Optional GPU configuration — caller provides static buffers.
pub const GpuConfig = struct {
    atlas_pixels: *[atlas_mod.ATLAS_SIZE]u8,
    batch_buf: *[batch_mod.BATCH_BUF_SIZE]u8,
};

/// The main application handle.
pub const App = struct {
    title: []const u8,
    width: u32,
    height: u32,
    builder: BuildFn,
    running: bool = false,
    focus: FocusState = .{},
    damage: DamageTracker = undefined,

    // Platform handles (real TRX only)
    compositor_handle: i64 = -1,
    surface_handle: i64 = -1,
    buffer_handle: i64 = -1,
    mapped_buffer: ?[*]u32 = null,

    // GPU state (Phase 3)
    gpu_ctx: gpu_mod.GpuContext = .{},
    gpu_atlas: ?atlas_mod.GlyphAtlas = null,
    gpu_batch: ?batch_mod.Batch = null,
    use_gpu: bool = false,
    gpu_config: ?GpuConfig = null,

    pub fn init(title: []const u8, width: u32, height: u32, builder: BuildFn) App {
        return .{
            .title = title,
            .width = width,
            .height = height,
            .builder = builder,
            .damage = DamageTracker.init(width, height),
        };
    }

    /// Initialize with optional GPU rendering support.
    pub fn initWithGpu(title: []const u8, width: u32, height: u32, builder: BuildFn, gpu_config: GpuConfig) App {
        var app = init(title, width, height, builder);
        app.gpu_config = gpu_config;
        return app;
    }

    /// Initialise platform resources (compositor, surface, buffer).
    /// No-op on non-TRX targets. Probes GPU if GpuConfig was provided.
    pub fn setup(self: *App) !void {
        if (!platform.is_trx) return;

        self.compositor_handle = try platform.compositorCreate();
        self.surface_handle = try platform.surfaceCreate(
            self.compositor_handle,
            self.width,
            self.height,
        );

        const buf_size = @as(u64, self.width) * @as(u64, self.height) * 4;
        self.buffer_handle = try platform.bufferCreate(buf_size);
        self.mapped_buffer = try platform.bufferMap(self.buffer_handle);

        // Probe GPU if caller provided buffers
        if (self.gpu_config) |cfg| {
            self.gpu_ctx = gpu_mod.GpuContext.probe();
            if (self.gpu_ctx.isAvailable()) {
                var atlas_inst = atlas_mod.GlyphAtlas.init(cfg.atlas_pixels);
                atlas_inst.rasterize();
                atlas_inst.uploadToGpu(&self.gpu_ctx) catch {
                    // GPU atlas upload failed — fall back to CPU
                    self.gpu_ctx.deinit();
                    return;
                };
                self.gpu_atlas = atlas_inst;
                self.gpu_batch = batch_mod.Batch.init(cfg.batch_buf);
                self.use_gpu = true;
            }
        }
    }

    /// Run the main event loop.  Blocks until `self.running` is set to false.
    ///
    /// Each frame:
    /// 1. Poll input events
    /// 2. Call builder to get node tree
    /// 3. Compute layout
    /// 4. Render to buffer
    /// 5. Present to compositor
    pub fn run(self: *App) void {
        self.running = true;
        self.damage.damageAll(); // first frame = full redraw

        while (self.running) {
            // 1. Poll input
            self.pollInput();

            // 2. Build tree
            const root = self.builder();

            // 3. Layout
            const w_f: f32 = @floatFromInt(self.width);
            const h_f: f32 = @floatFromInt(self.height);
            layout_mod.computeLayout(root, w_f, h_f);

            // 4. Render (only if damaged)
            if (self.damage.isDirty()) {
                if (self.use_gpu) {
                    self.renderFrameGpu(root);
                } else if (self.mapped_buffer) |buf| {
                    const pixel_count = @as(usize, self.width) * @as(usize, self.height);
                    var fb = Framebuffer.init(buf[0..pixel_count], self.width, self.height);
                    fb.clear(Color.BG_PRIMARY);
                    render_mod.renderTree(&fb, root);

                    // Present (CPU path)
                    if (platform.is_trx) {
                        platform.compositorPresent(
                            self.compositor_handle,
                            self.surface_handle,
                            self.buffer_handle,
                        ) catch {};
                    }
                }
                self.damage.reset();
            }

            // Yield CPU (when not GPU — GPU path uses fence sync)
            if (!self.use_gpu and platform.is_trx) {
                platform.yield();
            }
        }
    }

    /// Stop the event loop.
    pub fn stop(self: *App) void {
        self.running = false;
    }

    /// Clean up platform resources.
    pub fn shutdown(self: *App) void {
        if (self.use_gpu) {
            self.gpu_ctx.deinit();
            self.use_gpu = false;
        }
        if (!platform.is_trx) return;
        if (self.buffer_handle >= 0) {
            platform.bufferUnmap(self.buffer_handle);
        }
        if (self.surface_handle >= 0) {
            platform.surfaceDestroy(self.surface_handle);
        }
    }

    /// Render one frame via the GPU command-buffer path.
    /// On error, falls back to CPU permanently.
    fn renderFrameGpu(self: *App, root: *Node) void {
        var b = &(self.gpu_batch orelse return);
        b.reset();

        if (self.gpu_atlas) |*a| {
            var gpu_state = render_mod.GpuRenderState{
                .batch = b,
                .atlas = a,
                .width = self.width,
                .height = self.height,
            };
            render_mod.renderTreeGpu(&gpu_state, root);
        }

        const cmd_slice = b.commandSlice();
        if (cmd_slice.len == 0) return;

        const fence = self.gpu_ctx.submit(cmd_slice) catch {
            self.use_gpu = false; // one-way fallback
            return;
        };
        self.gpu_ctx.waitFence(fence) catch {
            self.use_gpu = false; // one-way fallback
            return;
        };
    }

    /// Run one frame without blocking.  Useful for testing.
    pub fn runOneFrame(self: *App) *Node {
        const root = self.builder();
        const w_f: f32 = @floatFromInt(self.width);
        const h_f: f32 = @floatFromInt(self.height);
        layout_mod.computeLayout(root, w_f, h_f);
        return root;
    }

    fn pollInput(self: *App) void {
        if (!platform.is_trx) return;

        var raw_events: [32]platform.InputEventRaw = undefined;
        const count = platform.inputReadEvents(&raw_events, 32);
        if (count <= 0) return;

        const n: usize = @intCast(count);
        for (0..n) |i| {
            const raw = raw_events[i];
            const evt = InputEvent{
                .timestamp_ns = raw.timestamp_ns,
                .event_type = @enumFromInt(raw.type_),
                .code = raw.code,
                .value = raw.value,
                .device_id = raw.device_id,
            };
            self.handleEvent(evt);
        }
    }

    fn handleEvent(self: *App, evt: InputEvent) void {
        _ = self;
        _ = evt;
        // Event routing will be expanded in Phase 2.
        // For now, the app builder is responsible for checking state.
    }
};

// ── Tests ──────────────────────────────────────────────────────────

const testing = @import("std").testing;

var test_node = Node{
    .tag = .box,
    .style = .{
        .width = .{ .px = 100 },
        .height = .{ .px = 100 },
        .bg = Color.ACCENT_PURPLE,
    },
};

fn testBuilder() *Node {
    return &test_node;
}

test "App.init sets dimensions" {
    const app = App.init("Test", 800, 600, &testBuilder);
    try testing.expectEqual(@as(u32, 800), app.width);
    try testing.expectEqual(@as(u32, 600), app.height);
    try testing.expect(!app.running);
}

test "App.runOneFrame computes layout" {
    var app = App.init("Test", 800, 600, &testBuilder);
    const root = app.runOneFrame();
    try testing.expectEqual(@as(f32, 100), root.layout.width);
    try testing.expectEqual(@as(f32, 100), root.layout.height);
}

test "App.stop sets running false" {
    var app = App.init("Test", 800, 600, &testBuilder);
    app.running = true;
    app.stop();
    try testing.expect(!app.running);
}

test "App.init without GPU config has use_gpu false" {
    const app = App.init("Test", 800, 600, &testBuilder);
    try testing.expect(!app.use_gpu);
    try testing.expect(app.gpu_config == null);
}

test "App.initWithGpu stores GPU config" {
    var atlas_buf: [atlas_mod.ATLAS_SIZE]u8 = [_]u8{0} ** atlas_mod.ATLAS_SIZE;
    var batch_buf: [batch_mod.BATCH_BUF_SIZE]u8 = [_]u8{0} ** batch_mod.BATCH_BUF_SIZE;
    const app = App.initWithGpu("Test", 800, 600, &testBuilder, .{
        .atlas_pixels = &atlas_buf,
        .batch_buf = &batch_buf,
    });
    try testing.expect(app.gpu_config != null);
    try testing.expect(!app.use_gpu); // GPU not probed until setup()
}

test "App.setup on non-trx does not enable GPU" {
    var atlas_buf: [atlas_mod.ATLAS_SIZE]u8 = [_]u8{0} ** atlas_mod.ATLAS_SIZE;
    var batch_buf: [batch_mod.BATCH_BUF_SIZE]u8 = [_]u8{0} ** batch_mod.BATCH_BUF_SIZE;
    var app = App.initWithGpu("Test", 800, 600, &testBuilder, .{
        .atlas_pixels = &atlas_buf,
        .batch_buf = &batch_buf,
    });
    try app.setup(); // no-op on host
    try testing.expect(!app.use_gpu); // GPU not available on host
}
