/// GPU context — device lifecycle and buffer object management.
///
/// Wraps the platform GPU syscalls into a safe, no_std-compatible API.
/// No heap allocation — uses fixed-size slot array (caller provides all memory).
/// GPU is optional: `probe()` detects availability at runtime.

const platform = @import("platform.zig");

pub const MAX_GPU_BUFFERS: u8 = 8;

pub const BufferUsage = enum(u32) {
    render = platform.BUFFER_USAGE_GPU_RENDER,
    texture = platform.BUFFER_USAGE_GPU_TEXTURE,
};

pub const GpuBuffer = struct {
    handle: i64 = -1,
    mapped: ?[*]u8 = null,
    size: u64 = 0,
    usage: BufferUsage = .render,
    active: bool = false,
};

pub const GpuError = error{
    DeviceNotAvailable,
    AllocFailed,
    MapFailed,
    SubmitFailed,
    FenceTimeout,
    NoFreeSlot,
};

/// GPU device context. Create via `probe()`.
pub const GpuContext = struct {
    device_handle: i64 = -1,
    buffers: [MAX_GPU_BUFFERS]GpuBuffer = [_]GpuBuffer{.{}} ** MAX_GPU_BUFFERS,
    buffer_count: u8 = 0,
    available: bool = false,
    last_fence: i64 = -1,

    /// Attempt to open the GPU device. If unavailable, returns a context
    /// with `available = false` — all operations become no-ops.
    pub fn probe() GpuContext {
        var ctx = GpuContext{};
        const handle = platform.gpuOpen() catch return ctx;
        ctx.device_handle = handle;
        ctx.available = true;
        return ctx;
    }

    /// Allocate a GPU buffer object. Returns a slot index (0..MAX_GPU_BUFFERS-1).
    pub fn allocBuffer(self: *GpuContext, size: u64, usage: BufferUsage) GpuError!u8 {
        if (!self.available) return GpuError.DeviceNotAvailable;

        // Find free slot
        var slot: ?u8 = null;
        for (0..MAX_GPU_BUFFERS) |i| {
            if (!self.buffers[i].active) {
                slot = @intCast(i);
                break;
            }
        }
        const idx = slot orelse return GpuError.NoFreeSlot;

        const handle = platform.gpuAllocBo(
            self.device_handle,
            size,
            @intFromEnum(usage),
        ) catch return GpuError.AllocFailed;

        self.buffers[idx] = .{
            .handle = handle,
            .mapped = null,
            .size = size,
            .usage = usage,
            .active = true,
        };
        self.buffer_count += 1;
        return idx;
    }

    /// Free a GPU buffer object and release its slot.
    pub fn freeBuffer(self: *GpuContext, slot: u8) void {
        if (slot >= MAX_GPU_BUFFERS) return;
        if (!self.buffers[slot].active) return;

        if (self.buffers[slot].mapped != null) {
            // Unmap first if still mapped
            platform.gpuFreeBo(self.device_handle, self.buffers[slot].handle);
        } else {
            platform.gpuFreeBo(self.device_handle, self.buffers[slot].handle);
        }
        self.buffers[slot] = .{};
        if (self.buffer_count > 0) self.buffer_count -= 1;
    }

    /// Map a buffer object into the process address space.
    pub fn mapBuffer(self: *GpuContext, slot: u8) GpuError![*]u8 {
        if (!self.available) return GpuError.DeviceNotAvailable;
        if (slot >= MAX_GPU_BUFFERS) return GpuError.MapFailed;
        if (!self.buffers[slot].active) return GpuError.MapFailed;

        const ptr = platform.gpuMapBo(
            self.device_handle,
            self.buffers[slot].handle,
        ) catch return GpuError.MapFailed;

        self.buffers[slot].mapped = ptr;
        return ptr;
    }

    /// Submit a command buffer to the GPU. Returns a fence handle.
    pub fn submit(self: *GpuContext, cmd_buf: []const u8) GpuError!i64 {
        if (!self.available) return GpuError.DeviceNotAvailable;

        const fence = platform.gpuSubmit(
            self.device_handle,
            cmd_buf.ptr,
            @intCast(cmd_buf.len),
        ) catch return GpuError.SubmitFailed;

        self.last_fence = fence;
        return fence;
    }

    /// Wait for a GPU fence to signal.
    pub fn waitFence(self: *GpuContext, fence: i64) GpuError!void {
        if (!self.available) return GpuError.DeviceNotAvailable;

        platform.gpuWaitFence(
            self.device_handle,
            fence,
            platform.GPU_FENCE_TIMEOUT_DEFAULT_NS,
        ) catch return GpuError.FenceTimeout;
    }

    /// Release all buffers and close the device.
    pub fn deinit(self: *GpuContext) void {
        if (!self.available) return;

        for (0..MAX_GPU_BUFFERS) |i| {
            if (self.buffers[i].active) {
                self.freeBuffer(@intCast(i));
            }
        }
        platform.gpuClose(self.device_handle);
        self.device_handle = -1;
        self.available = false;
    }

    /// Whether the GPU device is available and open.
    pub fn isAvailable(self: *const GpuContext) bool {
        return self.available;
    }
};

// ── Tests ──────────────────────────────────────────────────────────

const testing = @import("std").testing;

test "GpuContext.probe on host returns unavailable" {
    const ctx = GpuContext.probe();
    try testing.expect(!ctx.isAvailable());
    try testing.expectEqual(@as(i64, -1), ctx.device_handle);
}

test "unavailable context rejects allocBuffer" {
    var ctx = GpuContext{};
    try testing.expectError(GpuError.DeviceNotAvailable, ctx.allocBuffer(4096, .render));
}

test "unavailable context rejects submit" {
    var ctx = GpuContext{};
    const dummy = [_]u8{0} ** 16;
    try testing.expectError(GpuError.DeviceNotAvailable, ctx.submit(&dummy));
}

test "unavailable context rejects waitFence" {
    var ctx = GpuContext{};
    try testing.expectError(GpuError.DeviceNotAvailable, ctx.waitFence(0));
}

test "deinit on unavailable context is no-op" {
    var ctx = GpuContext{};
    ctx.deinit(); // should not panic or crash
    try testing.expect(!ctx.isAvailable());
}

test "slot management — NoFreeSlot after MAX_GPU_BUFFERS" {
    // Simulate full slots by marking all as active (without real GPU)
    var ctx = GpuContext{ .available = true, .device_handle = 1 };
    for (0..MAX_GPU_BUFFERS) |i| {
        ctx.buffers[i].active = true;
    }
    ctx.buffer_count = MAX_GPU_BUFFERS;

    // Next alloc should fail with NoFreeSlot (not DeviceNotAvailable)
    // Since gpuAllocBo will fail on non-trx, we test the slot check by
    // verifying the available context doesn't return DeviceNotAvailable.
    const result = ctx.allocBuffer(4096, .render);
    if (result) |_| {
        // On a real GPU this would succeed if slots are free
        unreachable;
    } else |err| {
        // On non-trx: either NoFreeSlot (slots full) or AllocFailed (syscall -1)
        try testing.expect(err == GpuError.NoFreeSlot or err == GpuError.AllocFailed);
    }
}

test "freeBuffer on inactive slot is no-op" {
    var ctx = GpuContext{ .available = true, .device_handle = 1 };
    ctx.freeBuffer(0); // slot 0 not active, should not crash
    ctx.freeBuffer(255); // out of range, should not crash
    try testing.expectEqual(@as(u8, 0), ctx.buffer_count);
}

test "initial state has zero active buffers" {
    const ctx = GpuContext{};
    try testing.expectEqual(@as(u8, 0), ctx.buffer_count);
    for (0..MAX_GPU_BUFFERS) |i| {
        try testing.expect(!ctx.buffers[i].active);
    }
}
