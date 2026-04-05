const color_mod = @import("color.zig");
pub const Color = color_mod.Color;

/// How a dimension is resolved.
pub const Dimension = union(enum) {
    /// Size determined by children / content.
    auto,
    /// Fixed pixel value.
    px: f32,
    /// Fraction of parent's available space (0.0 .. 1.0).
    fraction: f32,
};

/// Four-sided edge values (padding, margin, border).
pub const Edges = struct {
    top: f32 = 0,
    right: f32 = 0,
    bottom: f32 = 0,
    left: f32 = 0,

    pub fn horizontal(self: Edges) f32 {
        return self.left + self.right;
    }

    pub fn vertical(self: Edges) f32 {
        return self.top + self.bottom;
    }

    pub fn all(v: f32) Edges {
        return .{ .top = v, .right = v, .bottom = v, .left = v };
    }

    pub fn symmetric(h: f32, v: f32) Edges {
        return .{ .top = v, .right = h, .bottom = v, .left = h };
    }
};

/// Flexbox direction.
pub const FlexDirection = enum {
    row,
    column,
};

/// Cross-axis alignment.
pub const AlignItems = enum {
    start,
    end,
    center,
    stretch,
};

/// Main-axis distribution.
pub const JustifyContent = enum {
    start,
    end,
    center,
    space_between,
    space_around,
};

/// Overflow behaviour.
pub const Overflow = enum {
    visible,
    hidden,
    scroll,
};

/// Complete style for a UI node.
pub const Style = struct {
    // Layout
    direction: FlexDirection = .row,
    align_items: AlignItems = .start,
    justify_content: JustifyContent = .start,
    width: Dimension = .auto,
    height: Dimension = .auto,
    min_width: Dimension = .auto,
    min_height: Dimension = .auto,
    max_width: Dimension = .auto,
    max_height: Dimension = .auto,
    gap: f32 = 0,
    padding: Edges = .{},
    margin: Edges = .{},
    flex_grow: f32 = 0,
    flex_shrink: f32 = 1,
    overflow: Overflow = .visible,

    // Visual
    bg: Color = Color.TRANSPARENT,
    color: Color = Color.FG_PRIMARY,
    font_size: u16 = 16,
    border_radius: f32 = 0,
    border_width: f32 = 0,
    border_color: Color = Color.TRANSPARENT,
    opacity: f32 = 1.0,
};

// ── Tests ──────────────────────────────────────────────────────────

const testing = @import("std").testing;

test "edges helpers" {
    const e = Edges.all(8);
    try testing.expectEqual(@as(f32, 16), e.horizontal());
    try testing.expectEqual(@as(f32, 16), e.vertical());
}

test "edges symmetric" {
    const e = Edges.symmetric(10, 5);
    try testing.expectEqual(@as(f32, 20), e.horizontal());
    try testing.expectEqual(@as(f32, 10), e.vertical());
}

test "default style" {
    const s = Style{};
    try testing.expectEqual(FlexDirection.row, s.direction);
    try testing.expectEqual(AlignItems.start, s.align_items);
    try testing.expectEqual(@as(f32, 1.0), s.opacity);
    try testing.expect(Color.eql(Color.TRANSPARENT, s.bg));
}
