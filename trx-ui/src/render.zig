/// Software renderer — draws node trees into an ARGB32 pixel buffer.

const node_mod = @import("node.zig");
const text_mod = @import("text.zig");
const color_mod = @import("color.zig");
const layout_mod = @import("layout.zig");

pub const Node = node_mod.Node;
pub const NodeTag = node_mod.NodeTag;
pub const Color = color_mod.Color;

/// A framebuffer abstraction wrapping an ARGB32 pixel slice.
pub const Framebuffer = struct {
    pixels: []u32,
    width: u32,
    height: u32,

    pub fn init(pixels: []u32, width: u32, height: u32) Framebuffer {
        return .{ .pixels = pixels, .width = width, .height = height };
    }

    /// Fill a rectangle with a solid colour.
    pub fn fillRect(self: *Framebuffer, x: i32, y: i32, w: u32, h: u32, col: Color) void {
        if (col.a == 0) return;
        const argb = col.toArgb32();
        const x0: u32 = if (x < 0) 0 else @intCast(x);
        const y0: u32 = if (y < 0) 0 else @intCast(y);
        const x1 = @min(x0 + w, self.width);
        const y1 = @min(y0 + h, self.height);

        var row = y0;
        while (row < y1) : (row += 1) {
            var cx = x0;
            while (cx < x1) : (cx += 1) {
                const idx = row * self.width + cx;
                if (col.a == 255) {
                    self.pixels[idx] = argb;
                } else {
                    const dst = Color.fromArgb32(self.pixels[idx]);
                    self.pixels[idx] = col.blendOver(dst).toArgb32();
                }
            }
        }
    }

    /// Draw a 1px border rectangle (no fill).
    pub fn strokeRect(self: *Framebuffer, x: i32, y: i32, w: u32, h: u32, col: Color, thickness: u32) void {
        if (col.a == 0 or thickness == 0) return;
        // Top
        self.fillRect(x, y, w, thickness, col);
        // Bottom
        self.fillRect(x, y + @as(i32, @intCast(h)) - @as(i32, @intCast(thickness)), w, thickness, col);
        // Left
        self.fillRect(x, y, thickness, h, col);
        // Right
        self.fillRect(x + @as(i32, @intCast(w)) - @as(i32, @intCast(thickness)), y, thickness, h, col);
    }

    /// Clear the entire buffer.
    pub fn clear(self: *Framebuffer, col: Color) void {
        const argb = col.toArgb32();
        for (self.pixels) |*p| {
            p.* = argb;
        }
    }
};

/// Render an entire node tree into a framebuffer.
pub fn renderTree(fb: *Framebuffer, node: *const Node) void {
    renderNodeRecursive(fb, node);
}

fn renderNodeRecursive(fb: *Framebuffer, node: *const Node) void {
    const l = node.layout;
    const s = node.style;
    const x: i32 = @intFromFloat(l.x);
    const y: i32 = @intFromFloat(l.y);
    const w: u32 = @intFromFloat(@max(l.width, 0));
    const h: u32 = @intFromFloat(@max(l.height, 0));

    // Background fill
    if (s.bg.a > 0) {
        fb.fillRect(x, y, w, h, s.bg);
    }

    // Border
    if (s.border_width > 0 and s.border_color.a > 0) {
        const bw: u32 = @intFromFloat(@max(s.border_width, 0));
        fb.strokeRect(x, y, w, h, s.border_color, bw);
    }

    // Text content
    if (node.tag == .text) {
        if (node.text_content) |content| {
            const pad_x: i32 = @intFromFloat(s.padding.left);
            const pad_y: i32 = @intFromFloat(s.padding.top);
            text_mod.renderText(
                fb.pixels,
                fb.width,
                x + pad_x,
                y + pad_y,
                content,
                s.font_size,
                s.color,
            );
        }
        return;
    }

    // Recurse into children
    for (node.children) |child| {
        renderNodeRecursive(fb, child);
    }
}

// ── Tests ──────────────────────────────────────────────────────────

const testing = @import("std").testing;

test "framebuffer clear" {
    var pixels: [4]u32 = undefined;
    var fb = Framebuffer.init(&pixels, 2, 2);
    fb.clear(Color.BLACK);
    try testing.expectEqual(@as(u32, 0xFF000000), pixels[0]);
    try testing.expectEqual(@as(u32, 0xFF000000), pixels[3]);
}

test "fillRect solid" {
    var pixels: [16]u32 = [_]u32{0} ** 16;
    var fb = Framebuffer.init(&pixels, 4, 4);
    fb.fillRect(1, 1, 2, 2, Color.rgb(255, 0, 0));
    // (1,1) should be red
    try testing.expectEqual(@as(u32, 0xFF_FF0000), pixels[1 * 4 + 1]);
    // (0,0) should still be 0
    try testing.expectEqual(@as(u32, 0), pixels[0]);
}

test "fillRect clamps to bounds" {
    var pixels: [4]u32 = [_]u32{0} ** 4;
    var fb = Framebuffer.init(&pixels, 2, 2);
    // Rect extends past buffer — should not crash
    fb.fillRect(-1, -1, 10, 10, Color.rgb(0, 255, 0));
    try testing.expectEqual(@as(u32, 0xFF_00FF00), pixels[0]);
}

test "strokeRect draws border" {
    var pixels: [36]u32 = [_]u32{0} ** 36;
    var fb = Framebuffer.init(&pixels, 6, 6);
    fb.strokeRect(1, 1, 4, 4, Color.rgb(255, 255, 255), 1);
    // Top-left corner of border at (1,1)
    try testing.expectEqual(@as(u32, 0xFF_FFFFFF), pixels[1 * 6 + 1]);
    // Interior (2,2) should be untouched
    try testing.expectEqual(@as(u32, 0), pixels[2 * 6 + 2]);
}

test "renderTree renders box with bg" {
    var pixels: [100]u32 = [_]u32{0} ** 100;
    var fb = Framebuffer.init(&pixels, 10, 10);

    var root = Node{
        .tag = .box,
        .style = .{ .bg = Color.ACCENT_PURPLE },
        .layout = .{ .x = 0, .y = 0, .width = 10, .height = 10 },
    };
    renderTree(&fb, &root);
    try testing.expectEqual(Color.ACCENT_PURPLE.toArgb32(), pixels[0]);
}
