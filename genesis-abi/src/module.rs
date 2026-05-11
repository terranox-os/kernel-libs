//! Module loading ABI: capabilities, descriptor, kernel API table.
//! Mirrors `genesis_module.h`.

use crate::result::GenResult;

pub const MODULE_ABI_VERSION_MAJOR: u16 = 0;
pub const MODULE_ABI_VERSION_MINOR: u16 = 1;

pub const MODULE_NAME_MAX: usize = 64;
pub const MODULE_MAGIC: u32 = 0x47454E4D;
pub const MODULE_SECTION: &str = ".gen_module";

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
