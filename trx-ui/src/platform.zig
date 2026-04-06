/// Platform abstraction — TRX syscall wrappers.
///
/// On real TerranoxOS, these issue syscalls via inline assembly.
/// In tests, they are no-ops / stubs.  The build system detects the
/// target: freestanding = real syscalls, native = stubs.

const builtin = @import("builtin");

/// Whether we're running on the real TerranoxOS kernel.
pub const is_trx = builtin.os.tag == .freestanding;

// ── Syscall numbers (from genesis_syscall.h) ───────────────────────

/// Syscall numbers for TerranoxOS userspace.
///
/// **ABI note:** This module targets the TerranoxOS Linux-compat syscall ABI.
/// Basic ops (EXIT, YIELD, WRITE) use Linux x86_64 numbers so the kernel's
/// `linux_to_genesis` translation layer maps them to the correct Genesis
/// shared syscalls (where Genesis 0x0000 = EXIT collides with Linux 0 = read).
/// TRX-specific syscalls (0x0100+) use Genesis-native numbers and pass
/// through the translation layer untouched.
pub const SYS = struct {
    // Basic ops — Linux x86_64 numbers for kernel compat layer.
    pub const EXIT: u64 = 60; // Linux SYS_exit → GEN_SYS_EXIT
    pub const YIELD: u64 = 24; // Linux SYS_sched_yield → GEN_SYS_YIELD
    pub const WRITE: u64 = 1; // Linux SYS_write (for debug output)
    // TRX compositor/display/input syscalls — Genesis-native numbers.
    pub const COMPOSITOR_CREATE: u64 = 0x0152;
    pub const COMPOSITOR_PRESENT: u64 = 0x0153;
    pub const SURFACE_CREATE: u64 = 0x0154;
    pub const SURFACE_DESTROY: u64 = 0x0155;
    pub const SURFACE_RESIZE: u64 = 0x0156;
    pub const BUFFER_CREATE: u64 = 0x0157;
    pub const BUFFER_MAP: u64 = 0x0158;
    pub const BUFFER_UNMAP: u64 = 0x0159;
    pub const INPUT_READ_EVENTS: u64 = 0x0163;
    // TRX GPU/DRM syscalls — Genesis-native numbers.
    pub const GPU_OPEN: u64 = 0x0170;
    pub const GPU_CLOSE: u64 = 0x0171;
    pub const GPU_ALLOC_BO: u64 = 0x0172;
    pub const GPU_FREE_BO: u64 = 0x0173;
    pub const GPU_MAP_BO: u64 = 0x0174;
    pub const GPU_SUBMIT: u64 = 0x0175;
    pub const GPU_WAIT_FENCE: u64 = 0x0176;
    pub const GPU_EXPORT_DMABUF: u64 = 0x0177;
    pub const GPU_IMPORT_DMABUF: u64 = 0x0178;
    pub const GPU_GET_INFO: u64 = 0x0179;
};

// ── GPU buffer usage flags ────────────────────────────────────────

pub const BUFFER_USAGE_GPU_RENDER: u32 = 0x01;
pub const BUFFER_USAGE_GPU_TEXTURE: u32 = 0x02;
pub const GPU_FENCE_TIMEOUT_DEFAULT_NS: u64 = 16_000_000; // 16ms (~60fps)

// ── Raw syscall interface (x86_64) ─────────────────────────────────

fn syscall0(nr: u64) i64 {
    if (!is_trx) return -1;
    return asm volatile ("syscall"
        : [ret] "={rax}" (-> i64),
        : [nr] "{rax}" (nr),
        : "rcx", "r11", "memory"
    );
}

fn syscall1(nr: u64, a1: u64) i64 {
    if (!is_trx) return -1;
    return asm volatile ("syscall"
        : [ret] "={rax}" (-> i64),
        : [nr] "{rax}" (nr),
          [a1] "{rdi}" (a1),
        : "rcx", "r11", "memory"
    );
}

fn syscall2(nr: u64, a1: u64, a2: u64) i64 {
    if (!is_trx) return -1;
    return asm volatile ("syscall"
        : [ret] "={rax}" (-> i64),
        : [nr] "{rax}" (nr),
          [a1] "{rdi}" (a1),
          [a2] "{rsi}" (a2),
        : "rcx", "r11", "memory"
    );
}

fn syscall3(nr: u64, a1: u64, a2: u64, a3: u64) i64 {
    if (!is_trx) return -1;
    return asm volatile ("syscall"
        : [ret] "={rax}" (-> i64),
        : [nr] "{rax}" (nr),
          [a1] "{rdi}" (a1),
          [a2] "{rsi}" (a2),
          [a3] "{rdx}" (a3),
        : "rcx", "r11", "memory"
    );
}

fn syscall4(nr: u64, a1: u64, a2: u64, a3: u64, a4: u64) i64 {
    if (!is_trx) return -1;
    return asm volatile ("syscall"
        : [ret] "={rax}" (-> i64),
        : [nr] "{rax}" (nr),
          [a1] "{rdi}" (a1),
          [a2] "{rsi}" (a2),
          [a3] "{rdx}" (a3),
          [a4] "{r10}" (a4),
        : "rcx", "r11", "memory"
    );
}

fn syscall6(nr: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64, a6: u64) i64 {
    if (!is_trx) return -1;
    return asm volatile ("syscall"
        : [ret] "={rax}" (-> i64),
        : [nr] "{rax}" (nr),
          [a1] "{rdi}" (a1),
          [a2] "{rsi}" (a2),
          [a3] "{rdx}" (a3),
          [a4] "{r10}" (a4),
          [a5] "{r8}" (a5),
          [a6] "{r9}" (a6),
        : "rcx", "r11", "memory"
    );
}

// ── High-level wrappers ────────────────────────────────────────────

pub const PlatformError = error{
    SyscallFailed,
};

/// Create a compositor session. Returns compositor handle.
pub fn compositorCreate() PlatformError!i64 {
    const ret = syscall0(SYS.COMPOSITOR_CREATE);
    if (ret < 0) return error.SyscallFailed;
    return ret;
}

/// Create a surface. Returns surface handle.
pub fn surfaceCreate(compositor: i64, width: u32, height: u32) PlatformError!i64 {
    const ret = syscall3(
        SYS.SURFACE_CREATE,
        @bitCast(compositor),
        @as(u64, width),
        @as(u64, height),
    );
    if (ret < 0) return error.SyscallFailed;
    return ret;
}

/// Destroy a surface.
pub fn surfaceDestroy(surface: i64) void {
    _ = syscall1(SYS.SURFACE_DESTROY, @bitCast(surface));
}

/// Resize a surface.
pub fn surfaceResize(surface: i64, width: u32, height: u32) PlatformError!void {
    const ret = syscall3(
        SYS.SURFACE_RESIZE,
        @bitCast(surface),
        @as(u64, width),
        @as(u64, height),
    );
    if (ret < 0) return error.SyscallFailed;
}

/// Create a pixel buffer. Returns buffer handle.
pub fn bufferCreate(size: u64) PlatformError!i64 {
    const ret = syscall1(SYS.BUFFER_CREATE, size);
    if (ret < 0) return error.SyscallFailed;
    return ret;
}

/// Map a buffer into the process address space. Returns mapped pointer.
pub fn bufferMap(buffer: i64) PlatformError![*]u32 {
    const ret = syscall1(SYS.BUFFER_MAP, @bitCast(buffer));
    if (ret < 0) return error.SyscallFailed;
    return @ptrFromInt(@as(usize, @bitCast(ret)));
}

/// Unmap a previously mapped buffer.
pub fn bufferUnmap(buffer: i64) void {
    _ = syscall1(SYS.BUFFER_UNMAP, @bitCast(buffer));
}

/// Present a frame (submit layers to compositor).
pub fn compositorPresent(compositor: i64, surface: i64, buffer: i64) PlatformError!void {
    const ret = syscall3(
        SYS.COMPOSITOR_PRESENT,
        @bitCast(compositor),
        @bitCast(surface),
        @bitCast(buffer),
    );
    if (ret < 0) return error.SyscallFailed;
}

/// Read input events into a caller-provided buffer.
/// Returns number of events read.
pub fn inputReadEvents(buf: [*]InputEventRaw, max_count: u32) i64 {
    return syscall2(SYS.INPUT_READ_EVENTS, @intFromPtr(buf), @as(u64, max_count));
}

/// Yield the CPU timeslice.
pub fn yield() void {
    _ = syscall0(SYS.YIELD);
}

/// Exit the process.
pub fn exit(code: i32) noreturn {
    _ = syscall1(SYS.EXIT, @as(u64, @bitCast(@as(i64, code))));
    unreachable;
}

// ── GPU wrappers ──────────────────────────────────────────────────

/// Open the GPU device. Returns device handle.
pub fn gpuOpen() PlatformError!i64 {
    const ret = syscall0(SYS.GPU_OPEN);
    if (ret < 0) return error.SyscallFailed;
    return ret;
}

/// Close the GPU device.
pub fn gpuClose(gpu: i64) void {
    _ = syscall1(SYS.GPU_CLOSE, @bitCast(gpu));
}

/// Allocate a GPU buffer object. Returns BO handle.
pub fn gpuAllocBo(gpu: i64, size: u64, usage: u32) PlatformError!i64 {
    const ret = syscall3(SYS.GPU_ALLOC_BO, @bitCast(gpu), size, @as(u64, usage));
    if (ret < 0) return error.SyscallFailed;
    return ret;
}

/// Free a GPU buffer object.
pub fn gpuFreeBo(gpu: i64, bo: i64) void {
    _ = syscall2(SYS.GPU_FREE_BO, @bitCast(gpu), @bitCast(bo));
}

/// Map a GPU buffer object into the process address space. Returns mapped pointer.
pub fn gpuMapBo(gpu: i64, bo: i64) PlatformError![*]u8 {
    const ret = syscall2(SYS.GPU_MAP_BO, @bitCast(gpu), @bitCast(bo));
    if (ret < 0) return error.SyscallFailed;
    return @ptrFromInt(@as(usize, @bitCast(ret)));
}

/// Submit a command buffer to the GPU. Returns fence handle.
pub fn gpuSubmit(gpu: i64, cmd_buf: [*]const u8, cmd_len: u32) PlatformError!i64 {
    const ret = syscall3(SYS.GPU_SUBMIT, @bitCast(gpu), @intFromPtr(cmd_buf), @as(u64, cmd_len));
    if (ret < 0) return error.SyscallFailed;
    return ret;
}

/// Wait for a GPU fence to signal. Times out after timeout_ns nanoseconds.
pub fn gpuWaitFence(gpu: i64, fence: i64, timeout_ns: u64) PlatformError!void {
    const ret = syscall3(SYS.GPU_WAIT_FENCE, @bitCast(gpu), @bitCast(fence), timeout_ns);
    if (ret < 0) return error.SyscallFailed;
}

/// Query GPU device information into a caller-provided buffer.
/// Returns number of bytes written.
pub fn gpuGetInfo(gpu: i64, info_buf: [*]u8, buf_len: u32) PlatformError!i64 {
    const ret = syscall3(SYS.GPU_GET_INFO, @bitCast(gpu), @intFromPtr(info_buf), @as(u64, buf_len));
    if (ret < 0) return error.SyscallFailed;
    return ret;
}

/// Raw input event struct matching GenTrxInputEvent layout.
pub const InputEventRaw = extern struct {
    timestamp_ns: u64,
    type_: u32,
    code: u32,
    value: i32,
    device_id: u32,
};

// ── Tests ──────────────────────────────────────────────────────────

const testing = @import("std").testing;

test "basic ops use Linux x86_64 ABI numbers for kernel compat layer" {
    try testing.expectEqual(@as(u64, 60), SYS.EXIT);
    try testing.expectEqual(@as(u64, 24), SYS.YIELD);
    try testing.expectEqual(@as(u64, 1), SYS.WRITE);
}

test "TRX compositor/display/input syscalls use Genesis-native numbers" {
    try testing.expectEqual(@as(u64, 0x0152), SYS.COMPOSITOR_CREATE);
    try testing.expectEqual(@as(u64, 0x0153), SYS.COMPOSITOR_PRESENT);
    try testing.expectEqual(@as(u64, 0x0154), SYS.SURFACE_CREATE);
    try testing.expectEqual(@as(u64, 0x0155), SYS.SURFACE_DESTROY);
    try testing.expectEqual(@as(u64, 0x0156), SYS.SURFACE_RESIZE);
    try testing.expectEqual(@as(u64, 0x0157), SYS.BUFFER_CREATE);
    try testing.expectEqual(@as(u64, 0x0158), SYS.BUFFER_MAP);
    try testing.expectEqual(@as(u64, 0x0159), SYS.BUFFER_UNMAP);
    try testing.expectEqual(@as(u64, 0x0163), SYS.INPUT_READ_EVENTS);
}

test "TRX GPU/DRM syscalls use Genesis-native numbers" {
    try testing.expectEqual(@as(u64, 0x0170), SYS.GPU_OPEN);
    try testing.expectEqual(@as(u64, 0x0171), SYS.GPU_CLOSE);
    try testing.expectEqual(@as(u64, 0x0172), SYS.GPU_ALLOC_BO);
    try testing.expectEqual(@as(u64, 0x0173), SYS.GPU_FREE_BO);
    try testing.expectEqual(@as(u64, 0x0174), SYS.GPU_MAP_BO);
    try testing.expectEqual(@as(u64, 0x0175), SYS.GPU_SUBMIT);
    try testing.expectEqual(@as(u64, 0x0176), SYS.GPU_WAIT_FENCE);
    try testing.expectEqual(@as(u64, 0x0177), SYS.GPU_EXPORT_DMABUF);
    try testing.expectEqual(@as(u64, 0x0178), SYS.GPU_IMPORT_DMABUF);
    try testing.expectEqual(@as(u64, 0x0179), SYS.GPU_GET_INFO);
}

test "non-trx GPU wrappers return error" {
    if (!is_trx) {
        try testing.expectError(error.SyscallFailed, gpuOpen());
    }
}

test "InputEventRaw has correct layout" {
    try testing.expectEqual(@as(usize, 24), @sizeOf(InputEventRaw));
}

test "non-trx syscalls return -1" {
    if (!is_trx) {
        const ret = syscall0(SYS.YIELD);
        try testing.expectEqual(@as(i64, -1), ret);
    }
}
