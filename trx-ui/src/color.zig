/// RGBA color type and the Obsidian Dark palette for TerranoxOS.
pub const Color = struct {
    r: u8,
    g: u8,
    b: u8,
    a: u8,

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) Color {
        return .{ .r = r, .g = g, .b = b, .a = a };
    }

    pub fn rgb(r: u8, g: u8, b: u8) Color {
        return .{ .r = r, .g = g, .b = b, .a = 255 };
    }

    /// Pack into 0xAARRGGBB (pre-multiplied for framebuffer write).
    pub fn toArgb32(self: Color) u32 {
        return @as(u32, self.a) << 24 |
            @as(u32, self.r) << 16 |
            @as(u32, self.g) << 8 |
            @as(u32, self.b);
    }

    /// Unpack from 0xAARRGGBB.
    pub fn fromArgb32(v: u32) Color {
        return .{
            .r = @truncate(v >> 16),
            .g = @truncate(v >> 8),
            .b = @truncate(v),
            .a = @truncate(v >> 24),
        };
    }

    /// Alpha-blend `self` (foreground) over `dst` (background).
    pub fn blendOver(self: Color, dst: Color) Color {
        if (self.a == 255) return self;
        if (self.a == 0) return dst;
        const sa: u16 = self.a;
        const inv: u16 = 255 - sa;
        return .{
            .r = @truncate((@as(u16, self.r) * sa + @as(u16, dst.r) * inv) / 255),
            .g = @truncate((@as(u16, self.g) * sa + @as(u16, dst.g) * inv) / 255),
            .b = @truncate((@as(u16, self.b) * sa + @as(u16, dst.b) * inv) / 255),
            .a = @truncate(sa + (@as(u16, dst.a) * inv) / 255),
        };
    }

    pub fn eql(a: Color, b: Color) bool {
        return a.r == b.r and a.g == b.g and a.b == b.b and a.a == b.a;
    }

    // ── Obsidian Dark palette ──────────────────────────────────────

    pub const TRANSPARENT = Color{ .r = 0, .g = 0, .b = 0, .a = 0 };
    pub const BLACK = Color.rgb(0, 0, 0);
    pub const WHITE = Color.rgb(255, 255, 255);

    // Backgrounds
    pub const BG_PRIMARY = Color.rgb(18, 18, 24); // deep obsidian
    pub const BG_SECONDARY = Color.rgb(28, 28, 38); // elevated surface
    pub const BG_TERTIARY = Color.rgb(38, 38, 52); // cards / panels
    pub const BG_HOVER = Color.rgb(48, 48, 64); // hover state

    // Foregrounds
    pub const FG_PRIMARY = Color.rgb(230, 230, 240); // main text
    pub const FG_SECONDARY = Color.rgb(160, 160, 180); // secondary text
    pub const FG_DISABLED = Color.rgb(100, 100, 120); // disabled text

    // Accents
    pub const ACCENT_PURPLE = Color.rgb(138, 92, 246); // primary action
    pub const ACCENT_BLUE = Color.rgb(56, 152, 236); // links / info
    pub const ACCENT_GREEN = Color.rgb(52, 211, 153); // success
    pub const ACCENT_YELLOW = Color.rgb(250, 204, 21); // warning
    pub const ACCENT_RED = Color.rgb(239, 68, 68); // error / destructive

    // Borders
    pub const BORDER_DEFAULT = Color.rgb(55, 55, 75);
    pub const BORDER_FOCUS = Color.rgb(138, 92, 246); // matches accent

    // Aliases
    pub const purple = ACCENT_PURPLE;
    pub const blue = ACCENT_BLUE;
    pub const green = ACCENT_GREEN;
    pub const yellow = ACCENT_YELLOW;
    pub const red = ACCENT_RED;
    pub const white = FG_PRIMARY;
    pub const dim = FG_SECONDARY;
};

// ── Tests ──────────────────────────────────────────────────────────

const testing = @import("std").testing;

test "rgba round-trip through argb32" {
    const c = Color.rgba(0xAA, 0xBB, 0xCC, 0xDD);
    const argb = c.toArgb32();
    const unpacked = Color.fromArgb32(argb);
    try testing.expect(Color.eql(c, unpacked));
}

test "blend fully opaque replaces" {
    const fg = Color.rgb(255, 0, 0);
    const bg = Color.rgb(0, 255, 0);
    const result = fg.blendOver(bg);
    try testing.expectEqual(@as(u8, 255), result.r);
    try testing.expectEqual(@as(u8, 0), result.g);
}

test "blend fully transparent passes through" {
    const fg = Color.rgba(255, 0, 0, 0);
    const bg = Color.rgb(0, 255, 0);
    const result = fg.blendOver(bg);
    try testing.expect(Color.eql(bg, result));
}

test "palette colors are opaque" {
    try testing.expectEqual(@as(u8, 255), Color.ACCENT_PURPLE.a);
    try testing.expectEqual(@as(u8, 255), Color.BG_PRIMARY.a);
    try testing.expectEqual(@as(u8, 0), Color.TRANSPARENT.a);
}
