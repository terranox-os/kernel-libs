//! Freestanding ABI types for TerranoxOS, GenesisOS-RT, and HermeticaOS.
//!
//! This crate mirrors the C headers in `include/`. CI checks must ensure
//! the Rust definitions do not drift from the C source of truth.

#![no_std]
#![forbid(unsafe_op_in_unsafe_fn)]

// ────────────────────────────────────────────────────────────
// GenResult — mirrors genesis_result.h
// ────────────────────────────────────────────────────────────

/// Universal kernel result type. 0 = success, negative = error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct GenResult(pub i32);

impl GenResult {
    pub const OK: Self = Self(0);

    // General errors (-1 to -15)
    pub const ERR_INVALID_ARG: Self = Self(-1);
    pub const ERR_OUT_OF_MEMORY: Self = Self(-2);
    pub const ERR_NOT_FOUND: Self = Self(-3);
    pub const ERR_ALREADY_EXISTS: Self = Self(-4);
    pub const ERR_BUFFER_TOO_SMALL: Self = Self(-5);
    pub const ERR_NOT_SUPPORTED: Self = Self(-6);
    pub const ERR_BUSY: Self = Self(-7);
    pub const ERR_TIMEOUT: Self = Self(-8);
    pub const ERR_INTERRUPTED: Self = Self(-9);
    pub const ERR_OVERFLOW: Self = Self(-10);

    // Permission / security errors (-16 to -31)
    pub const ERR_PERMISSION_DENIED: Self = Self(-16);
    pub const ERR_ACCESS_VIOLATION: Self = Self(-17);
    pub const ERR_INVALID_CAPABILITY: Self = Self(-18);

    // I/O and hardware errors (-32 to -47)
    pub const ERR_IO: Self = Self(-32);
    pub const ERR_DEVICE_OFFLINE: Self = Self(-33);
    pub const ERR_BAD_ADDRESS: Self = Self(-34);

    // Format / parse errors (-48 to -63)
    pub const ERR_INVALID_FORMAT: Self = Self(-48);
    pub const ERR_CHECKSUM_MISMATCH: Self = Self(-49);
    pub const ERR_VERSION_MISMATCH: Self = Self(-50);

    // Module errors (-64 to -79)
    pub const ERR_MODULE_LOAD_FAILED: Self = Self(-64);
    pub const ERR_MODULE_INIT_FAILED: Self = Self(-65);
    pub const ERR_MODULE_NOT_FOUND: Self = Self(-66);
    pub const ERR_MODULE_INCOMPATIBLE: Self = Self(-67);

    // RT errors (-80 to -95) — GenesisOS-RT
    pub const ERR_DEADLINE_MISS: Self = Self(-80);
    pub const ERR_PRIORITY_INV: Self = Self(-81);
    pub const ERR_STACK_OVERFLOW: Self = Self(-82);

    // Syscall errors (-96 to -111) — TerranoxOS, HermeticaOS
    pub const ERR_BAD_SYSCALL: Self = Self(-96);
    pub const ERR_BAD_HANDLE: Self = Self(-97);
    pub const ERR_SYSCALL_INTERRUPTED: Self = Self(-98);
    pub const ERR_HANDLE_LIMIT: Self = Self(-99);

    // TerranoxOS I/O extension errors (-35 to -47)
    pub const ERR_CHANNEL_CLOSED: Self = Self(-35);
    pub const ERR_DISPLAY_OFFLINE: Self = Self(-36);
    pub const ERR_GPU_ERROR: Self = Self(-37);

    #[inline]
    pub const fn is_ok(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub const fn is_error(self) -> bool {
        self.0 < 0
    }

    /// Returns the error name as a static string slice.
    #[cfg(feature = "result-names")]
    pub const fn name(self) -> &'static str {
        match self.0 {
            0 => "OK",
            -1 => "INVALID_ARG",
            -2 => "OUT_OF_MEMORY",
            -3 => "NOT_FOUND",
            -4 => "ALREADY_EXISTS",
            -5 => "BUFFER_TOO_SMALL",
            -6 => "NOT_SUPPORTED",
            -7 => "BUSY",
            -8 => "TIMEOUT",
            -9 => "INTERRUPTED",
            -10 => "OVERFLOW",
            -16 => "PERMISSION_DENIED",
            -17 => "ACCESS_VIOLATION",
            -18 => "INVALID_CAPABILITY",
            -32 => "IO",
            -33 => "DEVICE_OFFLINE",
            -34 => "BAD_ADDRESS",
            -48 => "INVALID_FORMAT",
            -49 => "CHECKSUM_MISMATCH",
            -50 => "VERSION_MISMATCH",
            -64 => "MODULE_LOAD_FAILED",
            -65 => "MODULE_INIT_FAILED",
            -66 => "MODULE_NOT_FOUND",
            -67 => "MODULE_INCOMPATIBLE",
            -80 => "DEADLINE_MISS",
            -81 => "PRIORITY_INV",
            -82 => "STACK_OVERFLOW",
            -96 => "BAD_SYSCALL",
            -97 => "BAD_HANDLE",
            -98 => "SYSCALL_INTERRUPTED",
            -99 => "HANDLE_LIMIT",
            -35 => "CHANNEL_CLOSED",
            -36 => "DISPLAY_OFFLINE",
            -37 => "GPU_ERROR",
            _ => "UNKNOWN",
        }
    }
}

/// POSIX errno constants for syscall boundary translation.
pub mod posix_errno {
    pub const EPERM: i32 = 1;
    pub const ENOENT: i32 = 2;
    pub const ESRCH: i32 = 3;
    pub const EBADF: i32 = 9;
    pub const EAGAIN: i32 = 11;
    pub const ENOMEM: i32 = 12;
    pub const EACCES: i32 = 13;
    pub const EFAULT: i32 = 14;
    pub const EBUSY: i32 = 16;
    pub const EEXIST: i32 = 17;
    pub const EINVAL: i32 = 22;
    pub const EPIPE: i32 = 32;
    pub const ENOSYS: i32 = 38;
    pub const ETIMEDOUT: i32 = 110;
}

impl GenResult {
    /// Convert a GenResult to a POSIX errno value (0 = success, >0 = error).
    pub const fn to_errno(self) -> i32 {
        match self.0 {
            0 => 0,
            -1 => posix_errno::EINVAL,      // INVALID_ARG
            -2 => posix_errno::ENOMEM,       // OUT_OF_MEMORY
            -3 => posix_errno::ENOENT,       // NOT_FOUND
            -4 => posix_errno::EEXIST,       // ALREADY_EXISTS
            -5 => posix_errno::EINVAL,       // BUFFER_TOO_SMALL
            -6 => posix_errno::ENOSYS,       // NOT_SUPPORTED
            -7 => posix_errno::EBUSY,        // BUSY
            -8 => posix_errno::ETIMEDOUT,    // TIMEOUT
            -9 => posix_errno::EAGAIN,       // INTERRUPTED
            -10 => posix_errno::EINVAL,      // OVERFLOW
            -16 => posix_errno::EPERM,       // PERMISSION_DENIED
            -17 => posix_errno::EACCES,      // ACCESS_VIOLATION
            -18 => posix_errno::EPERM,       // INVALID_CAPABILITY
            -32 => posix_errno::EPIPE,       // IO
            -34 => posix_errno::EFAULT,      // BAD_ADDRESS
            -35 => posix_errno::EPIPE,       // CHANNEL_CLOSED
            -96 => posix_errno::ENOSYS,      // BAD_SYSCALL
            -97 => posix_errno::EBADF,       // BAD_HANDLE
            -98 => posix_errno::EAGAIN,      // SYSCALL_INTERRUPTED
            _ => posix_errno::EINVAL,
        }
    }

    /// Convert a POSIX errno value to a GenResult.
    pub const fn from_errno(e: i32) -> Self {
        match e {
            0 => Self::OK,
            1 => Self::ERR_PERMISSION_DENIED,     // EPERM
            2 => Self::ERR_NOT_FOUND,              // ENOENT
            3 => Self::ERR_NOT_FOUND,              // ESRCH
            9 => Self::ERR_BAD_HANDLE,             // EBADF
            11 => Self::ERR_INTERRUPTED,           // EAGAIN
            12 => Self::ERR_OUT_OF_MEMORY,         // ENOMEM
            13 => Self::ERR_ACCESS_VIOLATION,      // EACCES
            14 => Self::ERR_BAD_ADDRESS,           // EFAULT
            16 => Self::ERR_BUSY,                  // EBUSY
            17 => Self::ERR_ALREADY_EXISTS,        // EEXIST
            22 => Self::ERR_INVALID_ARG,           // EINVAL
            32 => Self::ERR_CHANNEL_CLOSED,        // EPIPE
            38 => Self::ERR_BAD_SYSCALL,           // ENOSYS
            110 => Self::ERR_TIMEOUT,              // ETIMEDOUT
            _ => Self::ERR_NOT_SUPPORTED,
        }
    }
}

// ────────────────────────────────────────────────────────────
// GenSyscallNr — mirrors genesis_syscall.h
// ────────────────────────────────────────────────────────────

/// Syscall number type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct GenSyscallNr(pub u32);

pub mod syscall {
    use super::GenSyscallNr;

    // Range bases
    pub const SHARED_BASE: GenSyscallNr = GenSyscallNr(0x0000);
    pub const TERRANOX_BASE: GenSyscallNr = GenSyscallNr(0x0100);
    pub const GENESISRT_BASE: GenSyscallNr = GenSyscallNr(0x0200);
    pub const HERMETICA_BASE: GenSyscallNr = GenSyscallNr(0x0300);

    // Range limits (exclusive)
    pub const SHARED_LIMIT: GenSyscallNr = GenSyscallNr(0x0100);
    pub const TERRANOX_LIMIT: GenSyscallNr = GenSyscallNr(0x0200);
    pub const GENESISRT_LIMIT: GenSyscallNr = GenSyscallNr(0x0300);
    pub const HERMETICA_LIMIT: GenSyscallNr = GenSyscallNr(0x0400);

    // Shared syscalls (0x0000–0x00FF)
    pub const SYS_EXIT: GenSyscallNr = GenSyscallNr(0x0000);
    pub const SYS_WRITE: GenSyscallNr = GenSyscallNr(0x0001);
    pub const SYS_READ: GenSyscallNr = GenSyscallNr(0x0002);
    pub const SYS_MMAP: GenSyscallNr = GenSyscallNr(0x0003);
    pub const SYS_MUNMAP: GenSyscallNr = GenSyscallNr(0x0004);
    pub const SYS_YIELD: GenSyscallNr = GenSyscallNr(0x0005);
    pub const SYS_GETPID: GenSyscallNr = GenSyscallNr(0x0006);
    pub const SYS_SLEEP: GenSyscallNr = GenSyscallNr(0x0007);
    pub const SYS_CLOCK_GETTIME: GenSyscallNr = GenSyscallNr(0x0008);
    pub const SYS_OPEN: GenSyscallNr = GenSyscallNr(0x0009);
    pub const SYS_CLOSE: GenSyscallNr = GenSyscallNr(0x000A);
    pub const SYS_STAT: GenSyscallNr = GenSyscallNr(0x000B);
    pub const SYS_FSTAT: GenSyscallNr = GenSyscallNr(0x000C);
    pub const SYS_LSEEK: GenSyscallNr = GenSyscallNr(0x000D);
    pub const SYS_BRK: GenSyscallNr = GenSyscallNr(0x000E);
    pub const SYS_IOCTL: GenSyscallNr = GenSyscallNr(0x000F);
    pub const SYS_DUP2: GenSyscallNr = GenSyscallNr(0x0010);
    pub const SYS_PIPE: GenSyscallNr = GenSyscallNr(0x0011);
    pub const SYS_FORK: GenSyscallNr = GenSyscallNr(0x0012);
    pub const SYS_EXEC: GenSyscallNr = GenSyscallNr(0x0013);
    pub const SYS_WAIT: GenSyscallNr = GenSyscallNr(0x0014);
    pub const SYS_FCNTL: GenSyscallNr = GenSyscallNr(0x0015);
    pub const SYS_POLL: GenSyscallNr = GenSyscallNr(0x0016);

    // TerranoxOS syscalls (0x0100–0x01FF) — organized by subsystem

    // Subsystem 0: Process management (0x0100–0x010F)
    pub const SYS_TRX_PROCESS_CREATE: GenSyscallNr = GenSyscallNr(0x0100);
    pub const SYS_TRX_PROCESS_KILL: GenSyscallNr = GenSyscallNr(0x0103);
    pub const SYS_TRX_PROCESS_INFO: GenSyscallNr = GenSyscallNr(0x0104);
    pub const SYS_TRX_PROCESS_CAP_GRANT: GenSyscallNr = GenSyscallNr(0x0105);
    pub const SYS_TRX_PROCESS_CAP_REVOKE: GenSyscallNr = GenSyscallNr(0x0106);
    pub const SYS_TRX_PROCESS_CAP_QUERY: GenSyscallNr = GenSyscallNr(0x0107);

    // Subsystem 1: Thread management (0x0110–0x011F)
    pub const SYS_TRX_THREAD_CREATE: GenSyscallNr = GenSyscallNr(0x0110);
    pub const SYS_TRX_THREAD_EXIT: GenSyscallNr = GenSyscallNr(0x0111);
    pub const SYS_TRX_THREAD_JOIN: GenSyscallNr = GenSyscallNr(0x0112);
    pub const SYS_TRX_THREAD_SET_AFFINITY: GenSyscallNr = GenSyscallNr(0x0114);
    pub const SYS_TRX_THREAD_GET_AFFINITY: GenSyscallNr = GenSyscallNr(0x0115);
    pub const SYS_TRX_THREAD_SET_NAME: GenSyscallNr = GenSyscallNr(0x0116);
    pub const SYS_TRX_FUTEX_WAIT: GenSyscallNr = GenSyscallNr(0x0117);
    pub const SYS_TRX_FUTEX_WAKE: GenSyscallNr = GenSyscallNr(0x0118);

    // Subsystem 2: Memory management (0x0120–0x012F)
    pub const SYS_TRX_MEM_PROTECT: GenSyscallNr = GenSyscallNr(0x0122);
    pub const SYS_TRX_MEM_MAP: GenSyscallNr = GenSyscallNr(0x0123);
    pub const SYS_TRX_MEM_UNMAP: GenSyscallNr = GenSyscallNr(0x0124);
    pub const SYS_TRX_MEM_SHARE_CREATE: GenSyscallNr = GenSyscallNr(0x0125);
    pub const SYS_TRX_MEM_SHARE_MAP: GenSyscallNr = GenSyscallNr(0x0126);
    pub const SYS_TRX_MEM_SHARE_UNMAP: GenSyscallNr = GenSyscallNr(0x0127);
    pub const SYS_TRX_MEM_DMA_ALLOC: GenSyscallNr = GenSyscallNr(0x0128);
    pub const SYS_TRX_MEM_DMA_FREE: GenSyscallNr = GenSyscallNr(0x0129);

    // Subsystem 3: IPC channels (0x0130–0x013F)
    pub const SYS_TRX_CHANNEL_CREATE: GenSyscallNr = GenSyscallNr(0x0130);
    pub const SYS_TRX_CHANNEL_SEND: GenSyscallNr = GenSyscallNr(0x0131);
    pub const SYS_TRX_CHANNEL_RECV: GenSyscallNr = GenSyscallNr(0x0132);
    pub const SYS_TRX_CHANNEL_CLOSE: GenSyscallNr = GenSyscallNr(0x0133);
    pub const SYS_TRX_CHANNEL_POLL: GenSyscallNr = GenSyscallNr(0x0134);
    pub const SYS_TRX_SIGNAL_CREATE: GenSyscallNr = GenSyscallNr(0x0135);
    pub const SYS_TRX_SIGNAL_RAISE: GenSyscallNr = GenSyscallNr(0x0136);
    pub const SYS_TRX_SIGNAL_WAIT: GenSyscallNr = GenSyscallNr(0x0137);
    pub const SYS_TRX_SIGNAL_CLEAR: GenSyscallNr = GenSyscallNr(0x0138);
    pub const SYS_TRX_EVENT_WAIT_MANY: GenSyscallNr = GenSyscallNr(0x0139);

    // Subsystem 4: File system extensions (0x0140–0x014F)
    pub const SYS_TRX_FS_MKDIR: GenSyscallNr = GenSyscallNr(0x0147);
    pub const SYS_TRX_FS_UNLINK: GenSyscallNr = GenSyscallNr(0x0148);
    pub const SYS_TRX_FS_RENAME: GenSyscallNr = GenSyscallNr(0x0149);

    // Subsystem 5: Display / compositor (0x0150–0x015F)
    pub const SYS_TRX_DISPLAY_ENUMERATE: GenSyscallNr = GenSyscallNr(0x0150);
    pub const SYS_TRX_DISPLAY_SET_MODE: GenSyscallNr = GenSyscallNr(0x0151);
    pub const SYS_TRX_COMPOSITOR_CREATE: GenSyscallNr = GenSyscallNr(0x0152);
    pub const SYS_TRX_COMPOSITOR_PRESENT: GenSyscallNr = GenSyscallNr(0x0153);
    pub const SYS_TRX_SURFACE_CREATE: GenSyscallNr = GenSyscallNr(0x0154);
    pub const SYS_TRX_SURFACE_DESTROY: GenSyscallNr = GenSyscallNr(0x0155);
    pub const SYS_TRX_SURFACE_RESIZE: GenSyscallNr = GenSyscallNr(0x0156);
    pub const SYS_TRX_BUFFER_CREATE: GenSyscallNr = GenSyscallNr(0x0157);
    pub const SYS_TRX_BUFFER_MAP: GenSyscallNr = GenSyscallNr(0x0158);
    pub const SYS_TRX_BUFFER_UNMAP: GenSyscallNr = GenSyscallNr(0x0159);

    // Subsystem 6: Input devices (0x0160–0x016F)
    pub const SYS_TRX_INPUT_ENUMERATE: GenSyscallNr = GenSyscallNr(0x0160);
    pub const SYS_TRX_INPUT_OPEN: GenSyscallNr = GenSyscallNr(0x0161);
    pub const SYS_TRX_INPUT_CLOSE: GenSyscallNr = GenSyscallNr(0x0162);
    pub const SYS_TRX_INPUT_READ_EVENTS: GenSyscallNr = GenSyscallNr(0x0163);
    pub const SYS_TRX_INPUT_GRAB: GenSyscallNr = GenSyscallNr(0x0164);
    pub const SYS_TRX_INPUT_UNGRAB: GenSyscallNr = GenSyscallNr(0x0165);
    pub const SYS_TRX_INPUT_SET_KEYMAP: GenSyscallNr = GenSyscallNr(0x0166);
    pub const SYS_TRX_TOUCH_READ_EVENTS: GenSyscallNr = GenSyscallNr(0x0167);
    pub const SYS_TRX_INPUT_SET_ACCEL: GenSyscallNr = GenSyscallNr(0x0168);

    // Subsystem 7: GPU / DRM (0x0170–0x017F)
    pub const SYS_TRX_GPU_OPEN: GenSyscallNr = GenSyscallNr(0x0170);
    pub const SYS_TRX_GPU_CLOSE: GenSyscallNr = GenSyscallNr(0x0171);
    pub const SYS_TRX_GPU_ALLOC_BO: GenSyscallNr = GenSyscallNr(0x0172);
    pub const SYS_TRX_GPU_FREE_BO: GenSyscallNr = GenSyscallNr(0x0173);
    pub const SYS_TRX_GPU_MAP_BO: GenSyscallNr = GenSyscallNr(0x0174);
    pub const SYS_TRX_GPU_SUBMIT: GenSyscallNr = GenSyscallNr(0x0175);
    pub const SYS_TRX_GPU_WAIT_FENCE: GenSyscallNr = GenSyscallNr(0x0176);
    pub const SYS_TRX_GPU_EXPORT_DMABUF: GenSyscallNr = GenSyscallNr(0x0177);
    pub const SYS_TRX_GPU_IMPORT_DMABUF: GenSyscallNr = GenSyscallNr(0x0178);
    pub const SYS_TRX_GPU_GET_INFO: GenSyscallNr = GenSyscallNr(0x0179);

    // Subsystem 8: Networking (0x0180–0x018F)
    pub const SYS_TRX_NET_SOCKET: GenSyscallNr = GenSyscallNr(0x0180);
    pub const SYS_TRX_NET_BIND: GenSyscallNr = GenSyscallNr(0x0181);
    pub const SYS_TRX_NET_LISTEN: GenSyscallNr = GenSyscallNr(0x0182);
    pub const SYS_TRX_NET_ACCEPT: GenSyscallNr = GenSyscallNr(0x0183);
    pub const SYS_TRX_NET_CONNECT: GenSyscallNr = GenSyscallNr(0x0184);
    pub const SYS_TRX_NET_SENDMSG: GenSyscallNr = GenSyscallNr(0x0185);
    pub const SYS_TRX_NET_RECVMSG: GenSyscallNr = GenSyscallNr(0x0186);

    // Subsystem 9: Time / timers (0x0190–0x019F)
    pub const SYS_TRX_TIMER_CREATE: GenSyscallNr = GenSyscallNr(0x0192);
    pub const SYS_TRX_TIMER_SET: GenSyscallNr = GenSyscallNr(0x0193);

    // Subsystem 10: System / audit (0x01A0–0x01AF)
    pub const SYS_TRX_SYSTEM_REBOOT: GenSyscallNr = GenSyscallNr(0x01A0);
    pub const SYS_TRX_MODULE_LOAD: GenSyscallNr = GenSyscallNr(0x01A1);
    pub const SYS_TRX_MODULE_UNLOAD: GenSyscallNr = GenSyscallNr(0x01A2);
    pub const SYS_TRX_AUDIT_READ: GenSyscallNr = GenSyscallNr(0x01A3);
    pub const SYS_TRX_AUDIT_SET_POLICY: GenSyscallNr = GenSyscallNr(0x01A4);

    // Subsystem 11: Sigil / sandbox — legacy (0x01B0–0x01BF)
    pub const SYS_TRX_SIGIL_SIGN: GenSyscallNr = GenSyscallNr(0x01B0);
    pub const SYS_TRX_SIGIL_VERIFY: GenSyscallNr = GenSyscallNr(0x01B1);
    pub const SYS_TRX_SANDBOX_CREATE: GenSyscallNr = GenSyscallNr(0x01B2);
    pub const SYS_TRX_SANDBOX_ENTER: GenSyscallNr = GenSyscallNr(0x01B3);

    // Deprecated aliases — removed in v0.2.0. Use SYS_TRX_* names.
    pub const SYS_CAP_GRANT: GenSyscallNr = SYS_TRX_PROCESS_CAP_GRANT;
    pub const SYS_CAP_REVOKE: GenSyscallNr = SYS_TRX_PROCESS_CAP_REVOKE;
    pub const SYS_CAP_CHECK: GenSyscallNr = SYS_TRX_PROCESS_CAP_QUERY;
    pub const SYS_SIGIL_SIGN: GenSyscallNr = SYS_TRX_SIGIL_SIGN;
    pub const SYS_SIGIL_VERIFY: GenSyscallNr = SYS_TRX_SIGIL_VERIFY;
    pub const SYS_AUDIT_LOG: GenSyscallNr = SYS_TRX_AUDIT_READ;
    pub const SYS_SANDBOX_CREATE: GenSyscallNr = SYS_TRX_SANDBOX_CREATE;
    pub const SYS_SANDBOX_ENTER: GenSyscallNr = SYS_TRX_SANDBOX_ENTER;

    // GenesisOS-RT syscalls (0x0200–0x02FF)
    pub const SYS_RT_TASK_CREATE: GenSyscallNr = GenSyscallNr(0x0200);
    pub const SYS_RT_TASK_SET_PRIO: GenSyscallNr = GenSyscallNr(0x0201);
    pub const SYS_RT_TASK_SET_DEADLINE: GenSyscallNr = GenSyscallNr(0x0202);
    pub const SYS_RT_TIMER_CREATE: GenSyscallNr = GenSyscallNr(0x0203);
    pub const SYS_RT_TIMER_ARM: GenSyscallNr = GenSyscallNr(0x0204);
    pub const SYS_RT_SENSOR_READ: GenSyscallNr = GenSyscallNr(0x0205);
    pub const SYS_RT_ACTUATOR_WRITE: GenSyscallNr = GenSyscallNr(0x0206);

    // HermeticaOS syscalls (0x0300–0x03FF)
    pub const SYS_MOD_LOAD: GenSyscallNr = GenSyscallNr(0x0300);
    pub const SYS_MOD_UNLOAD: GenSyscallNr = GenSyscallNr(0x0301);
    pub const SYS_MOD_QUERY: GenSyscallNr = GenSyscallNr(0x0302);
    pub const SYS_MOD_HOT_SWAP: GenSyscallNr = GenSyscallNr(0x0303);
    pub const SYS_MOD_IPC_SEND: GenSyscallNr = GenSyscallNr(0x0304);
    pub const SYS_MOD_IPC_RECV: GenSyscallNr = GenSyscallNr(0x0305);
    pub const SYS_MOD_CAP_REQUEST: GenSyscallNr = GenSyscallNr(0x0306);
}

impl GenSyscallNr {
    #[inline]
    pub const fn is_shared(self) -> bool {
        self.0 < syscall::SHARED_LIMIT.0
    }

    #[inline]
    pub const fn is_terranox(self) -> bool {
        self.0 >= syscall::TERRANOX_BASE.0 && self.0 < syscall::TERRANOX_LIMIT.0
    }

    #[inline]
    pub const fn is_genesisrt(self) -> bool {
        self.0 >= syscall::GENESISRT_BASE.0 && self.0 < syscall::GENESISRT_LIMIT.0
    }

    #[inline]
    pub const fn is_hermetica(self) -> bool {
        self.0 >= syscall::HERMETICA_BASE.0 && self.0 < syscall::HERMETICA_LIMIT.0
    }

    /// Returns the TerranoxOS subsystem index (0-15), or -1 if not TerranoxOS.
    #[inline]
    pub const fn trx_subsystem(self) -> i32 {
        if !self.is_terranox() {
            return -1;
        }
        ((self.0 - syscall::TERRANOX_BASE.0) >> 4) as i32
    }
}

// ────────────────────────────────────────────────────────────
// Module types — mirrors genesis_module.h
// ────────────────────────────────────────────────────────────

pub const MODULE_ABI_VERSION_MAJOR: u16 = 0;
pub const MODULE_ABI_VERSION_MINOR: u16 = 1;

/// Capability bitfield type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct GenCapability(pub u64);

impl GenCapability {
    pub const NONE: Self = Self(0);
    pub const MEM_READ: Self = Self(1 << 0);
    pub const MEM_WRITE: Self = Self(1 << 1);
    pub const MEM_EXEC: Self = Self(1 << 2);
    pub const IO_PORT: Self = Self(1 << 3);
    pub const IRQ_REGISTER: Self = Self(1 << 4);
    pub const DMA: Self = Self(1 << 5);
    pub const TIMER: Self = Self(1 << 6);
    pub const NET: Self = Self(1 << 7);
    pub const BLOCK: Self = Self(1 << 8);
    pub const FS: Self = Self(1 << 9);
    pub const PROCESS_CREATE: Self = Self(1 << 10);
    pub const PROCESS_SIGNAL: Self = Self(1 << 11);
    pub const IPC: Self = Self(1 << 12);
    pub const AUDIT: Self = Self(1 << 13);
    pub const MODULE_LOAD: Self = Self(1 << 14);
    pub const CRYPTO: Self = Self(1 << 15);
    pub const ALL: Self = Self(0xFFFF);

    /// Check if this capability set contains a specific capability.
    #[inline]
    pub const fn contains(self, cap: Self) -> bool {
        (self.0 & cap.0) == cap.0
    }

    /// Combine two capability sets (bitwise OR).
    #[inline]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Intersect two capability sets (bitwise AND).
    #[inline]
    pub const fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    /// Remove capabilities (bitwise AND NOT).
    #[inline]
    pub const fn difference(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }
}

// ────────────────────────────────────────────────────────────
// TrxCapSet — hierarchical capability model (TerranoxOS)
// ────────────────────────────────────────────────────────────

/// 128-bit domain-partitioned capability bitmask for TerranoxOS.
///
/// Each of 12 capability domains occupies a contiguous bit range.
/// Parent domain constants are the bitwise OR of all child sub-capabilities.
/// Hierarchy is resolved at compile time — no runtime graph traversal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct TrxCapSet {
    pub lo: u64,
    pub hi: u64,
}

pub mod trx_cap {
    use super::TrxCapSet;

    // ── process domain (lo bits 0-3) ──
    pub const PROCESS_CREATE: TrxCapSet = TrxCapSet { lo: 1 << 0, hi: 0 };
    pub const PROCESS_SIGNAL: TrxCapSet = TrxCapSet { lo: 1 << 1, hi: 0 };
    pub const PROCESS_INSPECT: TrxCapSet = TrxCapSet { lo: 1 << 2, hi: 0 };
    pub const PROCESS_MANAGE: TrxCapSet = TrxCapSet { lo: 1 << 3, hi: 0 };
    pub const PROCESS: TrxCapSet = TrxCapSet { lo: 0xF, hi: 0 };

    // ── memory domain (lo bits 4-7) ──
    pub const MEMORY_ALLOC: TrxCapSet = TrxCapSet { lo: 1 << 4, hi: 0 };
    pub const MEMORY_MAP: TrxCapSet = TrxCapSet { lo: 1 << 5, hi: 0 };
    pub const MEMORY_SHARE: TrxCapSet = TrxCapSet { lo: 1 << 6, hi: 0 };
    pub const MEMORY_DMA: TrxCapSet = TrxCapSet { lo: 1 << 7, hi: 0 };
    pub const MEMORY: TrxCapSet = TrxCapSet { lo: 0xF0, hi: 0 };

    // ── thread domain (lo bits 8-10) ──
    pub const THREAD_CREATE: TrxCapSet = TrxCapSet { lo: 1 << 8, hi: 0 };
    pub const THREAD_JOIN: TrxCapSet = TrxCapSet { lo: 1 << 9, hi: 0 };
    pub const THREAD_AFFINITY: TrxCapSet = TrxCapSet { lo: 1 << 10, hi: 0 };
    pub const THREAD: TrxCapSet = TrxCapSet { lo: 0x700, hi: 0 };

    // ── ipc domain (lo bits 11-13) ──
    pub const IPC_CHANNEL: TrxCapSet = TrxCapSet { lo: 1 << 11, hi: 0 };
    pub const IPC_SIGNAL: TrxCapSet = TrxCapSet { lo: 1 << 12, hi: 0 };
    pub const IPC_EVENT: TrxCapSet = TrxCapSet { lo: 1 << 13, hi: 0 };
    pub const IPC: TrxCapSet = TrxCapSet { lo: 0x3800, hi: 0 };

    // ── fs domain (lo bits 14-17) ──
    pub const FS_READ: TrxCapSet = TrxCapSet { lo: 1 << 14, hi: 0 };
    pub const FS_WRITE: TrxCapSet = TrxCapSet { lo: 1 << 15, hi: 0 };
    pub const FS_CREATE: TrxCapSet = TrxCapSet { lo: 1 << 16, hi: 0 };
    pub const FS_DELETE: TrxCapSet = TrxCapSet { lo: 1 << 17, hi: 0 };
    pub const FS: TrxCapSet = TrxCapSet { lo: 0x3C000, hi: 0 };

    // ── io domain (lo bits 18-20) ──
    pub const IO_PORT: TrxCapSet = TrxCapSet { lo: 1 << 18, hi: 0 };
    pub const IO_IRQ: TrxCapSet = TrxCapSet { lo: 1 << 19, hi: 0 };
    pub const IO_MMIO: TrxCapSet = TrxCapSet { lo: 1 << 20, hi: 0 };
    pub const IO: TrxCapSet = TrxCapSet { lo: 0x1C_0000, hi: 0 };

    // ── display domain (lo bits 21-24) ──
    pub const DISPLAY_COMPOSITOR: TrxCapSet = TrxCapSet { lo: 1 << 21, hi: 0 };
    pub const DISPLAY_SURFACE: TrxCapSet = TrxCapSet { lo: 1 << 22, hi: 0 };
    pub const DISPLAY_BUFFER: TrxCapSet = TrxCapSet { lo: 1 << 23, hi: 0 };
    pub const DISPLAY_MODE: TrxCapSet = TrxCapSet { lo: 1 << 24, hi: 0 };
    pub const DISPLAY: TrxCapSet = TrxCapSet { lo: 0x1E0_0000, hi: 0 };

    // ── input domain (lo bits 25-27) ──
    pub const INPUT_KEYBOARD: TrxCapSet = TrxCapSet { lo: 1 << 25, hi: 0 };
    pub const INPUT_POINTER: TrxCapSet = TrxCapSet { lo: 1 << 26, hi: 0 };
    pub const INPUT_TOUCH: TrxCapSet = TrxCapSet { lo: 1 << 27, hi: 0 };
    pub const INPUT: TrxCapSet = TrxCapSet { lo: 0xE00_0000, hi: 0 };

    // ── gpu domain (hi bits 0-2) ──
    pub const GPU_RENDER: TrxCapSet = TrxCapSet { lo: 0, hi: 1 << 0 };
    pub const GPU_COMPUTE: TrxCapSet = TrxCapSet { lo: 0, hi: 1 << 1 };
    pub const GPU_ALLOC: TrxCapSet = TrxCapSet { lo: 0, hi: 1 << 2 };
    pub const GPU: TrxCapSet = TrxCapSet { lo: 0, hi: 0x7 };

    // ── net domain (hi bits 3-5) ──
    pub const NET_SOCKET: TrxCapSet = TrxCapSet { lo: 0, hi: 1 << 3 };
    pub const NET_BIND: TrxCapSet = TrxCapSet { lo: 0, hi: 1 << 4 };
    pub const NET_RAW: TrxCapSet = TrxCapSet { lo: 0, hi: 1 << 5 };
    pub const NET: TrxCapSet = TrxCapSet { lo: 0, hi: 0x38 };

    // ── time domain (hi bits 6-8) ──
    pub const TIME_READ: TrxCapSet = TrxCapSet { lo: 0, hi: 1 << 6 };
    pub const TIME_SLEEP: TrxCapSet = TrxCapSet { lo: 0, hi: 1 << 7 };
    pub const TIME_TIMER: TrxCapSet = TrxCapSet { lo: 0, hi: 1 << 8 };
    pub const TIME: TrxCapSet = TrxCapSet { lo: 0, hi: 0x1C0 };

    // ── system domain (hi bits 9-11) ──
    pub const SYSTEM_REBOOT: TrxCapSet = TrxCapSet { lo: 0, hi: 1 << 9 };
    pub const SYSTEM_MODULE: TrxCapSet = TrxCapSet { lo: 0, hi: 1 << 10 };
    pub const SYSTEM_AUDIT: TrxCapSet = TrxCapSet { lo: 0, hi: 1 << 11 };
    pub const SYSTEM: TrxCapSet = TrxCapSet { lo: 0, hi: 0xE00 };

    // ── aggregate constants ──
    pub const NONE: TrxCapSet = TrxCapSet { lo: 0, hi: 0 };
    pub const ROOT: TrxCapSet = TrxCapSet { lo: 0xFFF_FFFF, hi: 0xFFF };
}

impl TrxCapSet {
    /// Check if this capability set contains all bits in `cap`.
    #[inline]
    pub const fn contains(self, cap: Self) -> bool {
        (self.lo & cap.lo) == cap.lo && (self.hi & cap.hi) == cap.hi
    }

    /// Combine two capability sets (bitwise OR).
    #[inline]
    pub const fn union(self, other: Self) -> Self {
        Self {
            lo: self.lo | other.lo,
            hi: self.hi | other.hi,
        }
    }

    /// Intersect two capability sets (bitwise AND).
    #[inline]
    pub const fn intersection(self, other: Self) -> Self {
        Self {
            lo: self.lo & other.lo,
            hi: self.hi & other.hi,
        }
    }

    /// Remove capabilities (bitwise AND NOT).
    #[inline]
    pub const fn difference(self, other: Self) -> Self {
        Self {
            lo: self.lo & !other.lo,
            hi: self.hi & !other.hi,
        }
    }

    /// Check if the capability set is empty.
    #[inline]
    pub const fn is_empty(self) -> bool {
        self.lo == 0 && self.hi == 0
    }
}

// ────────────────────────────────────────────────────────────
// TerranoxOS syscall data structures — mirrors genesis_trx_types.h
// ────────────────────────────────────────────────────────────

/// Transferable capability handle used at the syscall interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct GenTrxCapToken {
    pub id: u64,
    pub rights: u64,
}

/// Header for a variable-length array of capability tokens.
/// `count` GenTrxCapToken entries follow immediately in memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct GenTrxCapTokenSet {
    pub count: u32,
    pub _pad0: u32,
}

/// Process metadata returned by `trx_process_info`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct GenTrxProcessInfo {
    pub pid: i64,
    pub state: i32,
    pub thread_count: u32,
    pub memory_bytes: u64,
    pub cpu_time_ns: u64,
    pub cap_count: u32,
    pub _pad0: u32,
}

/// Display information returned by `trx_display_enumerate`.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct GenTrxDisplayInfo {
    pub display_id: u32,
    pub width_px: u32,
    pub height_px: u32,
    pub refresh_mhz: u32,
    pub connector: u32,
    pub name: [u8; 32],
    pub _pad0: u32,
}

/// Input event (libinput-compatible layout).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct GenTrxInputEvent {
    pub timestamp_ns: u64,
    pub r#type: u32,
    pub code: u32,
    pub value: i32,
    pub device_id: u32,
}

/// Multi-touch event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct GenTrxTouchEvent {
    pub timestamp_ns: u64,
    pub slot: u32,
    pub r#type: u32,
    pub x: i32,
    pub y: i32,
    pub pressure: i32,
    pub _pad0: u32,
}

/// Wait item for `trx_event_wait_many`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct GenTrxWaitItem {
    pub handle: i64,
    pub events: u32,
    pub observed: u32,
}

/// POSIX-compatible timespec.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct GenTrxTimespec {
    pub tv_sec: i64,
    pub tv_nsec: i64,
}

/// GPU capabilities information.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct GenTrxGpuInfo {
    pub vendor_id: u32,
    pub device_id: u32,
    pub vram_bytes: u64,
    pub max_texture_size: u32,
    pub supported_formats: u32,
    pub driver_name: [u8; 64],
}

/// Capability audit log entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct GenTrxAuditEntry {
    pub timestamp_ns: u64,
    pub pid: i64,
    pub tid: i64,
    pub capability: GenTrxCapToken,
    pub syscall_nr: u32,
    pub result: u32,
}

pub const MODULE_NAME_MAX: usize = 64;
pub const MODULE_MAGIC: u32 = 0x47454E4D;
pub const MODULE_SECTION: &str = ".gen_module";

/// Kernel API function pointer table provided to modules.
///
/// Each field is `Option<unsafe extern "C" fn(...)>` — the kernel may
/// leave unsupported operations as `None`. `Option<extern "C" fn>` has
/// the same ABI layout as a nullable C function pointer due to niche
/// optimization.
#[repr(C)]
pub struct GenKernelAPI {
    pub alloc_pages: Option<unsafe extern "C" fn(count: usize) -> *mut u8>,
    pub free_pages: Option<unsafe extern "C" fn(addr: *mut u8, count: usize)>,

    pub log: Option<unsafe extern "C" fn(level: u8, msg: *const u8, len: usize) -> GenResult>,

    pub ipc_send:
        Option<unsafe extern "C" fn(target: u32, data: *const u8, len: usize) -> GenResult>,
    pub ipc_recv: Option<
        unsafe extern "C" fn(
            source: *mut u32,
            buf: *mut u8,
            buf_len: usize,
            out_len: *mut usize,
        ) -> GenResult,
    >,

    pub irq_register: Option<
        unsafe extern "C" fn(
            irq: u32,
            handler: unsafe extern "C" fn(u32, *mut u8),
            ctx: *mut u8,
        ) -> GenResult,
    >,
    pub irq_unregister: Option<unsafe extern "C" fn(irq: u32) -> GenResult>,

    pub timer_create: Option<
        unsafe extern "C" fn(
            interval_ns: u64,
            callback: unsafe extern "C" fn(*mut u8),
            ctx: *mut u8,
            out_id: *mut u32,
        ) -> GenResult,
    >,
    pub timer_cancel: Option<unsafe extern "C" fn(timer_id: u32) -> GenResult>,

    pub query_capability:
        Option<unsafe extern "C" fn(cap: GenCapability, out_granted: *mut i32) -> GenResult>,
}

/// Module descriptor placed in the `.gen_module` ELF section.
#[repr(C)]
pub struct GenModuleDescriptor {
    pub magic: u32,
    pub abi_version_major: u16,
    pub abi_version_minor: u16,

    pub name: [u8; MODULE_NAME_MAX],
    pub module_version_major: u32,
    pub module_version_minor: u32,
    pub module_version_patch: u32,
    pub _pad0: u32,

    pub required_caps: GenCapability,
    pub optional_caps: GenCapability,

    pub init: Option<unsafe extern "C" fn(api: *const GenKernelAPI) -> GenResult>,
    pub fini: Option<unsafe extern "C" fn()>,
}

// ────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem;

    // -- GenResult tests --

    #[test]
    fn gen_result_ok_is_zero() {
        assert_eq!(GenResult::OK.0, 0);
    }

    #[test]
    fn gen_result_is_ok() {
        assert!(GenResult::OK.is_ok());
        assert!(!GenResult::OK.is_error());
    }

    #[test]
    fn gen_result_errors_are_negative() {
        let errors = [
            GenResult::ERR_INVALID_ARG,
            GenResult::ERR_OUT_OF_MEMORY,
            GenResult::ERR_NOT_FOUND,
            GenResult::ERR_ALREADY_EXISTS,
            GenResult::ERR_BUFFER_TOO_SMALL,
            GenResult::ERR_NOT_SUPPORTED,
            GenResult::ERR_BUSY,
            GenResult::ERR_TIMEOUT,
            GenResult::ERR_INTERRUPTED,
            GenResult::ERR_OVERFLOW,
            GenResult::ERR_PERMISSION_DENIED,
            GenResult::ERR_ACCESS_VIOLATION,
            GenResult::ERR_INVALID_CAPABILITY,
            GenResult::ERR_IO,
            GenResult::ERR_DEVICE_OFFLINE,
            GenResult::ERR_BAD_ADDRESS,
            GenResult::ERR_INVALID_FORMAT,
            GenResult::ERR_CHECKSUM_MISMATCH,
            GenResult::ERR_VERSION_MISMATCH,
            GenResult::ERR_MODULE_LOAD_FAILED,
            GenResult::ERR_MODULE_INIT_FAILED,
            GenResult::ERR_MODULE_NOT_FOUND,
            GenResult::ERR_MODULE_INCOMPATIBLE,
            GenResult::ERR_DEADLINE_MISS,
            GenResult::ERR_PRIORITY_INV,
            GenResult::ERR_STACK_OVERFLOW,
            GenResult::ERR_BAD_SYSCALL,
            GenResult::ERR_BAD_HANDLE,
            GenResult::ERR_SYSCALL_INTERRUPTED,
            GenResult::ERR_HANDLE_LIMIT,
            GenResult::ERR_CHANNEL_CLOSED,
            GenResult::ERR_DISPLAY_OFFLINE,
            GenResult::ERR_GPU_ERROR,
        ];
        for e in errors {
            assert!(e.is_error(), "Expected error for code {}", e.0);
            assert!(!e.is_ok());
        }
    }

    #[test]
    fn gen_result_no_duplicate_codes() {
        let codes = [
            GenResult::OK.0,
            GenResult::ERR_INVALID_ARG.0,
            GenResult::ERR_OUT_OF_MEMORY.0,
            GenResult::ERR_NOT_FOUND.0,
            GenResult::ERR_ALREADY_EXISTS.0,
            GenResult::ERR_BUFFER_TOO_SMALL.0,
            GenResult::ERR_NOT_SUPPORTED.0,
            GenResult::ERR_BUSY.0,
            GenResult::ERR_TIMEOUT.0,
            GenResult::ERR_INTERRUPTED.0,
            GenResult::ERR_OVERFLOW.0,
            GenResult::ERR_PERMISSION_DENIED.0,
            GenResult::ERR_ACCESS_VIOLATION.0,
            GenResult::ERR_INVALID_CAPABILITY.0,
            GenResult::ERR_IO.0,
            GenResult::ERR_DEVICE_OFFLINE.0,
            GenResult::ERR_BAD_ADDRESS.0,
            GenResult::ERR_INVALID_FORMAT.0,
            GenResult::ERR_CHECKSUM_MISMATCH.0,
            GenResult::ERR_VERSION_MISMATCH.0,
            GenResult::ERR_MODULE_LOAD_FAILED.0,
            GenResult::ERR_MODULE_INIT_FAILED.0,
            GenResult::ERR_MODULE_NOT_FOUND.0,
            GenResult::ERR_MODULE_INCOMPATIBLE.0,
            GenResult::ERR_DEADLINE_MISS.0,
            GenResult::ERR_PRIORITY_INV.0,
            GenResult::ERR_STACK_OVERFLOW.0,
            GenResult::ERR_BAD_SYSCALL.0,
            GenResult::ERR_BAD_HANDLE.0,
            GenResult::ERR_SYSCALL_INTERRUPTED.0,
            GenResult::ERR_HANDLE_LIMIT.0,
            GenResult::ERR_CHANNEL_CLOSED.0,
            GenResult::ERR_DISPLAY_OFFLINE.0,
            GenResult::ERR_GPU_ERROR.0,
        ];
        for (i, &a) in codes.iter().enumerate() {
            for &b in &codes[i + 1..] {
                assert_ne!(a, b, "Duplicate error code: {}", a);
            }
        }
    }

    #[test]
    fn gen_result_size() {
        assert_eq!(mem::size_of::<GenResult>(), 4);
    }

    // -- Errno mapping tests --

    #[test]
    fn errno_ok_maps_to_zero() {
        assert_eq!(GenResult::OK.to_errno(), 0);
        assert_eq!(GenResult::from_errno(0), GenResult::OK);
    }

    #[test]
    fn errno_round_trip_mapped_values() {
        // (GenResult, expected errno)
        let mappings: &[(GenResult, i32)] = &[
            (GenResult::ERR_INVALID_ARG, posix_errno::EINVAL),
            (GenResult::ERR_OUT_OF_MEMORY, posix_errno::ENOMEM),
            (GenResult::ERR_NOT_FOUND, posix_errno::ENOENT),
            (GenResult::ERR_ALREADY_EXISTS, posix_errno::EEXIST),
            (GenResult::ERR_BUSY, posix_errno::EBUSY),
            (GenResult::ERR_TIMEOUT, posix_errno::ETIMEDOUT),
            (GenResult::ERR_PERMISSION_DENIED, posix_errno::EPERM),
            (GenResult::ERR_ACCESS_VIOLATION, posix_errno::EACCES),
            (GenResult::ERR_BAD_ADDRESS, posix_errno::EFAULT),
            (GenResult::ERR_BAD_SYSCALL, posix_errno::ENOSYS),
            (GenResult::ERR_BAD_HANDLE, posix_errno::EBADF),
            (GenResult::ERR_CHANNEL_CLOSED, posix_errno::EPIPE),
        ];
        for &(result, expected_errno) in mappings {
            assert_eq!(
                result.to_errno(),
                expected_errno,
                "to_errno failed for {:?}",
                result
            );
        }
    }

    #[test]
    fn errno_from_errno_known_values() {
        assert_eq!(GenResult::from_errno(posix_errno::EINVAL), GenResult::ERR_INVALID_ARG);
        assert_eq!(GenResult::from_errno(posix_errno::ENOMEM), GenResult::ERR_OUT_OF_MEMORY);
        assert_eq!(GenResult::from_errno(posix_errno::EPERM), GenResult::ERR_PERMISSION_DENIED);
        assert_eq!(GenResult::from_errno(posix_errno::EBADF), GenResult::ERR_BAD_HANDLE);
        assert_eq!(GenResult::from_errno(posix_errno::ETIMEDOUT), GenResult::ERR_TIMEOUT);
    }

    #[test]
    fn errno_unknown_returns_sentinel() {
        // Unknown errno maps to NOT_SUPPORTED
        assert_eq!(GenResult::from_errno(999), GenResult::ERR_NOT_SUPPORTED);
        // Unknown GenResult maps to EINVAL
        assert_eq!(GenResult(i32::MIN).to_errno(), posix_errno::EINVAL);
    }

    // -- GenSyscallNr tests --

    #[test]
    fn syscall_ranges_no_overlap() {
        let ranges = [
            (syscall::SHARED_BASE.0, syscall::SHARED_LIMIT.0),
            (syscall::TERRANOX_BASE.0, syscall::TERRANOX_LIMIT.0),
            (syscall::GENESISRT_BASE.0, syscall::GENESISRT_LIMIT.0),
            (syscall::HERMETICA_BASE.0, syscall::HERMETICA_LIMIT.0),
        ];
        for (i, &(a_base, a_limit)) in ranges.iter().enumerate() {
            assert!(a_base < a_limit, "Empty range at index {i}");
            for &(b_base, b_limit) in &ranges[i + 1..] {
                assert!(
                    a_limit <= b_base || b_limit <= a_base,
                    "Overlapping ranges: [{a_base:#x},{a_limit:#x}) and [{b_base:#x},{b_limit:#x})"
                );
            }
        }
    }

    #[test]
    fn syscall_range_classification() {
        assert!(syscall::SYS_EXIT.is_shared());
        assert!(!syscall::SYS_EXIT.is_terranox());

        assert!(syscall::SYS_TRX_PROCESS_CAP_GRANT.is_terranox());
        assert!(!syscall::SYS_TRX_PROCESS_CAP_GRANT.is_shared());

        assert!(syscall::SYS_RT_TASK_CREATE.is_genesisrt());
        assert!(!syscall::SYS_RT_TASK_CREATE.is_terranox());

        assert!(syscall::SYS_MOD_LOAD.is_hermetica());
        assert!(!syscall::SYS_MOD_LOAD.is_genesisrt());
    }

    #[test]
    fn trx_syscall_all_in_range() {
        let trx_syscalls = [
            // Process
            syscall::SYS_TRX_PROCESS_CREATE, syscall::SYS_TRX_PROCESS_KILL,
            syscall::SYS_TRX_PROCESS_INFO, syscall::SYS_TRX_PROCESS_CAP_GRANT,
            syscall::SYS_TRX_PROCESS_CAP_REVOKE, syscall::SYS_TRX_PROCESS_CAP_QUERY,
            // Thread
            syscall::SYS_TRX_THREAD_CREATE, syscall::SYS_TRX_THREAD_EXIT,
            syscall::SYS_TRX_THREAD_JOIN, syscall::SYS_TRX_THREAD_SET_AFFINITY,
            syscall::SYS_TRX_THREAD_GET_AFFINITY, syscall::SYS_TRX_THREAD_SET_NAME,
            syscall::SYS_TRX_FUTEX_WAIT, syscall::SYS_TRX_FUTEX_WAKE,
            // Memory
            syscall::SYS_TRX_MEM_PROTECT, syscall::SYS_TRX_MEM_MAP,
            syscall::SYS_TRX_MEM_UNMAP, syscall::SYS_TRX_MEM_SHARE_CREATE,
            syscall::SYS_TRX_MEM_SHARE_MAP, syscall::SYS_TRX_MEM_SHARE_UNMAP,
            syscall::SYS_TRX_MEM_DMA_ALLOC, syscall::SYS_TRX_MEM_DMA_FREE,
            // IPC
            syscall::SYS_TRX_CHANNEL_CREATE, syscall::SYS_TRX_CHANNEL_SEND,
            syscall::SYS_TRX_CHANNEL_RECV, syscall::SYS_TRX_CHANNEL_CLOSE,
            syscall::SYS_TRX_CHANNEL_POLL, syscall::SYS_TRX_SIGNAL_CREATE,
            syscall::SYS_TRX_SIGNAL_RAISE, syscall::SYS_TRX_SIGNAL_WAIT,
            syscall::SYS_TRX_SIGNAL_CLEAR, syscall::SYS_TRX_EVENT_WAIT_MANY,
            // FS
            syscall::SYS_TRX_FS_MKDIR, syscall::SYS_TRX_FS_UNLINK,
            syscall::SYS_TRX_FS_RENAME,
            // Display
            syscall::SYS_TRX_DISPLAY_ENUMERATE, syscall::SYS_TRX_DISPLAY_SET_MODE,
            syscall::SYS_TRX_COMPOSITOR_CREATE, syscall::SYS_TRX_COMPOSITOR_PRESENT,
            syscall::SYS_TRX_SURFACE_CREATE, syscall::SYS_TRX_SURFACE_DESTROY,
            syscall::SYS_TRX_SURFACE_RESIZE, syscall::SYS_TRX_BUFFER_CREATE,
            syscall::SYS_TRX_BUFFER_MAP, syscall::SYS_TRX_BUFFER_UNMAP,
            // Input
            syscall::SYS_TRX_INPUT_ENUMERATE, syscall::SYS_TRX_INPUT_OPEN,
            syscall::SYS_TRX_INPUT_CLOSE, syscall::SYS_TRX_INPUT_READ_EVENTS,
            syscall::SYS_TRX_INPUT_GRAB, syscall::SYS_TRX_INPUT_UNGRAB,
            syscall::SYS_TRX_INPUT_SET_KEYMAP, syscall::SYS_TRX_TOUCH_READ_EVENTS,
            syscall::SYS_TRX_INPUT_SET_ACCEL,
            // GPU
            syscall::SYS_TRX_GPU_OPEN, syscall::SYS_TRX_GPU_CLOSE,
            syscall::SYS_TRX_GPU_ALLOC_BO, syscall::SYS_TRX_GPU_FREE_BO,
            syscall::SYS_TRX_GPU_MAP_BO, syscall::SYS_TRX_GPU_SUBMIT,
            syscall::SYS_TRX_GPU_WAIT_FENCE, syscall::SYS_TRX_GPU_EXPORT_DMABUF,
            syscall::SYS_TRX_GPU_IMPORT_DMABUF, syscall::SYS_TRX_GPU_GET_INFO,
            // Net
            syscall::SYS_TRX_NET_SOCKET, syscall::SYS_TRX_NET_BIND,
            syscall::SYS_TRX_NET_LISTEN, syscall::SYS_TRX_NET_ACCEPT,
            syscall::SYS_TRX_NET_CONNECT, syscall::SYS_TRX_NET_SENDMSG,
            syscall::SYS_TRX_NET_RECVMSG,
            // Time
            syscall::SYS_TRX_TIMER_CREATE, syscall::SYS_TRX_TIMER_SET,
            // System
            syscall::SYS_TRX_SYSTEM_REBOOT, syscall::SYS_TRX_MODULE_LOAD,
            syscall::SYS_TRX_MODULE_UNLOAD, syscall::SYS_TRX_AUDIT_READ,
            syscall::SYS_TRX_AUDIT_SET_POLICY,
            // Sigil/sandbox
            syscall::SYS_TRX_SIGIL_SIGN, syscall::SYS_TRX_SIGIL_VERIFY,
            syscall::SYS_TRX_SANDBOX_CREATE, syscall::SYS_TRX_SANDBOX_ENTER,
        ];
        for s in trx_syscalls {
            assert!(
                s.is_terranox(),
                "TRX syscall {:#x} not in TerranoxOS range",
                s.0
            );
        }
    }

    #[test]
    fn trx_syscall_no_duplicates() {
        let trx_nrs: [u32; 82] = [
            0x0100, 0x0103, 0x0104, 0x0105, 0x0106, 0x0107,
            0x0110, 0x0111, 0x0112, 0x0114, 0x0115, 0x0116, 0x0117, 0x0118,
            0x0122, 0x0123, 0x0124, 0x0125, 0x0126, 0x0127, 0x0128, 0x0129,
            0x0130, 0x0131, 0x0132, 0x0133, 0x0134, 0x0135, 0x0136, 0x0137, 0x0138, 0x0139,
            0x0147, 0x0148, 0x0149,
            0x0150, 0x0151, 0x0152, 0x0153, 0x0154, 0x0155, 0x0156, 0x0157, 0x0158, 0x0159,
            0x0160, 0x0161, 0x0162, 0x0163, 0x0164, 0x0165, 0x0166, 0x0167, 0x0168,
            0x0170, 0x0171, 0x0172, 0x0173, 0x0174, 0x0175, 0x0176, 0x0177, 0x0178, 0x0179,
            0x0180, 0x0181, 0x0182, 0x0183, 0x0184, 0x0185, 0x0186,
            0x0192, 0x0193,
            0x01A0, 0x01A1, 0x01A2, 0x01A3, 0x01A4,
            0x01B0, 0x01B1, 0x01B2, 0x01B3,
        ];
        for (i, &a) in trx_nrs.iter().enumerate() {
            for &b in &trx_nrs[i + 1..] {
                assert_ne!(a, b, "Duplicate TRX syscall number: {:#x}", a);
            }
        }
    }

    #[test]
    fn trx_syscall_subsystem_classification() {
        assert_eq!(syscall::SYS_TRX_PROCESS_CREATE.trx_subsystem(), 0);
        assert_eq!(syscall::SYS_TRX_THREAD_CREATE.trx_subsystem(), 1);
        assert_eq!(syscall::SYS_TRX_MEM_PROTECT.trx_subsystem(), 2);
        assert_eq!(syscall::SYS_TRX_CHANNEL_CREATE.trx_subsystem(), 3);
        assert_eq!(syscall::SYS_TRX_FS_MKDIR.trx_subsystem(), 4);
        assert_eq!(syscall::SYS_TRX_DISPLAY_ENUMERATE.trx_subsystem(), 5);
        assert_eq!(syscall::SYS_TRX_INPUT_ENUMERATE.trx_subsystem(), 6);
        assert_eq!(syscall::SYS_TRX_GPU_OPEN.trx_subsystem(), 7);
        assert_eq!(syscall::SYS_TRX_NET_SOCKET.trx_subsystem(), 8);
        assert_eq!(syscall::SYS_TRX_TIMER_CREATE.trx_subsystem(), 9);
        assert_eq!(syscall::SYS_TRX_SYSTEM_REBOOT.trx_subsystem(), 10);
        assert_eq!(syscall::SYS_TRX_SIGIL_SIGN.trx_subsystem(), 11);
        // Non-TerranoxOS returns -1
        assert_eq!(syscall::SYS_EXIT.trx_subsystem(), -1);
    }

    #[test]
    fn trx_deprecated_aliases_match_new_names() {
        assert_eq!(syscall::SYS_CAP_GRANT, syscall::SYS_TRX_PROCESS_CAP_GRANT);
        assert_eq!(syscall::SYS_CAP_REVOKE, syscall::SYS_TRX_PROCESS_CAP_REVOKE);
        assert_eq!(syscall::SYS_CAP_CHECK, syscall::SYS_TRX_PROCESS_CAP_QUERY);
        assert_eq!(syscall::SYS_SIGIL_SIGN, syscall::SYS_TRX_SIGIL_SIGN);
        assert_eq!(syscall::SYS_SIGIL_VERIFY, syscall::SYS_TRX_SIGIL_VERIFY);
        assert_eq!(syscall::SYS_AUDIT_LOG, syscall::SYS_TRX_AUDIT_READ);
        assert_eq!(syscall::SYS_SANDBOX_CREATE, syscall::SYS_TRX_SANDBOX_CREATE);
        assert_eq!(syscall::SYS_SANDBOX_ENTER, syscall::SYS_TRX_SANDBOX_ENTER);
    }

    #[test]
    fn syscall_shared_in_range() {
        let shared = [
            syscall::SYS_EXIT,
            syscall::SYS_WRITE,
            syscall::SYS_READ,
            syscall::SYS_MMAP,
            syscall::SYS_MUNMAP,
            syscall::SYS_YIELD,
            syscall::SYS_GETPID,
            syscall::SYS_SLEEP,
            syscall::SYS_CLOCK_GETTIME,
            syscall::SYS_OPEN,
            syscall::SYS_CLOSE,
            syscall::SYS_STAT,
            syscall::SYS_FSTAT,
            syscall::SYS_LSEEK,
            syscall::SYS_BRK,
            syscall::SYS_IOCTL,
            syscall::SYS_DUP2,
            syscall::SYS_PIPE,
            syscall::SYS_FORK,
            syscall::SYS_EXEC,
            syscall::SYS_WAIT,
            syscall::SYS_FCNTL,
            syscall::SYS_POLL,
        ];
        for s in shared {
            assert!(s.0 < 0x0100, "Shared syscall {:#x} out of range", s.0);
        }
    }

    #[test]
    fn syscall_nr_size() {
        assert_eq!(mem::size_of::<GenSyscallNr>(), 4);
    }

    // -- GenCapability tests --

    #[test]
    fn capability_bits_unique() {
        let caps = [
            GenCapability::MEM_READ,
            GenCapability::MEM_WRITE,
            GenCapability::MEM_EXEC,
            GenCapability::IO_PORT,
            GenCapability::IRQ_REGISTER,
            GenCapability::DMA,
            GenCapability::TIMER,
            GenCapability::NET,
            GenCapability::BLOCK,
            GenCapability::FS,
            GenCapability::PROCESS_CREATE,
            GenCapability::PROCESS_SIGNAL,
            GenCapability::IPC,
            GenCapability::AUDIT,
            GenCapability::MODULE_LOAD,
            GenCapability::CRYPTO,
        ];
        // Each capability must be a single bit
        for c in &caps {
            assert_eq!(
                c.0.count_ones(),
                1,
                "Capability {:#x} is not a single bit",
                c.0
            );
        }
        // No duplicates
        for (i, a) in caps.iter().enumerate() {
            for b in &caps[i + 1..] {
                assert_ne!(a.0, b.0, "Duplicate capability bit: {:#x}", a.0);
            }
        }
    }

    #[test]
    fn capability_contains() {
        assert!(GenCapability::ALL.contains(GenCapability::MEM_READ));
        assert!(GenCapability::ALL.contains(GenCapability::CRYPTO));
        assert!(!GenCapability::NONE.contains(GenCapability::MEM_READ));
        assert!(GenCapability::NONE.contains(GenCapability::NONE));
    }

    #[test]
    fn capability_union_intersection() {
        let rw = GenCapability::MEM_READ.union(GenCapability::MEM_WRITE);
        assert!(rw.contains(GenCapability::MEM_READ));
        assert!(rw.contains(GenCapability::MEM_WRITE));
        assert!(!rw.contains(GenCapability::MEM_EXEC));

        let r = rw.intersection(GenCapability::MEM_READ);
        assert_eq!(r, GenCapability::MEM_READ);
    }

    #[test]
    fn capability_difference() {
        let all_but_exec = GenCapability::ALL.difference(GenCapability::MEM_EXEC);
        assert!(all_but_exec.contains(GenCapability::MEM_READ));
        assert!(!all_but_exec.contains(GenCapability::MEM_EXEC));
    }

    #[test]
    fn capability_size() {
        assert_eq!(mem::size_of::<GenCapability>(), 8);
    }

    // -- Module descriptor tests --

    #[test]
    fn module_magic_value() {
        assert_eq!(MODULE_MAGIC, 0x47454E4D);
    }

    #[test]
    fn module_descriptor_alignment() {
        assert!(mem::align_of::<GenModuleDescriptor>() >= 4);
    }

    #[test]
    fn kernel_api_all_fields_option() {
        // Verify that a zeroed GenKernelAPI is valid (all None)
        let api: GenKernelAPI = unsafe { mem::zeroed() };
        assert!(api.alloc_pages.is_none());
        assert!(api.free_pages.is_none());
        assert!(api.log.is_none());
        assert!(api.ipc_send.is_none());
        assert!(api.ipc_recv.is_none());
        assert!(api.irq_register.is_none());
        assert!(api.irq_unregister.is_none());
        assert!(api.timer_create.is_none());
        assert!(api.timer_cancel.is_none());
        assert!(api.query_capability.is_none());
    }

    #[test]
    fn module_descriptor_name_capacity() {
        assert_eq!(MODULE_NAME_MAX, 64);
        let desc: GenModuleDescriptor = unsafe { mem::zeroed() };
        assert_eq!(desc.name.len(), MODULE_NAME_MAX);
    }

    #[test]
    fn abi_version_constants() {
        assert_eq!(MODULE_ABI_VERSION_MAJOR, 0);
        assert_eq!(MODULE_ABI_VERSION_MINOR, 1);
    }

    // -- TrxCapSet tests --

    #[test]
    fn trx_capset_size() {
        assert_eq!(mem::size_of::<TrxCapSet>(), 16);
    }

    #[test]
    fn trx_capset_alignment() {
        assert!(mem::align_of::<TrxCapSet>() >= 8);
    }

    #[test]
    fn trx_capset_leaf_bits_are_single_bit() {
        let leaves = [
            // lo word leaves
            trx_cap::PROCESS_CREATE,
            trx_cap::PROCESS_SIGNAL,
            trx_cap::PROCESS_INSPECT,
            trx_cap::PROCESS_MANAGE,
            trx_cap::MEMORY_ALLOC,
            trx_cap::MEMORY_MAP,
            trx_cap::MEMORY_SHARE,
            trx_cap::MEMORY_DMA,
            trx_cap::THREAD_CREATE,
            trx_cap::THREAD_JOIN,
            trx_cap::THREAD_AFFINITY,
            trx_cap::IPC_CHANNEL,
            trx_cap::IPC_SIGNAL,
            trx_cap::IPC_EVENT,
            trx_cap::FS_READ,
            trx_cap::FS_WRITE,
            trx_cap::FS_CREATE,
            trx_cap::FS_DELETE,
            trx_cap::IO_PORT,
            trx_cap::IO_IRQ,
            trx_cap::IO_MMIO,
            trx_cap::DISPLAY_COMPOSITOR,
            trx_cap::DISPLAY_SURFACE,
            trx_cap::DISPLAY_BUFFER,
            trx_cap::DISPLAY_MODE,
            trx_cap::INPUT_KEYBOARD,
            trx_cap::INPUT_POINTER,
            trx_cap::INPUT_TOUCH,
            // hi word leaves
            trx_cap::GPU_RENDER,
            trx_cap::GPU_COMPUTE,
            trx_cap::GPU_ALLOC,
            trx_cap::NET_SOCKET,
            trx_cap::NET_BIND,
            trx_cap::NET_RAW,
            trx_cap::TIME_READ,
            trx_cap::TIME_SLEEP,
            trx_cap::TIME_TIMER,
            trx_cap::SYSTEM_REBOOT,
            trx_cap::SYSTEM_MODULE,
            trx_cap::SYSTEM_AUDIT,
        ];
        for cap in &leaves {
            let total_bits = cap.lo.count_ones() + cap.hi.count_ones();
            assert_eq!(
                total_bits, 1,
                "Leaf cap lo={:#x} hi={:#x} should be a single bit",
                cap.lo, cap.hi
            );
        }
    }

    #[test]
    fn trx_capset_leaf_bits_unique() {
        let leaves: [(u64, u64); 40] = [
            (trx_cap::PROCESS_CREATE.lo, trx_cap::PROCESS_CREATE.hi),
            (trx_cap::PROCESS_SIGNAL.lo, trx_cap::PROCESS_SIGNAL.hi),
            (trx_cap::PROCESS_INSPECT.lo, trx_cap::PROCESS_INSPECT.hi),
            (trx_cap::PROCESS_MANAGE.lo, trx_cap::PROCESS_MANAGE.hi),
            (trx_cap::MEMORY_ALLOC.lo, trx_cap::MEMORY_ALLOC.hi),
            (trx_cap::MEMORY_MAP.lo, trx_cap::MEMORY_MAP.hi),
            (trx_cap::MEMORY_SHARE.lo, trx_cap::MEMORY_SHARE.hi),
            (trx_cap::MEMORY_DMA.lo, trx_cap::MEMORY_DMA.hi),
            (trx_cap::THREAD_CREATE.lo, trx_cap::THREAD_CREATE.hi),
            (trx_cap::THREAD_JOIN.lo, trx_cap::THREAD_JOIN.hi),
            (trx_cap::THREAD_AFFINITY.lo, trx_cap::THREAD_AFFINITY.hi),
            (trx_cap::IPC_CHANNEL.lo, trx_cap::IPC_CHANNEL.hi),
            (trx_cap::IPC_SIGNAL.lo, trx_cap::IPC_SIGNAL.hi),
            (trx_cap::IPC_EVENT.lo, trx_cap::IPC_EVENT.hi),
            (trx_cap::FS_READ.lo, trx_cap::FS_READ.hi),
            (trx_cap::FS_WRITE.lo, trx_cap::FS_WRITE.hi),
            (trx_cap::FS_CREATE.lo, trx_cap::FS_CREATE.hi),
            (trx_cap::FS_DELETE.lo, trx_cap::FS_DELETE.hi),
            (trx_cap::IO_PORT.lo, trx_cap::IO_PORT.hi),
            (trx_cap::IO_IRQ.lo, trx_cap::IO_IRQ.hi),
            (trx_cap::IO_MMIO.lo, trx_cap::IO_MMIO.hi),
            (trx_cap::DISPLAY_COMPOSITOR.lo, trx_cap::DISPLAY_COMPOSITOR.hi),
            (trx_cap::DISPLAY_SURFACE.lo, trx_cap::DISPLAY_SURFACE.hi),
            (trx_cap::DISPLAY_BUFFER.lo, trx_cap::DISPLAY_BUFFER.hi),
            (trx_cap::DISPLAY_MODE.lo, trx_cap::DISPLAY_MODE.hi),
            (trx_cap::INPUT_KEYBOARD.lo, trx_cap::INPUT_KEYBOARD.hi),
            (trx_cap::INPUT_POINTER.lo, trx_cap::INPUT_POINTER.hi),
            (trx_cap::INPUT_TOUCH.lo, trx_cap::INPUT_TOUCH.hi),
            (trx_cap::GPU_RENDER.lo, trx_cap::GPU_RENDER.hi),
            (trx_cap::GPU_COMPUTE.lo, trx_cap::GPU_COMPUTE.hi),
            (trx_cap::GPU_ALLOC.lo, trx_cap::GPU_ALLOC.hi),
            (trx_cap::NET_SOCKET.lo, trx_cap::NET_SOCKET.hi),
            (trx_cap::NET_BIND.lo, trx_cap::NET_BIND.hi),
            (trx_cap::NET_RAW.lo, trx_cap::NET_RAW.hi),
            (trx_cap::TIME_READ.lo, trx_cap::TIME_READ.hi),
            (trx_cap::TIME_SLEEP.lo, trx_cap::TIME_SLEEP.hi),
            (trx_cap::TIME_TIMER.lo, trx_cap::TIME_TIMER.hi),
            (trx_cap::SYSTEM_REBOOT.lo, trx_cap::SYSTEM_REBOOT.hi),
            (trx_cap::SYSTEM_MODULE.lo, trx_cap::SYSTEM_MODULE.hi),
            (trx_cap::SYSTEM_AUDIT.lo, trx_cap::SYSTEM_AUDIT.hi),
        ];
        for (i, a) in leaves.iter().enumerate() {
            for b in &leaves[i + 1..] {
                assert!(
                    a != b,
                    "Duplicate leaf cap: lo={:#x} hi={:#x}",
                    a.0, a.1
                );
            }
        }
    }

    #[test]
    fn trx_capset_parent_contains_children() {
        // Process domain
        assert!(trx_cap::PROCESS.contains(trx_cap::PROCESS_CREATE));
        assert!(trx_cap::PROCESS.contains(trx_cap::PROCESS_SIGNAL));
        assert!(trx_cap::PROCESS.contains(trx_cap::PROCESS_INSPECT));
        assert!(trx_cap::PROCESS.contains(trx_cap::PROCESS_MANAGE));
        // Memory domain
        assert!(trx_cap::MEMORY.contains(trx_cap::MEMORY_ALLOC));
        assert!(trx_cap::MEMORY.contains(trx_cap::MEMORY_DMA));
        // Thread domain
        assert!(trx_cap::THREAD.contains(trx_cap::THREAD_CREATE));
        assert!(trx_cap::THREAD.contains(trx_cap::THREAD_AFFINITY));
        // IPC domain
        assert!(trx_cap::IPC.contains(trx_cap::IPC_CHANNEL));
        assert!(trx_cap::IPC.contains(trx_cap::IPC_EVENT));
        // FS domain
        assert!(trx_cap::FS.contains(trx_cap::FS_READ));
        assert!(trx_cap::FS.contains(trx_cap::FS_DELETE));
        // IO domain
        assert!(trx_cap::IO.contains(trx_cap::IO_PORT));
        assert!(trx_cap::IO.contains(trx_cap::IO_MMIO));
        // Display domain
        assert!(trx_cap::DISPLAY.contains(trx_cap::DISPLAY_COMPOSITOR));
        assert!(trx_cap::DISPLAY.contains(trx_cap::DISPLAY_MODE));
        // Input domain
        assert!(trx_cap::INPUT.contains(trx_cap::INPUT_KEYBOARD));
        assert!(trx_cap::INPUT.contains(trx_cap::INPUT_TOUCH));
        // GPU domain
        assert!(trx_cap::GPU.contains(trx_cap::GPU_RENDER));
        assert!(trx_cap::GPU.contains(trx_cap::GPU_ALLOC));
        // Net domain
        assert!(trx_cap::NET.contains(trx_cap::NET_SOCKET));
        assert!(trx_cap::NET.contains(trx_cap::NET_RAW));
        // Time domain
        assert!(trx_cap::TIME.contains(trx_cap::TIME_READ));
        assert!(trx_cap::TIME.contains(trx_cap::TIME_TIMER));
        // System domain
        assert!(trx_cap::SYSTEM.contains(trx_cap::SYSTEM_REBOOT));
        assert!(trx_cap::SYSTEM.contains(trx_cap::SYSTEM_AUDIT));
    }

    #[test]
    fn trx_capset_root_contains_all_domains() {
        assert!(trx_cap::ROOT.contains(trx_cap::PROCESS));
        assert!(trx_cap::ROOT.contains(trx_cap::MEMORY));
        assert!(trx_cap::ROOT.contains(trx_cap::THREAD));
        assert!(trx_cap::ROOT.contains(trx_cap::IPC));
        assert!(trx_cap::ROOT.contains(trx_cap::FS));
        assert!(trx_cap::ROOT.contains(trx_cap::IO));
        assert!(trx_cap::ROOT.contains(trx_cap::DISPLAY));
        assert!(trx_cap::ROOT.contains(trx_cap::INPUT));
        assert!(trx_cap::ROOT.contains(trx_cap::GPU));
        assert!(trx_cap::ROOT.contains(trx_cap::NET));
        assert!(trx_cap::ROOT.contains(trx_cap::TIME));
        assert!(trx_cap::ROOT.contains(trx_cap::SYSTEM));
    }

    #[test]
    fn trx_capset_none_is_empty() {
        assert!(trx_cap::NONE.is_empty());
        assert!(!trx_cap::ROOT.is_empty());
        assert!(!trx_cap::NONE.contains(trx_cap::PROCESS_CREATE));
    }

    #[test]
    fn trx_capset_domains_no_overlap() {
        let domains = [
            trx_cap::PROCESS,
            trx_cap::MEMORY,
            trx_cap::THREAD,
            trx_cap::IPC,
            trx_cap::FS,
            trx_cap::IO,
            trx_cap::DISPLAY,
            trx_cap::INPUT,
            trx_cap::GPU,
            trx_cap::NET,
            trx_cap::TIME,
            trx_cap::SYSTEM,
        ];
        for (i, a) in domains.iter().enumerate() {
            for b in &domains[i + 1..] {
                let inter = a.intersection(*b);
                assert!(
                    inter.is_empty(),
                    "Domains overlap: ({:#x},{:#x}) & ({:#x},{:#x}) = ({:#x},{:#x})",
                    a.lo, a.hi, b.lo, b.hi, inter.lo, inter.hi
                );
            }
        }
    }

    #[test]
    fn trx_capset_union_intersection_difference() {
        let proc_mem = trx_cap::PROCESS.union(trx_cap::MEMORY);
        assert!(proc_mem.contains(trx_cap::PROCESS_CREATE));
        assert!(proc_mem.contains(trx_cap::MEMORY_DMA));
        assert!(!proc_mem.contains(trx_cap::THREAD_CREATE));

        let just_proc = proc_mem.intersection(trx_cap::PROCESS);
        assert_eq!(just_proc, trx_cap::PROCESS);

        let just_mem = proc_mem.difference(trx_cap::PROCESS);
        assert_eq!(just_mem, trx_cap::MEMORY);
    }

    // -- TerranoxOS data structure tests --

    #[test]
    fn trx_cap_token_size() {
        assert_eq!(mem::size_of::<GenTrxCapToken>(), 16);
    }

    #[test]
    fn trx_cap_token_set_header_size() {
        assert_eq!(mem::size_of::<GenTrxCapTokenSet>(), 8);
    }

    #[test]
    fn trx_process_info_size() {
        assert_eq!(mem::size_of::<GenTrxProcessInfo>(), 40);
    }

    #[test]
    fn trx_display_info_size() {
        assert_eq!(mem::size_of::<GenTrxDisplayInfo>(), 56);
    }

    #[test]
    fn trx_input_event_size() {
        assert_eq!(mem::size_of::<GenTrxInputEvent>(), 24);
    }

    #[test]
    fn trx_touch_event_size() {
        assert_eq!(mem::size_of::<GenTrxTouchEvent>(), 32);
    }

    #[test]
    fn trx_wait_item_size() {
        assert_eq!(mem::size_of::<GenTrxWaitItem>(), 16);
    }

    #[test]
    fn trx_timespec_size() {
        assert_eq!(mem::size_of::<GenTrxTimespec>(), 16);
    }

    #[test]
    fn trx_gpu_info_size() {
        assert_eq!(mem::size_of::<GenTrxGpuInfo>(), 88);
    }

    #[test]
    fn trx_audit_entry_size() {
        assert_eq!(mem::size_of::<GenTrxAuditEntry>(), 48);
    }

    #[test]
    fn trx_structs_alignment() {
        assert!(mem::align_of::<GenTrxCapToken>() >= 8);
        assert!(mem::align_of::<GenTrxProcessInfo>() >= 8);
        assert!(mem::align_of::<GenTrxWaitItem>() >= 4);
        assert!(mem::align_of::<GenTrxTimespec>() >= 8);
        assert!(mem::align_of::<GenTrxAuditEntry>() >= 8);
    }

    #[test]
    fn trx_capset_cross_word_operations() {
        // Combine lo-word and hi-word capabilities
        let mixed = trx_cap::PROCESS_CREATE.union(trx_cap::GPU_RENDER);
        assert!(mixed.contains(trx_cap::PROCESS_CREATE));
        assert!(mixed.contains(trx_cap::GPU_RENDER));
        assert!(!mixed.contains(trx_cap::PROCESS_SIGNAL));
        assert!(!mixed.contains(trx_cap::GPU_COMPUTE));
    }
}
