/// Glyph atlas — packs the 8x16 bitmap font into a GPU-uploadable texture.
///
/// 95 ASCII glyphs (0x20-0x7E) packed into a 128x128 A8 (alpha) texture.
/// 16 glyphs per row, 6 rows = 96 cells (95 used + 1 blank).
/// Caller provides the 16KB pixel buffer.

const text_mod = @import("text.zig");
const gpu_mod = @import("gpu.zig");
const platform = @import("platform.zig");

pub const ATLAS_W: u32 = 128;
pub const ATLAS_H: u32 = 128;
pub const CELLS_PER_ROW: u32 = ATLAS_W / text_mod.GLYPH_W; // 16
pub const ATLAS_SIZE: u32 = ATLAS_W * ATLAS_H; // 16,384 bytes

const GLYPH_W = text_mod.GLYPH_W;
const GLYPH_H = text_mod.GLYPH_H;
const FIRST_CHAR = text_mod.FIRST_CHAR;
const LAST_CHAR = text_mod.LAST_CHAR;
const GLYPH_COUNT: u32 = LAST_CHAR - FIRST_CHAR + 1; // 95

/// UV coordinates for a glyph cell in the atlas (pixel coordinates).
pub const GlyphUV = struct {
    u0: u16, // left x
    v0: u16, // top y
    u1: u16, // right x
    v1: u16, // bottom y
};

/// A glyph atlas backed by a caller-provided pixel buffer.
pub const GlyphAtlas = struct {
    pixels: *[ATLAS_SIZE]u8,
    uv_table: [GLYPH_COUNT]GlyphUV = undefined,
    gpu_slot: ?u8 = null,

    /// Initialize the atlas with a caller-provided pixel buffer.
    /// Call `rasterize()` after init to fill the texture.
    pub fn init(pixels: *[ATLAS_SIZE]u8) GlyphAtlas {
        var atlas = GlyphAtlas{ .pixels = pixels };
        // Pre-compute UV table
        for (0..GLYPH_COUNT) |i| {
            const col: u16 = @intCast(i % CELLS_PER_ROW);
            const row: u16 = @intCast(i / CELLS_PER_ROW);
            atlas.uv_table[i] = .{
                .u0 = col * @as(u16, GLYPH_W),
                .v0 = row * @as(u16, GLYPH_H),
                .u1 = (col + 1) * @as(u16, GLYPH_W),
                .v1 = (row + 1) * @as(u16, GLYPH_H),
            };
        }
        return atlas;
    }

    /// Rasterize all 95 glyphs from the bitmap font into the atlas texture.
    /// Each pixel is 0x00 (transparent) or 0xFF (opaque).
    pub fn rasterize(self: *GlyphAtlas) void {
        // Clear atlas
        for (self.pixels) |*p| p.* = 0;

        for (0..GLYPH_COUNT) |glyph_idx| {
            const glyph = text_mod.font_data[glyph_idx];
            const uv = self.uv_table[glyph_idx];
            const base_x: u32 = uv.u0;
            const base_y: u32 = uv.v0;

            // Each glyph row is one byte, MSB = leftmost pixel
            for (0..GLYPH_H) |dy| {
                const row_bits: u8 = glyph[dy];
                for (0..GLYPH_W) |dx| {
                    const bit: u8 = @as(u8, 0x80) >> @intCast(dx);
                    const alpha: u8 = if (row_bits & bit != 0) 0xFF else 0x00;
                    const px = (base_y + @as(u32, @intCast(dy))) * ATLAS_W + (base_x + @as(u32, @intCast(dx)));
                    self.pixels[px] = alpha;
                }
            }
        }
    }

    /// Look up the UV coordinates for an ASCII character.
    /// Returns null for characters outside the printable range.
    pub fn getUV(self: *const GlyphAtlas, char: u8) ?GlyphUV {
        if (char < FIRST_CHAR or char > LAST_CHAR) return null;
        return self.uv_table[char - FIRST_CHAR];
    }

    /// Upload the atlas texture to a GPU buffer object.
    pub fn uploadToGpu(self: *GlyphAtlas, gpu_ctx: *gpu_mod.GpuContext) gpu_mod.GpuError!void {
        const slot = try gpu_ctx.allocBuffer(ATLAS_SIZE, .texture);
        const mapped = try gpu_ctx.mapBuffer(slot);

        // Copy atlas pixels to GPU memory
        for (0..ATLAS_SIZE) |i| {
            mapped[i] = self.pixels[i];
        }
        self.gpu_slot = slot;
    }
};

// ── Tests ──────────────────────────────────────────────────────────

const testing = @import("std").testing;

test "atlas dimensions are power-of-2" {
    try testing.expectEqual(@as(u32, 128), ATLAS_W);
    try testing.expectEqual(@as(u32, 128), ATLAS_H);
    try testing.expectEqual(@as(u32, 16384), ATLAS_SIZE);
}

test "GLYPH_COUNT is 95" {
    try testing.expectEqual(@as(u32, 95), GLYPH_COUNT);
    try testing.expectEqual(@as(u32, 16), CELLS_PER_ROW);
}

test "getUV returns correct cell for 'A'" {
    var pixels: [ATLAS_SIZE]u8 = [_]u8{0} ** ATLAS_SIZE;
    const atlas = GlyphAtlas.init(&pixels);
    // 'A' = 0x41, index = 0x41 - 0x20 = 33, col = 33 % 16 = 1, row = 33 / 16 = 2
    const uv = atlas.getUV('A') orelse unreachable;
    try testing.expectEqual(@as(u16, 1 * 8), uv.u0); // col 1 * 8
    try testing.expectEqual(@as(u16, 2 * 16), uv.v0); // row 2 * 16
    try testing.expectEqual(@as(u16, 2 * 8), uv.u1);
    try testing.expectEqual(@as(u16, 3 * 16), uv.v1);
}

test "getUV returns correct cell for space (first char)" {
    var pixels: [ATLAS_SIZE]u8 = [_]u8{0} ** ATLAS_SIZE;
    const atlas = GlyphAtlas.init(&pixels);
    const uv = atlas.getUV(' ') orelse unreachable;
    try testing.expectEqual(@as(u16, 0), uv.u0);
    try testing.expectEqual(@as(u16, 0), uv.v0);
}

test "getUV returns null for non-printable char" {
    var pixels: [ATLAS_SIZE]u8 = [_]u8{0} ** ATLAS_SIZE;
    const atlas = GlyphAtlas.init(&pixels);
    try testing.expect(atlas.getUV(0x19) == null); // below FIRST_CHAR
    try testing.expect(atlas.getUV(0x7F) == null); // above LAST_CHAR
    try testing.expect(atlas.getUV(0x00) == null); // NUL
}

test "rasterize produces non-zero pixels for '!'" {
    var pixels: [ATLAS_SIZE]u8 = [_]u8{0} ** ATLAS_SIZE;
    var atlas = GlyphAtlas.init(&pixels);
    atlas.rasterize();

    // '!' = 0x21, index 1, col=1, row=0
    const uv = atlas.getUV('!') orelse unreachable;
    // Check that at least some pixels in this cell are non-zero
    var non_zero: u32 = 0;
    var y: u32 = uv.v0;
    while (y < uv.v1) : (y += 1) {
        var x: u32 = uv.u0;
        while (x < uv.u1) : (x += 1) {
            if (pixels[y * ATLAS_W + x] != 0) non_zero += 1;
        }
    }
    try testing.expect(non_zero > 0);
}

test "rasterize produces zero pixels for space" {
    var pixels: [ATLAS_SIZE]u8 = [_]u8{0} ** ATLAS_SIZE;
    var atlas = GlyphAtlas.init(&pixels);
    atlas.rasterize();

    // ' ' = 0x20, index 0 — should be all zeros (space character)
    const uv = atlas.getUV(' ') orelse unreachable;
    var non_zero: u32 = 0;
    var y: u32 = uv.v0;
    while (y < uv.v1) : (y += 1) {
        var x: u32 = uv.u0;
        while (x < uv.u1) : (x += 1) {
            if (pixels[y * ATLAS_W + x] != 0) non_zero += 1;
        }
    }
    try testing.expectEqual(@as(u32, 0), non_zero);
}

test "atlas GPU upload fails gracefully without GPU" {
    var pixels: [ATLAS_SIZE]u8 = [_]u8{0} ** ATLAS_SIZE;
    var atlas = GlyphAtlas.init(&pixels);
    var gpu_ctx = gpu_mod.GpuContext{}; // unavailable
    try testing.expectError(gpu_mod.GpuError.DeviceNotAvailable, atlas.uploadToGpu(&gpu_ctx));
    try testing.expect(atlas.gpu_slot == null);
}
