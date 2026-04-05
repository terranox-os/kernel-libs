/// Platform abstraction — TRX syscall wrappers.
///
/// On real TerranoxOS, these issue syscalls via inline assembly.
/// In tests, they are no-ops / stubs.  The build system detects the
/// target: freestanding = real syscalls, native = stubs.

const builtin = @import("builtin");

/// Whether we're running on the real TerranoxOS kernel.
pub const is_trx = builtin.os.tag == .freestanding;

// ── Syscall numbers (from genesis_syscall.h) ───────────────────────

pub const SYS = struct {
    pub const EXIT: u64 = 0x0000;
    pub const YIELD: u64 = 0x0005;
    pub const COMPOSITOR_CREATE: u64 = 0x0152;
    pub const COMPOSITOR_PRESENT: u64 = 0x0153;
    pub const SURFACE_CREATE: u64 = 0x0154;
    pub const SURFACE_DESTROY: u64 = 0x0155;
    pub const SURFACE_RESIZE: u64 = 0x0156;
    pub const BUFFER_CREATE: u64 = 0x0157;
    pub const BUFFER_MAP: u64 = 0x0158;
    pub const BUFFER_UNMAP: u64 = 0x0159;
    pub const INPUT_READ_EVENTS: u64 = 0x0163;
};

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

test "SYS constants match genesis_syscall.h" {
    try testing.expectEqual(@as(u64, 0x0154), SYS.SURFACE_CREATE);
    try testing.expectEqual(@as(u64, 0x0153), SYS.COMPOSITOR_PRESENT);
    try testing.expectEqual(@as(u64, 0x0163), SYS.INPUT_READ_EVENTS);
    try testing.expectEqual(@as(u64, 0x0005), SYS.YIELD);
    try testing.expectEqual(@as(u64, 0x0000), SYS.EXIT);
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
