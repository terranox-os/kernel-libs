const node_mod = @import("node.zig");
pub const Node = node_mod.Node;
pub const NodeTag = node_mod.NodeTag;
pub const LayoutResult = node_mod.LayoutResult;
pub const Style = node_mod.Style;
pub const Dimension = node_mod.Dimension;
pub const Edges = node_mod.Edges;
pub const FlexDirection = node_mod.FlexDirection;
pub const AlignItems = node_mod.AlignItems;
pub const JustifyContent = node_mod.JustifyContent;

const text_mod = @import("text.zig");

/// Resolve a Dimension to a concrete pixel value.
fn resolve(dim: Dimension, parent_size: f32) f32 {
    return switch (dim) {
        .auto => 0, // auto handled specially by caller
        .px => |v| v,
        .fraction => |f| f * parent_size,
    };
}

fn isAuto(dim: Dimension) bool {
    return dim == .auto;
}

/// Measure the intrinsic size of a single node (text content, etc).
fn measureIntrinsic(node: *const Node) struct { w: f32, h: f32 } {
    if (node.tag == .text) {
        if (node.text_content) |content| {
            const font_h: f32 = @floatFromInt(node.style.font_size);
            const w = text_mod.measureTextWidth(content, node.style.font_size);
            return .{ .w = w, .h = font_h };
        }
    }
    return .{ .w = 0, .h = 0 };
}

/// Recursively compute layout for a node and all its descendants.
///
/// `avail_w` / `avail_h` are the space offered by the parent's content area.
/// After this call, every node in the subtree has its `layout` field populated
/// with absolute (x, y, width, height).
pub fn computeLayout(node: *Node, avail_w: f32, avail_h: f32) void {
    computeLayoutAt(node, 0, 0, avail_w, avail_h);
}

fn computeLayoutAt(node: *Node, origin_x: f32, origin_y: f32, avail_w: f32, avail_h: f32) void {
    const style = node.style;
    const pad = style.padding;

    // --- resolve own size ---
    var w: f32 = if (!isAuto(style.width)) resolve(style.width, avail_w) else avail_w;
    var h: f32 = if (!isAuto(style.height)) resolve(style.height, avail_h) else 0; // auto height determined below

    // Apply min/max constraints
    w = applyMinMax(w, style.min_width, style.max_width, avail_w);

    const content_w = w - pad.horizontal();
    const content_h_budget = if (!isAuto(style.height))
        h - pad.vertical()
    else
        avail_h - pad.vertical();

    const content_x = origin_x + pad.left;
    const content_y = origin_y + pad.top;

    // --- leaf node (text) ---
    if (node.tag == .text) {
        const intrinsic = measureIntrinsic(node);
        if (isAuto(style.width)) w = intrinsic.w + pad.horizontal();
        if (isAuto(style.height)) h = intrinsic.h + pad.vertical();
        h = applyMinMax(h, style.min_height, style.max_height, avail_h);
        node.layout = .{ .x = origin_x, .y = origin_y, .width = w, .height = h };
        return;
    }

    // --- box with children: flexbox algorithm ---
    const children = node.children;
    if (children.len == 0) {
        if (isAuto(style.height)) h = avail_h;
        h = applyMinMax(h, style.min_height, style.max_height, avail_h);
        node.layout = .{ .x = origin_x, .y = origin_y, .width = w, .height = h };
        return;
    }

    const is_row = style.direction == .row;
    const main_budget = if (is_row) content_w else content_h_budget;
    const cross_budget = if (is_row) content_h_budget else content_w;
    const gap = style.gap;

    // Phase 1: measure children along main axis (intrinsic sizes)
    var child_mains: [64]f32 = undefined;
    var child_crosses: [64]f32 = undefined;
    const n = @min(children.len, 64);
    var total_fixed: f32 = 0;
    var total_grow: f32 = 0;

    for (0..n) |i| {
        const child = children[i];
        const cs = child.style;

        // Resolve child's explicit main-axis size
        const child_main_dim = if (is_row) cs.width else cs.height;
        const child_cross_dim = if (is_row) cs.height else cs.width;

        if (!isAuto(child_main_dim)) {
            child_mains[i] = resolve(child_main_dim, main_budget);
        } else {
            // Intrinsic size
            const intr = measureIntrinsic(child);
            child_mains[i] = if (is_row) intr.w + cs.padding.horizontal() else intr.h + cs.padding.vertical();
            // For box children with no explicit size, we'll size after flex distribution
            if (child.tag == .box and child.children.len > 0) {
                child_mains[i] = 0; // will be distributed via flex_grow
            }
        }

        if (!isAuto(child_cross_dim)) {
            child_crosses[i] = resolve(child_cross_dim, cross_budget);
        } else {
            const intr = measureIntrinsic(child);
            child_crosses[i] = if (is_row) intr.h + cs.padding.vertical() else intr.w + cs.padding.horizontal();
        }

        total_fixed += child_mains[i];
        total_grow += cs.flex_grow;
    }

    // Phase 2: distribute remaining space via flex_grow
    const total_gap = gap * @as(f32, @floatFromInt(@max(n, 1) - 1));
    var remaining = main_budget - total_fixed - total_gap;
    if (remaining < 0) remaining = 0;

    if (total_grow > 0 and remaining > 0) {
        for (0..n) |i| {
            const grow = children[i].style.flex_grow;
            if (grow > 0) {
                child_mains[i] += (grow / total_grow) * remaining;
            }
        }
    }

    // Phase 3: compute justify offsets
    var main_cursor: f32 = 0;
    var item_gap: f32 = gap;
    var used_main: f32 = 0;
    for (0..n) |i| {
        used_main += child_mains[i];
    }
    used_main += total_gap;
    const free_main = main_budget - used_main;

    switch (style.justify_content) {
        .start => {},
        .end => {
            main_cursor = if (free_main > 0) free_main else 0;
        },
        .center => {
            main_cursor = if (free_main > 0) free_main / 2 else 0;
        },
        .space_between => {
            if (n > 1) {
                item_gap = gap + (if (free_main > 0) free_main / @as(f32, @floatFromInt(n - 1)) else 0);
            }
        },
        .space_around => {
            if (n > 0) {
                const slot = if (free_main > 0) free_main / @as(f32, @floatFromInt(n)) else 0;
                main_cursor = slot / 2;
                item_gap = gap + slot;
            }
        },
    }

    // Phase 4: position children and recurse
    var max_cross: f32 = 0;
    for (0..n) |i| {
        const child: *Node = @constCast(children[i]);
        const cs = child.style;
        const child_main = child_mains[i];
        var child_cross = child_crosses[i];

        // Cross-axis alignment
        const cross_offset: f32 = switch (style.align_items) {
            .start => 0,
            .end => if (cross_budget > child_cross) cross_budget - child_cross else 0,
            .center => if (cross_budget > child_cross) (cross_budget - child_cross) / 2 else 0,
            .stretch => blk: {
                child_cross = cross_budget;
                break :blk 0;
            },
        };

        const cx: f32 = if (is_row) content_x + main_cursor else content_x + cross_offset;
        const cy: f32 = if (is_row) content_y + cross_offset else content_y + main_cursor;
        const cw: f32 = if (is_row) child_main else child_cross;
        const ch: f32 = if (is_row) child_cross else child_main;

        // Apply margin offset
        const mx = cx + cs.margin.left;
        const my = cy + cs.margin.top;
        const mw = cw - cs.margin.horizontal();
        const mh = ch - cs.margin.vertical();

        computeLayoutAt(child, mx, my, mw, mh);

        // If child was auto-sized, use computed layout
        if (isAuto(if (is_row) cs.width else cs.height)) {
            // child_main stays as distributed
        }

        main_cursor += child_main + item_gap;
        if (child_cross > max_cross) max_cross = child_cross;
    }

    // Resolve auto height from children
    if (isAuto(style.height)) {
        if (is_row) {
            h = max_cross + pad.vertical();
        } else {
            h = main_cursor - item_gap + gap + pad.vertical(); // last gap already counted
            if (n > 0) h = main_cursor - item_gap + pad.vertical() + gap;
            // Simpler: use cursor minus last gap
            h = (main_cursor - item_gap) + pad.vertical();
        }
    }
    h = applyMinMax(h, style.min_height, style.max_height, avail_h);

    node.layout = .{ .x = origin_x, .y = origin_y, .width = w, .height = h };
}

fn applyMinMax(value: f32, min_dim: Dimension, max_dim: Dimension, parent: f32) f32 {
    var v = value;
    if (!isAuto(min_dim)) {
        const min_v = resolve(min_dim, parent);
        if (v < min_v) v = min_v;
    }
    if (!isAuto(max_dim)) {
        const max_v = resolve(max_dim, parent);
        if (v > max_v) v = max_v;
    }
    return v;
}

// ── Tests ──────────────────────────────────────────────────────────

const testing = @import("std").testing;

test "single box fills available width" {
    var root = Node{
        .tag = .box,
        .style = .{},
    };
    computeLayout(&root, 800, 600);
    try testing.expectEqual(@as(f32, 0), root.layout.x);
    try testing.expectEqual(@as(f32, 0), root.layout.y);
    try testing.expectEqual(@as(f32, 800), root.layout.width);
}

test "fixed size box" {
    var root = Node{
        .tag = .box,
        .style = .{ .width = .{ .px = 200 }, .height = .{ .px = 100 } },
    };
    computeLayout(&root, 800, 600);
    try testing.expectEqual(@as(f32, 200), root.layout.width);
    try testing.expectEqual(@as(f32, 100), root.layout.height);
}

test "row layout positions children side-by-side" {
    var c1 = Node{ .tag = .box, .style = .{ .width = .{ .px = 100 }, .height = .{ .px = 50 } } };
    var c2 = Node{ .tag = .box, .style = .{ .width = .{ .px = 150 }, .height = .{ .px = 50 } } };
    var root = Node{
        .tag = .box,
        .style = .{ .direction = .row, .width = .{ .px = 400 }, .height = .{ .px = 100 } },
        .children = &.{ &c1, &c2 },
    };
    computeLayout(&root, 400, 100);
    try testing.expectEqual(@as(f32, 0), c1.layout.x);
    try testing.expectEqual(@as(f32, 100), c2.layout.x);
    try testing.expectEqual(@as(f32, 100), c1.layout.width);
    try testing.expectEqual(@as(f32, 150), c2.layout.width);
}

test "column layout stacks children vertically" {
    var c1 = Node{ .tag = .box, .style = .{ .width = .{ .px = 100 }, .height = .{ .px = 40 } } };
    var c2 = Node{ .tag = .box, .style = .{ .width = .{ .px = 100 }, .height = .{ .px = 60 } } };
    var root = Node{
        .tag = .box,
        .style = .{ .direction = .column, .width = .{ .px = 200 }, .height = .{ .px = 200 } },
        .children = &.{ &c1, &c2 },
    };
    computeLayout(&root, 200, 200);
    try testing.expectEqual(@as(f32, 0), c1.layout.y);
    try testing.expectEqual(@as(f32, 40), c2.layout.y);
}

test "gap inserts space between children" {
    var c1 = Node{ .tag = .box, .style = .{ .width = .{ .px = 50 }, .height = .{ .px = 50 } } };
    var c2 = Node{ .tag = .box, .style = .{ .width = .{ .px = 50 }, .height = .{ .px = 50 } } };
    var root = Node{
        .tag = .box,
        .style = .{ .direction = .row, .gap = 10, .width = .{ .px = 200 }, .height = .{ .px = 100 } },
        .children = &.{ &c1, &c2 },
    };
    computeLayout(&root, 200, 100);
    try testing.expectEqual(@as(f32, 0), c1.layout.x);
    try testing.expectEqual(@as(f32, 60), c2.layout.x); // 50 + 10 gap
}

test "justify_content center" {
    var c1 = Node{ .tag = .box, .style = .{ .width = .{ .px = 100 }, .height = .{ .px = 50 } } };
    var root = Node{
        .tag = .box,
        .style = .{
            .direction = .row,
            .justify_content = .center,
            .width = .{ .px = 400 },
            .height = .{ .px = 100 },
        },
        .children = &.{&c1},
    };
    computeLayout(&root, 400, 100);
    try testing.expectEqual(@as(f32, 150), c1.layout.x); // (400-100)/2
}

test "justify_content end" {
    var c1 = Node{ .tag = .box, .style = .{ .width = .{ .px = 100 }, .height = .{ .px = 50 } } };
    var root = Node{
        .tag = .box,
        .style = .{
            .direction = .row,
            .justify_content = .end,
            .width = .{ .px = 400 },
            .height = .{ .px = 100 },
        },
        .children = &.{&c1},
    };
    computeLayout(&root, 400, 100);
    try testing.expectEqual(@as(f32, 300), c1.layout.x); // 400-100
}

test "justify_content space_between" {
    var c1 = Node{ .tag = .box, .style = .{ .width = .{ .px = 50 }, .height = .{ .px = 50 } } };
    var c2 = Node{ .tag = .box, .style = .{ .width = .{ .px = 50 }, .height = .{ .px = 50 } } };
    var root = Node{
        .tag = .box,
        .style = .{
            .direction = .row,
            .justify_content = .space_between,
            .width = .{ .px = 200 },
            .height = .{ .px = 100 },
        },
        .children = &.{ &c1, &c2 },
    };
    computeLayout(&root, 200, 100);
    try testing.expectEqual(@as(f32, 0), c1.layout.x);
    try testing.expectEqual(@as(f32, 150), c2.layout.x); // 200-50
}

test "align_items center" {
    var c1 = Node{ .tag = .box, .style = .{ .width = .{ .px = 50 }, .height = .{ .px = 30 } } };
    var root = Node{
        .tag = .box,
        .style = .{
            .direction = .row,
            .align_items = .center,
            .width = .{ .px = 200 },
            .height = .{ .px = 100 },
        },
        .children = &.{&c1},
    };
    computeLayout(&root, 200, 100);
    try testing.expectEqual(@as(f32, 35), c1.layout.y); // (100-30)/2
}

test "align_items stretch" {
    var c1 = Node{ .tag = .box, .style = .{ .width = .{ .px = 50 } } };
    var root = Node{
        .tag = .box,
        .style = .{
            .direction = .row,
            .align_items = .stretch,
            .width = .{ .px = 200 },
            .height = .{ .px = 100 },
        },
        .children = &.{&c1},
    };
    computeLayout(&root, 200, 100);
    try testing.expectEqual(@as(f32, 100), c1.layout.height);
}

test "flex_grow distributes remaining space" {
    var c1 = Node{ .tag = .box, .style = .{ .width = .{ .px = 100 }, .height = .{ .px = 50 }, .flex_grow = 0 } };
    var c2 = Node{ .tag = .box, .style = .{ .height = .{ .px = 50 }, .flex_grow = 1 } };
    var root = Node{
        .tag = .box,
        .style = .{ .direction = .row, .width = .{ .px = 400 }, .height = .{ .px = 100 } },
        .children = &.{ &c1, &c2 },
    };
    computeLayout(&root, 400, 100);
    try testing.expectEqual(@as(f32, 100), c1.layout.width);
    try testing.expectEqual(@as(f32, 300), c2.layout.width); // remaining
}

test "padding offsets children" {
    var c1 = Node{ .tag = .box, .style = .{ .width = .{ .px = 50 }, .height = .{ .px = 50 } } };
    var root = Node{
        .tag = .box,
        .style = .{
            .padding = Edges.all(20),
            .width = .{ .px = 200 },
            .height = .{ .px = 200 },
        },
        .children = &.{&c1},
    };
    computeLayout(&root, 200, 200);
    try testing.expectEqual(@as(f32, 20), c1.layout.x);
    try testing.expectEqual(@as(f32, 20), c1.layout.y);
}

test "text node measures intrinsic size" {
    var t = Node{
        .tag = .text,
        .style = .{ .font_size = 16 },
        .text_content = "AB",
    };
    computeLayout(&t, 800, 600);
    // 8x16 font: each char is 8px wide at font_size=16
    try testing.expectEqual(@as(f32, 16), t.layout.width);
    try testing.expectEqual(@as(f32, 16), t.layout.height);
}

test "fraction dimension" {
    var root = Node{
        .tag = .box,
        .style = .{ .width = .{ .fraction = 0.5 }, .height = .{ .fraction = 0.25 } },
    };
    computeLayout(&root, 800, 600);
    try testing.expectEqual(@as(f32, 400), root.layout.width);
    try testing.expectEqual(@as(f32, 150), root.layout.height);
}
