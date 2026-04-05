const style_mod = @import("style.zig");
pub const Style = style_mod.Style;
pub const Color = style_mod.Color;
pub const Dimension = style_mod.Dimension;
pub const Edges = style_mod.Edges;
pub const FlexDirection = style_mod.FlexDirection;
pub const AlignItems = style_mod.AlignItems;
pub const JustifyContent = style_mod.JustifyContent;

/// Result of layout computation — absolute pixel rect.
pub const LayoutResult = struct {
    x: f32 = 0,
    y: f32 = 0,
    width: f32 = 0,
    height: f32 = 0,
};

/// Tag discriminating node kinds.
pub const NodeTag = enum {
    box,
    text,
};

/// A single node in the UI tree.
///
/// Nodes are lightweight value types built each frame by the
/// application's builder function.  The framework computes layout,
/// renders, and then discards the tree.
pub const Node = struct {
    tag: NodeTag = .box,
    style: Style = .{},
    children: []const *const Node = &.{},
    text_content: ?[]const u8 = null,
    on_click: ?*const fn () void = null,
    on_key: ?*const fn (u32) void = null,
    /// Filled in by layout pass.
    layout: LayoutResult = .{},
};

/// Options accepted by `box()`.
pub const BoxOptions = struct {
    style: Style = .{},
    children: []const *const Node = &.{},
    on_click: ?*const fn () void = null,
    on_key: ?*const fn (u32) void = null,
};

/// Options accepted by `text()`.
pub const TextOptions = struct {
    color: Color = Color.FG_PRIMARY,
    font_size: u16 = 16,
    on_click: ?*const fn () void = null,
};

/// Construct a box node.
pub fn box(opts: BoxOptions) Node {
    return .{
        .tag = .box,
        .style = opts.style,
        .children = opts.children,
        .text_content = null,
        .on_click = opts.on_click,
        .on_key = opts.on_key,
    };
}

/// Construct a text node.
pub fn text(content: []const u8, opts: TextOptions) Node {
    var s = Style{};
    s.color = opts.color;
    s.font_size = opts.font_size;
    return .{
        .tag = .text,
        .style = s,
        .children = &.{},
        .text_content = content,
        .on_click = opts.on_click,
    };
}

// ── Tests ──────────────────────────────────────────────────────────

const testing = @import("std").testing;

test "box creates box node" {
    const n = box(.{});
    try testing.expectEqual(NodeTag.box, n.tag);
    try testing.expectEqual(@as(usize, 0), n.children.len);
    try testing.expectEqual(@as(?[]const u8, null), n.text_content);
}

test "text creates text node" {
    const n = text("hello", .{ .font_size = 24 });
    try testing.expectEqual(NodeTag.text, n.tag);
    try testing.expectEqualStrings("hello", n.text_content.?);
    try testing.expectEqual(@as(u16, 24), n.style.font_size);
}

test "box with children" {
    const child = text("child", .{});
    const parent = box(.{ .children = &.{&child} });
    try testing.expectEqual(@as(usize, 1), parent.children.len);
    try testing.expectEqualStrings("child", parent.children[0].text_content.?);
}
