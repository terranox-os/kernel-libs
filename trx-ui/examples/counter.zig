// Hand-crafted counter example demonstrating the trx-ui API.
//
// Build:   zig build example
// (or)     zig build-exe examples/counter.zig -Mtrx_ui=src/root.zig

const ui = @import("trx_ui");

// ── State ───────────────────────────────────────────────────

const State = struct {
    count: i32 = 0,
};

var state: State = State{};

// ── Event handlers ──────────────────────────────────────────

fn onDecrement() void {
    state.count -= 1;
}

fn onIncrement() void {
    state.count += 1;
}

// ── UI builder ──────────────────────────────────────────────

fn build() ui.Node {
    // Title
    var title = ui.text("Counter", .{ .font_size = 32, .color = ui.Color.white });

    // Decrement button
    var minus_label = ui.text("-", .{ .font_size = 24, .color = ui.Color.WHITE });
    var minus_btn = ui.box(.{
        .style = .{
            .width = .{ .px = 60 },
            .height = .{ .px = 40 },
            .bg = ui.Color.red,
            .border_radius = 6,
            .align_items = .center,
            .justify_content = .center,
        },
        .children = &.{&minus_label},
        .on_click = &onDecrement,
    });

    // Count display (static for Phase 1 — dynamic text in Phase 2)
    var count_display = ui.text("0", .{ .font_size = 24, .color = ui.Color.white });

    // Increment button
    var plus_label = ui.text("+", .{ .font_size = 24, .color = ui.Color.WHITE });
    var plus_btn = ui.box(.{
        .style = .{
            .width = .{ .px = 60 },
            .height = .{ .px = 40 },
            .bg = ui.Color.green,
            .border_radius = 6,
            .align_items = .center,
            .justify_content = .center,
        },
        .children = &.{&plus_label},
        .on_click = &onIncrement,
    });

    // Button row
    var button_row = ui.box(.{
        .style = .{
            .direction = .row,
            .gap = 16,
            .justify_content = .center,
            .align_items = .center,
        },
        .children = &.{ &minus_btn, &count_display, &plus_btn },
    });

    // Root container
    const root = ui.box(.{
        .style = .{
            .direction = .column,
            .align_items = .center,
            .justify_content = .center,
            .gap = 24,
            .padding = ui.Edges.all(40),
            .bg = ui.Color.BG_PRIMARY,
            .width = .{ .px = 400 },
            .height = .{ .px = 300 },
        },
        .children = &.{ &title, &button_row },
    });

    return root;
}

// ── Entry point ─────────────────────────────────────────────

const builtin = @import("builtin");

pub fn main() !void {
    var app = ui.App.init("Counter App", 400, 300, &build);
    try app.setup();
    app.run();
    app.shutdown();
}

// Override _start for freestanding targets — skip argc/argv parsing.
fn trxStart() callconv(.c) noreturn {
    main() catch {};
    // exit(0) via Linux syscall nr 60
    asm volatile ("syscall"
        :
        : [nr] "{rax}" (@as(u64, 60)),
          [a1] "{rdi}" (@as(u64, 0)),
        : .{ .rcx = true, .r11 = true, .memory = true }
    );
    unreachable;
}

comptime {
    @export(&trxStart, .{ .name = "_start" });
}

// Prevent Zig's std _start from being linked in freestanding mode.
pub const _start = {};
