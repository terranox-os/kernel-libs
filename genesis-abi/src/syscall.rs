//! Syscall numbers and range helpers. Mirrors `genesis_syscall.h`.
//!
//! The `SYS_*` consts and range helpers live directly in this module so that
//! `genesis_abi::syscall::SYS_EXIT` (the v0.1.0 path) keeps resolving.

/// Syscall number type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct GenSyscallNr(pub u32);

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
pub const SYS_TRX_AUDIT_WRITE: GenSyscallNr = GenSyscallNr(0x01A5);

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
pub const SYS_AUDIT_LOG: GenSyscallNr = SYS_TRX_AUDIT_WRITE; // LOG is a write operation
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

impl GenSyscallNr {
    #[inline]
    pub const fn is_shared(self) -> bool {
        self.0 < SHARED_LIMIT.0
    }

    #[inline]
    pub const fn is_terranox(self) -> bool {
        self.0 >= TERRANOX_BASE.0 && self.0 < TERRANOX_LIMIT.0
    }

    #[inline]
    pub const fn is_genesisrt(self) -> bool {
        self.0 >= GENESISRT_BASE.0 && self.0 < GENESISRT_LIMIT.0
    }

    #[inline]
    pub const fn is_hermetica(self) -> bool {
        self.0 >= HERMETICA_BASE.0 && self.0 < HERMETICA_LIMIT.0
    }

    /// Returns the TerranoxOS subsystem index (0-15), or -1 if not TerranoxOS.
    #[inline]
    pub const fn trx_subsystem(self) -> i32 {
        if !self.is_terranox() {
            return -1;
        }
        ((self.0 - TERRANOX_BASE.0) >> 4) as i32
    }
}
