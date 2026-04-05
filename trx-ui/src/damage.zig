/// Dirty-region tracker for incremental redraw.
///
/// Tracks up to MAX_RECTS damaged rectangles per frame.  The render
/// loop queries dirty regions and only redraws those areas.  If the
/// tracker overflows, it collapses to a single full-screen rect.

pub const MAX_RECTS: usize = 64;

pub const Rect = struct {
    x: i32,
    y: i32,
    w: u32,
    h: u32,

    pub fn intersects(a: Rect, b: Rect) bool {
        return a.x < b.x + @as(i32, @intCast(b.w)) and
            a.x + @as(i32, @intCast(a.w)) > b.x and
            a.y < b.y + @as(i32, @intCast(b.h)) and
            a.y + @as(i32, @intCast(a.h)) > b.y;
    }

    pub fn union_(a: Rect, b: Rect) Rect {
        const x0 = @min(a.x, b.x);
        const y0 = @min(a.y, b.y);
        const x1 = @max(a.x + @as(i32, @intCast(a.w)), b.x + @as(i32, @intCast(b.w)));
        const y1 = @max(a.y + @as(i32, @intCast(a.h)), b.y + @as(i32, @intCast(b.h)));
        return .{
            .x = x0,
            .y = y0,
            .w = @intCast(x1 - x0),
            .h = @intCast(y1 - y0),
        };
    }

    pub fn area(self: Rect) u64 {
        return @as(u64, self.w) * @as(u64, self.h);
    }
};

pub const DamageTracker = struct {
    rects: [MAX_RECTS]Rect = undefined,
    count: usize = 0,
    screen_w: u32,
    screen_h: u32,
    full_damage: bool = false,

    pub fn init(screen_w: u32, screen_h: u32) DamageTracker {
        return .{ .screen_w = screen_w, .screen_h = screen_h };
    }

    /// Mark the entire screen as dirty.
    pub fn damageAll(self: *DamageTracker) void {
        self.full_damage = true;
        self.count = 1;
        self.rects[0] = .{ .x = 0, .y = 0, .w = self.screen_w, .h = self.screen_h };
    }

    /// Mark a rectangle as dirty.
    pub fn addRect(self: *DamageTracker, r: Rect) void {
        if (self.full_damage) return;

        // Try to merge with existing overlapping rect
        for (0..self.count) |i| {
            if (self.rects[i].intersects(r)) {
                self.rects[i] = self.rects[i].union_(r);
                return;
            }
        }

        if (self.count < MAX_RECTS) {
            self.rects[self.count] = r;
            self.count += 1;
        } else {
            // Overflow — collapse to full screen
            self.damageAll();
        }
    }

    /// Get the current dirty rectangles.
    pub fn getDirtyRects(self: *const DamageTracker) []const Rect {
        return self.rects[0..self.count];
    }

    /// Reset for next frame.
    pub fn reset(self: *DamageTracker) void {
        self.count = 0;
        self.full_damage = false;
    }

    /// Check if anything is dirty.
    pub fn isDirty(self: *const DamageTracker) bool {
        return self.count > 0;
    }
};

// ── Tests ──────────────────────────────────────────────────────────

const testing = @import("std").testing;

test "empty tracker has no dirty rects" {
    const dt = DamageTracker.init(800, 600);
    try testing.expectEqual(@as(usize, 0), dt.getDirtyRects().len);
    try testing.expect(!dt.isDirty());
}

test "addRect tracks single rect" {
    var dt = DamageTracker.init(800, 600);
    dt.addRect(.{ .x = 10, .y = 20, .w = 100, .h = 50 });
    try testing.expectEqual(@as(usize, 1), dt.getDirtyRects().len);
    try testing.expect(dt.isDirty());
}

test "overlapping rects merge" {
    var dt = DamageTracker.init(800, 600);
    dt.addRect(.{ .x = 0, .y = 0, .w = 50, .h = 50 });
    dt.addRect(.{ .x = 25, .y = 25, .w = 50, .h = 50 });
    // Should merge into one rect
    try testing.expectEqual(@as(usize, 1), dt.getDirtyRects().len);
    const r = dt.getDirtyRects()[0];
    try testing.expectEqual(@as(i32, 0), r.x);
    try testing.expectEqual(@as(i32, 0), r.y);
    try testing.expectEqual(@as(u32, 75), r.w);
    try testing.expectEqual(@as(u32, 75), r.h);
}

test "non-overlapping rects stay separate" {
    var dt = DamageTracker.init(800, 600);
    dt.addRect(.{ .x = 0, .y = 0, .w = 10, .h = 10 });
    dt.addRect(.{ .x = 100, .y = 100, .w = 10, .h = 10 });
    try testing.expectEqual(@as(usize, 2), dt.getDirtyRects().len);
}

test "damageAll collapses to full screen" {
    var dt = DamageTracker.init(800, 600);
    dt.addRect(.{ .x = 0, .y = 0, .w = 10, .h = 10 });
    dt.damageAll();
    try testing.expectEqual(@as(usize, 1), dt.getDirtyRects().len);
    try testing.expectEqual(@as(u32, 800), dt.getDirtyRects()[0].w);
}

test "reset clears damage" {
    var dt = DamageTracker.init(800, 600);
    dt.addRect(.{ .x = 0, .y = 0, .w = 10, .h = 10 });
    dt.reset();
    try testing.expect(!dt.isDirty());
}

test "overflow collapses to full screen" {
    var dt = DamageTracker.init(800, 600);
    // Add MAX_RECTS non-overlapping rects
    for (0..MAX_RECTS) |i| {
        dt.addRect(.{
            .x = @intCast(i * 10),
            .y = 0,
            .w = 5,
            .h = 5,
        });
    }
    try testing.expectEqual(@as(usize, MAX_RECTS), dt.count);
    // One more should collapse
    dt.addRect(.{ .x = 700, .y = 500, .w = 5, .h = 5 });
    try testing.expect(dt.full_damage);
    try testing.expectEqual(@as(usize, 1), dt.count);
}

test "rect intersects" {
    const a = Rect{ .x = 0, .y = 0, .w = 50, .h = 50 };
    const b = Rect{ .x = 25, .y = 25, .w = 50, .h = 50 };
    const c = Rect{ .x = 100, .y = 100, .w = 10, .h = 10 };
    try testing.expect(a.intersects(b));
    try testing.expect(!a.intersects(c));
}
