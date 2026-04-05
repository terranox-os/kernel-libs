/// App lifecycle — init, run (event loop), shutdown.

const node_mod = @import("node.zig");
const layout_mod = @import("layout.zig");
const render_mod = @import("render.zig");
const damage_mod = @import("damage.zig");
const input_mod = @import("input.zig");
const platform = @import("platform.zig");
const color_mod = @import("color.zig");

pub const Node = node_mod.Node;
pub const Color = color_mod.Color;
pub const Framebuffer = render_mod.Framebuffer;
pub const InputEvent = input_mod.InputEvent;
pub const EventType = input_mod.EventType;
pub const FocusState = input_mod.FocusState;
pub const DamageTracker = damage_mod.DamageTracker;

/// Builder function type — called each frame to produce the UI tree.
pub const BuildFn = *const fn () *Node;

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

    pub fn init(title: []const u8, width: u32, height: u32, builder: BuildFn) App {
        return .{
            .title = title,
            .width = width,
            .height = height,
            .builder = builder,
            .damage = DamageTracker.init(width, height),
        };
    }

    /// Initialise platform resources (compositor, surface, buffer).
    /// No-op on non-TRX targets.
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
                if (self.mapped_buffer) |buf| {
                    const pixel_count = @as(usize, self.width) * @as(usize, self.height);
                    var fb = Framebuffer.init(buf[0..pixel_count], self.width, self.height);
                    fb.clear(Color.BG_PRIMARY);
                    render_mod.renderTree(&fb, root);
                }
                self.damage.reset();

                // 5. Present
                if (platform.is_trx) {
                    platform.compositorPresent(
                        self.compositor_handle,
                        self.surface_handle,
                        self.buffer_handle,
                    ) catch {};
                }
            }

            // Yield CPU
            if (platform.is_trx) {
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
        if (!platform.is_trx) return;
        if (self.buffer_handle >= 0) {
            platform.bufferUnmap(self.buffer_handle);
        }
        if (self.surface_handle >= 0) {
            platform.surfaceDestroy(self.surface_handle);
        }
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
