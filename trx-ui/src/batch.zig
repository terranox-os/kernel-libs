/// Command buffer batching for GPU submission.
///
/// Accumulates draw commands (rects, glyphs, clip regions) during a frame
/// and serializes them into a flat byte buffer for a single `gpu_submit` call.
/// All command structs are exactly 16 bytes (aligned, cache-friendly).
/// Caller provides the 64KB batch buffer.

pub const CMD_SIZE: u32 = 16;
pub const MAX_COMMANDS: u32 = 4096;
pub const BATCH_BUF_SIZE: u32 = MAX_COMMANDS * CMD_SIZE; // 65,536 bytes

// ── Command types ─────────────────────────────────────────────────

pub const CmdType = enum(u8) {
    draw_rect = 0x01,
    draw_glyph = 0x02,
    set_clip = 0x03,
};

pub const CmdDrawRect = extern struct {
    cmd: CmdType = .draw_rect,
    _pad: [3]u8 = .{ 0, 0, 0 },
    x: i16,
    y: i16,
    w: u16,
    h: u16,
    color: u32, // ARGB32
};

pub const CmdDrawGlyph = extern struct {
    cmd: CmdType = .draw_glyph,
    _pad: [1]u8 = .{0},
    char_code: u8,
    scale: u8,
    x: i16,
    y: i16,
    color: u32, // ARGB32 foreground
    atlas_slot: u8,
    _pad2: [3]u8 = .{ 0, 0, 0 },
};

pub const CmdSetClip = extern struct {
    cmd: CmdType = .set_clip,
    _pad: [3]u8 = .{ 0, 0, 0 },
    x: i16,
    y: i16,
    w: u16,
    h: u16,
    _pad2: [4]u8 = .{ 0, 0, 0, 0 },
};

// ── Batch ─────────────────────────────────────────────────────────

pub const Batch = struct {
    buf: *[BATCH_BUF_SIZE]u8,
    count: u32 = 0,
    atlas_slot: u8 = 0,

    pub fn init(buf: *[BATCH_BUF_SIZE]u8) Batch {
        return .{ .buf = buf };
    }

    pub fn reset(self: *Batch) void {
        self.count = 0;
    }

    pub fn addRect(self: *Batch, x: i16, y: i16, w: u16, h: u16, color: u32) void {
        if (self.count >= MAX_COMMANDS) return;
        const cmd = CmdDrawRect{ .x = x, .y = y, .w = w, .h = h, .color = color };
        self.writeCmd(&cmd);
    }

    pub fn addGlyph(self: *Batch, x: i16, y: i16, char_code: u8, scale: u8, color: u32) void {
        if (self.count >= MAX_COMMANDS) return;
        const cmd = CmdDrawGlyph{
            .char_code = char_code,
            .scale = scale,
            .x = x,
            .y = y,
            .color = color,
            .atlas_slot = self.atlas_slot,
        };
        self.writeCmd(&cmd);
    }

    pub fn setClip(self: *Batch, x: i16, y: i16, w: u16, h: u16) void {
        if (self.count >= MAX_COMMANDS) return;
        const cmd = CmdSetClip{ .x = x, .y = y, .w = w, .h = h };
        self.writeCmd(&cmd);
    }

    fn writeCmd(self: *Batch, cmd: anytype) void {
        const offset = self.count * CMD_SIZE;
        const bytes: *const [CMD_SIZE]u8 = @ptrCast(cmd);
        @memcpy(self.buf[offset..][0..CMD_SIZE], bytes);
        self.count += 1;
    }

    pub fn usedBytes(self: *const Batch) u32 {
        return self.count * CMD_SIZE;
    }

    pub fn commandSlice(self: *const Batch) []const u8 {
        return self.buf[0..self.usedBytes()];
    }

    pub fn isFull(self: *const Batch) bool {
        return self.count >= MAX_COMMANDS;
    }
};

// ── Tests ──────────────────────────────────────────────────────────

const testing = @import("std").testing;

test "command structs are exactly 16 bytes" {
    try testing.expectEqual(@as(usize, 16), @sizeOf(CmdDrawRect));
    try testing.expectEqual(@as(usize, 16), @sizeOf(CmdDrawGlyph));
    try testing.expectEqual(@as(usize, 16), @sizeOf(CmdSetClip));
}

test "BATCH_BUF_SIZE is 64KB" {
    try testing.expectEqual(@as(u32, 65536), BATCH_BUF_SIZE);
}

test "addRect writes correct command type" {
    var buf: [BATCH_BUF_SIZE]u8 = [_]u8{0} ** BATCH_BUF_SIZE;
    var batch = Batch.init(&buf);
    batch.addRect(10, 20, 100, 50, 0xFF_FF0000);

    try testing.expectEqual(@as(u32, 1), batch.count);
    try testing.expectEqual(@as(u32, 16), batch.usedBytes());
    // First byte is the command type
    try testing.expectEqual(@as(u8, 0x01), buf[0]);
}

test "addGlyph writes char_code and scale" {
    var buf: [BATCH_BUF_SIZE]u8 = [_]u8{0} ** BATCH_BUF_SIZE;
    var batch = Batch.init(&buf);
    batch.addGlyph(5, 10, 'A', 2, 0xFF_FFFFFF);

    try testing.expectEqual(@as(u32, 1), batch.count);
    // cmd type at offset 0
    try testing.expectEqual(@as(u8, 0x02), buf[0]);
    // char_code at offset 2, scale at offset 3
    try testing.expectEqual(@as(u8, 'A'), buf[2]);
    try testing.expectEqual(@as(u8, 2), buf[3]);
}

test "setClip writes correct command type" {
    var buf: [BATCH_BUF_SIZE]u8 = [_]u8{0} ** BATCH_BUF_SIZE;
    var batch = Batch.init(&buf);
    batch.setClip(0, 0, 800, 600);

    try testing.expectEqual(@as(u32, 1), batch.count);
    try testing.expectEqual(@as(u8, 0x03), buf[0]);
}

test "multiple commands accumulate sequentially" {
    var buf: [BATCH_BUF_SIZE]u8 = [_]u8{0} ** BATCH_BUF_SIZE;
    var batch = Batch.init(&buf);
    batch.addRect(0, 0, 10, 10, 0xFF000000);
    batch.addGlyph(0, 0, 'X', 1, 0xFFFFFFFF);
    batch.setClip(0, 0, 100, 100);

    try testing.expectEqual(@as(u32, 3), batch.count);
    try testing.expectEqual(@as(u32, 48), batch.usedBytes());
    // Verify command types at offsets 0, 16, 32
    try testing.expectEqual(@as(u8, 0x01), buf[0]);
    try testing.expectEqual(@as(u8, 0x02), buf[16]);
    try testing.expectEqual(@as(u8, 0x03), buf[32]);
}

test "reset zeroes count" {
    var buf: [BATCH_BUF_SIZE]u8 = [_]u8{0} ** BATCH_BUF_SIZE;
    var batch = Batch.init(&buf);
    batch.addRect(0, 0, 1, 1, 0);
    batch.addRect(0, 0, 1, 1, 0);
    try testing.expectEqual(@as(u32, 2), batch.count);
    batch.reset();
    try testing.expectEqual(@as(u32, 0), batch.count);
    try testing.expectEqual(@as(u32, 0), batch.usedBytes());
}

test "isFull returns false initially and commandSlice is empty" {
    var buf: [BATCH_BUF_SIZE]u8 = [_]u8{0} ** BATCH_BUF_SIZE;
    const batch = Batch.init(&buf);
    try testing.expect(!batch.isFull());
    try testing.expectEqual(@as(usize, 0), batch.commandSlice().len);
}

test "addRect to full batch is no-op" {
    var buf: [BATCH_BUF_SIZE]u8 = [_]u8{0} ** BATCH_BUF_SIZE;
    var batch = Batch.init(&buf);
    batch.count = MAX_COMMANDS; // simulate full
    batch.addRect(0, 0, 1, 1, 0);
    try testing.expectEqual(MAX_COMMANDS, batch.count); // unchanged
    try testing.expect(batch.isFull());
}

test "CmdDrawRect field layout" {
    const cmd = CmdDrawRect{ .x = 10, .y = 20, .w = 100, .h = 50, .color = 0xAABBCCDD };
    const bytes: *const [16]u8 = @ptrCast(&cmd);
    // cmd type at byte 0
    try testing.expectEqual(@as(u8, 0x01), bytes[0]);
    // x is i16 LE at offset 4-5
    const x_val = @as(i16, @bitCast([2]u8{ bytes[4], bytes[5] }));
    try testing.expectEqual(@as(i16, 10), x_val);
}
