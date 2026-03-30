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
            _ => "UNKNOWN",
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

    // TerranoxOS syscalls (0x0100–0x01FF)
    pub const SYS_CAP_GRANT: GenSyscallNr = GenSyscallNr(0x0100);
    pub const SYS_CAP_REVOKE: GenSyscallNr = GenSyscallNr(0x0101);
    pub const SYS_CAP_CHECK: GenSyscallNr = GenSyscallNr(0x0102);
    pub const SYS_SIGIL_SIGN: GenSyscallNr = GenSyscallNr(0x0103);
    pub const SYS_SIGIL_VERIFY: GenSyscallNr = GenSyscallNr(0x0104);
    pub const SYS_AUDIT_LOG: GenSyscallNr = GenSyscallNr(0x0105);
    pub const SYS_SANDBOX_CREATE: GenSyscallNr = GenSyscallNr(0x0106);
    pub const SYS_SANDBOX_ENTER: GenSyscallNr = GenSyscallNr(0x0107);

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

        assert!(syscall::SYS_CAP_GRANT.is_terranox());
        assert!(!syscall::SYS_CAP_GRANT.is_shared());

        assert!(syscall::SYS_RT_TASK_CREATE.is_genesisrt());
        assert!(!syscall::SYS_RT_TASK_CREATE.is_terranox());

        assert!(syscall::SYS_MOD_LOAD.is_hermetica());
        assert!(!syscall::SYS_MOD_LOAD.is_genesisrt());
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
}
