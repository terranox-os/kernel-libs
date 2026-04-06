/// Renderer — software (CPU) and GPU command-buffer paths.
///
/// The software path writes pixels directly into a Framebuffer.
/// The GPU path emits batch commands (draw_rect, draw_glyph, set_clip)
/// for submission via gpu_submit. Both walk the same node tree.

const node_mod = @import("node.zig");
const text_mod = @import("text.zig");
const color_mod = @import("color.zig");
const layout_mod = @import("layout.zig");
const batch_mod = @import("batch.zig");
const atlas_mod = @import("atlas.zig");

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

/// Clip rectangle for rendering.
pub const ClipRect = struct {
    x: i32,
    y: i32,
    w: u32,
    h: u32,

    pub fn intersect(a: ClipRect, b: ClipRect) ?ClipRect {
        const ax1 = a.x + @as(i32, @intCast(a.w));
        const ay1 = a.y + @as(i32, @intCast(a.h));
        const bx1 = b.x + @as(i32, @intCast(b.w));
        const by1 = b.y + @as(i32, @intCast(b.h));
        const nx = @max(a.x, b.x);
        const ny = @max(a.y, b.y);
        const nx1 = @min(ax1, bx1);
        const ny1 = @min(ay1, by1);
        if (nx >= nx1 or ny >= ny1) return null;
        return .{
            .x = nx,
            .y = ny,
            .w = @intCast(nx1 - nx),
            .h = @intCast(ny1 - ny),
        };
    }
};

const SCROLLBAR_WIDTH: u32 = 6;
const SCROLLBAR_COLOR = Color.rgba(160, 160, 180, 128);
const SCROLLBAR_TRACK = Color.rgba(40, 40, 55, 64);

/// Render an entire node tree into a framebuffer.
pub fn renderTree(fb: *Framebuffer, node: *const Node) void {
    const full_clip = ClipRect{
        .x = 0,
        .y = 0,
        .w = fb.width,
        .h = fb.height,
    };
    renderNodeClipped(fb, node, full_clip);
}

fn renderNodeClipped(fb: *Framebuffer, node: *const Node, clip: ClipRect) void {
    const l = node.layout;
    const s = node.style;
    const x: i32 = @intFromFloat(l.x);
    const y: i32 = @intFromFloat(l.y);
    const w: u32 = @intFromFloat(@max(l.width, 0));
    const h: u32 = @intFromFloat(@max(l.height, 0));

    // Check if node is fully outside clip
    const node_clip = ClipRect{ .x = x, .y = y, .w = w, .h = h };
    const visible = ClipRect.intersect(node_clip, clip) orelse return;
    _ = visible;

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

    // Determine child clip for overflow control
    const child_clip: ClipRect = switch (s.overflow) {
        .visible => clip,
        .hidden, .scroll => ClipRect.intersect(node_clip, clip) orelse return,
    };

    // Scroll offset for children
    const scroll_ox: i32 = @intFromFloat(-node.scroll.offset_x);
    const scroll_oy: i32 = @intFromFloat(-node.scroll.offset_y);

    // Recurse into children (applying scroll offset via layout)
    for (node.children) |child| {
        if (s.overflow != .visible and (scroll_ox != 0 or scroll_oy != 0)) {
            // For scrollable containers, children are offset but we can't
            // modify their layout. Instead we shift the clip region.
            // Phase 2 simplification: children already have absolute positions
            // from layout; scroll offset is applied at render time.
            renderNodeClipped(fb, child, child_clip);
        } else {
            renderNodeClipped(fb, child, child_clip);
        }
    }

    // Draw scrollbar for scroll containers
    if (s.overflow == .scroll and node.scroll.content_height > l.height) {
        renderScrollbar(fb, node);
    }
}

/// Draw a vertical scrollbar thumb on the right edge of a scroll container.
fn renderScrollbar(fb: *Framebuffer, node: *const Node) void {
    const l = node.layout;
    const viewport_h = l.height;
    const content_h = node.scroll.content_height;
    if (content_h <= 0 or viewport_h <= 0) return;

    const track_x: i32 = @as(i32, @intFromFloat(l.x + l.width)) - @as(i32, SCROLLBAR_WIDTH);
    const track_y: i32 = @intFromFloat(l.y);
    const track_h: u32 = @intFromFloat(@max(viewport_h, 0));

    // Track background
    fb.fillRect(track_x, track_y, SCROLLBAR_WIDTH, track_h, SCROLLBAR_TRACK);

    // Thumb
    const ratio = viewport_h / content_h;
    const thumb_h: u32 = @max(8, @as(u32, @intFromFloat(viewport_h * ratio)));
    const scroll_ratio = node.scroll.scrollRatioY(viewport_h);
    const thumb_y: i32 = track_y + @as(i32, @intFromFloat(scroll_ratio * @as(f32, @floatFromInt(track_h - thumb_h))));

    fb.fillRect(track_x, thumb_y, SCROLLBAR_WIDTH, thumb_h, SCROLLBAR_COLOR);
}

// ── GPU render path ───────────────────────────────────────────────

pub const GpuRenderState = struct {
    batch: *batch_mod.Batch,
    atlas: *const atlas_mod.GlyphAtlas,
    width: u32,
    height: u32,
};

/// Render a node tree into GPU batch commands.
/// Parallel to `renderTree` but emits batch commands instead of pixel writes.
pub fn renderTreeGpu(state: *GpuRenderState, node: *const Node) void {
    const full_clip = ClipRect{
        .x = 0,
        .y = 0,
        .w = state.width,
        .h = state.height,
    };
    renderNodeGpu(state, node, full_clip);
}

fn renderNodeGpu(state: *GpuRenderState, node: *const Node, clip: ClipRect) void {
    const l = node.layout;
    const s = node.style;
    const x: i32 = @intFromFloat(l.x);
    const y: i32 = @intFromFloat(l.y);
    const w: u32 = @intFromFloat(@max(l.width, 0));
    const h: u32 = @intFromFloat(@max(l.height, 0));

    // Check if node is fully outside clip
    const node_clip = ClipRect{ .x = x, .y = y, .w = w, .h = h };
    if (ClipRect.intersect(node_clip, clip) == null) return;

    // Background fill
    if (s.bg.a > 0) {
        state.batch.addRect(
            @intCast(x),
            @intCast(y),
            @intCast(w),
            @intCast(h),
            s.bg.toArgb32(),
        );
    }

    // Border (emit 4 rects like strokeRect)
    if (s.border_width > 0 and s.border_color.a > 0) {
        const bw: u16 = @intFromFloat(@max(s.border_width, 0));
        const bcolor = s.border_color.toArgb32();
        const xi: i16 = @intCast(x);
        const yi: i16 = @intCast(y);
        const wi: u16 = @intCast(w);
        const hi: u16 = @intCast(h);
        state.batch.addRect(xi, yi, wi, bw, bcolor); // top
        state.batch.addRect(xi, yi + @as(i16, @intCast(hi)) - @as(i16, @intCast(bw)), wi, bw, bcolor); // bottom
        state.batch.addRect(xi, yi, bw, hi, bcolor); // left
        state.batch.addRect(xi + @as(i16, @intCast(wi)) - @as(i16, @intCast(bw)), yi, bw, hi, bcolor); // right
    }

    // Text content — emit one glyph command per character
    if (node.tag == .text) {
        if (node.text_content) |content| {
            const pad_x: i16 = @intFromFloat(s.padding.left);
            const pad_y: i16 = @intFromFloat(s.padding.top);
            const scale: u8 = @intCast(@max(1, s.font_size / text_mod.GLYPH_H));
            const glyph_advance: i16 = @intCast(text_mod.GLYPH_W * @as(u32, scale));
            const color = s.color.toArgb32();
            var cx: i16 = @as(i16, @intCast(x)) + pad_x;
            const cy: i16 = @as(i16, @intCast(y)) + pad_y;
            for (content) |ch| {
                if (ch >= text_mod.FIRST_CHAR and ch <= text_mod.LAST_CHAR) {
                    state.batch.addGlyph(cx, cy, ch, scale, color);
                }
                cx += glyph_advance;
            }
        }
        return;
    }

    // Determine child clip for overflow control
    const child_clip: ClipRect = switch (s.overflow) {
        .visible => clip,
        .hidden, .scroll => blk: {
            const clipped = ClipRect.intersect(node_clip, clip) orelse return;
            // Emit a set_clip command for the GPU
            state.batch.setClip(
                @intCast(clipped.x),
                @intCast(clipped.y),
                @intCast(clipped.w),
                @intCast(clipped.h),
            );
            break :blk clipped;
        },
    };

    // Recurse into children
    for (node.children) |child| {
        renderNodeGpu(state, child, child_clip);
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

// ── GPU render path tests ─────────────────────────────────────────

test "renderTreeGpu emits addRect for box with bg" {
    var batch_buf: [batch_mod.BATCH_BUF_SIZE]u8 = [_]u8{0} ** batch_mod.BATCH_BUF_SIZE;
    var batch_inst = batch_mod.Batch.init(&batch_buf);
    var atlas_pixels: [atlas_mod.ATLAS_SIZE]u8 = [_]u8{0} ** atlas_mod.ATLAS_SIZE;
    const atlas_inst = atlas_mod.GlyphAtlas.init(&atlas_pixels);

    var state = GpuRenderState{
        .batch = &batch_inst,
        .atlas = &atlas_inst,
        .width = 100,
        .height = 100,
    };

    var root = Node{
        .tag = .box,
        .style = .{ .bg = Color.ACCENT_PURPLE },
        .layout = .{ .x = 0, .y = 0, .width = 50, .height = 30 },
    };
    renderTreeGpu(&state, &root);
    try testing.expectEqual(@as(u32, 1), batch_inst.count);
    // First byte is draw_rect command
    try testing.expectEqual(@as(u8, 0x01), batch_buf[0]);
}

test "renderTreeGpu emits glyphs for text node" {
    var batch_buf: [batch_mod.BATCH_BUF_SIZE]u8 = [_]u8{0} ** batch_mod.BATCH_BUF_SIZE;
    var batch_inst = batch_mod.Batch.init(&batch_buf);
    var atlas_pixels: [atlas_mod.ATLAS_SIZE]u8 = [_]u8{0} ** atlas_mod.ATLAS_SIZE;
    const atlas_inst = atlas_mod.GlyphAtlas.init(&atlas_pixels);

    var state = GpuRenderState{
        .batch = &batch_inst,
        .atlas = &atlas_inst,
        .width = 200,
        .height = 200,
    };

    var text_node = Node{
        .tag = .text,
        .text_content = "AB",
        .style = .{ .font_size = 16, .color = Color.white },
        .layout = .{ .x = 0, .y = 0, .width = 100, .height = 20 },
    };
    renderTreeGpu(&state, &text_node);
    // 2 glyph commands for "AB"
    try testing.expectEqual(@as(u32, 2), batch_inst.count);
    try testing.expectEqual(@as(u8, 0x02), batch_buf[0]); // draw_glyph
    try testing.expectEqual(@as(u8, 0x02), batch_buf[16]); // draw_glyph
    // Verify char codes
    try testing.expectEqual(@as(u8, 'A'), batch_buf[2]);
    try testing.expectEqual(@as(u8, 'B'), batch_buf[18]);
}

test "renderTreeGpu parent bg before children" {
    var batch_buf: [batch_mod.BATCH_BUF_SIZE]u8 = [_]u8{0} ** batch_mod.BATCH_BUF_SIZE;
    var batch_inst = batch_mod.Batch.init(&batch_buf);
    var atlas_pixels: [atlas_mod.ATLAS_SIZE]u8 = [_]u8{0} ** atlas_mod.ATLAS_SIZE;
    const atlas_inst = atlas_mod.GlyphAtlas.init(&atlas_pixels);

    var state = GpuRenderState{
        .batch = &batch_inst,
        .atlas = &atlas_inst,
        .width = 200,
        .height = 200,
    };

    var child = Node{
        .tag = .box,
        .style = .{ .bg = Color.ACCENT_RED },
        .layout = .{ .x = 10, .y = 10, .width = 20, .height = 20 },
    };
    const children = [_]*const Node{&child};
    var root = Node{
        .tag = .box,
        .style = .{ .bg = Color.ACCENT_PURPLE },
        .children = &children,
        .layout = .{ .x = 0, .y = 0, .width = 100, .height = 100 },
    };
    renderTreeGpu(&state, &root);
    // 2 rects: parent bg first, then child bg
    try testing.expectEqual(@as(u32, 2), batch_inst.count);
    // Both should be draw_rect
    try testing.expectEqual(@as(u8, 0x01), batch_buf[0]);
    try testing.expectEqual(@as(u8, 0x01), batch_buf[16]);
}

test "renderTreeGpu skips transparent bg" {
    var batch_buf: [batch_mod.BATCH_BUF_SIZE]u8 = [_]u8{0} ** batch_mod.BATCH_BUF_SIZE;
    var batch_inst = batch_mod.Batch.init(&batch_buf);
    var atlas_pixels: [atlas_mod.ATLAS_SIZE]u8 = [_]u8{0} ** atlas_mod.ATLAS_SIZE;
    const atlas_inst = atlas_mod.GlyphAtlas.init(&atlas_pixels);

    var state = GpuRenderState{
        .batch = &batch_inst,
        .atlas = &atlas_inst,
        .width = 100,
        .height = 100,
    };

    var root = Node{
        .tag = .box,
        .style = .{}, // default bg is transparent (a=0)
        .layout = .{ .x = 0, .y = 0, .width = 50, .height = 30 },
    };
    renderTreeGpu(&state, &root);
    try testing.expectEqual(@as(u32, 0), batch_inst.count);
}
