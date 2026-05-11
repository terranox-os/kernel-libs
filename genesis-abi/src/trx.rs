//! TerranoxOS-specific types: hierarchical capability model + syscall data structures.
//! Mirrors `genesis_trx_types.h`.
//!
//! The `trx_cap` inner module preserves the v0.1.0 path
//! `genesis_abi::trx_cap::PROCESS_CREATE` (re-exported from `lib.rs`).

// ────────────────────────────────────────────────────────────
// TrxCapSet — hierarchical capability model
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
    pub const IO: TrxCapSet = TrxCapSet {
        lo: 0x1C_0000,
        hi: 0,
    };

    // ── display domain (lo bits 21-24) ──
    pub const DISPLAY_COMPOSITOR: TrxCapSet = TrxCapSet { lo: 1 << 21, hi: 0 };
    pub const DISPLAY_SURFACE: TrxCapSet = TrxCapSet { lo: 1 << 22, hi: 0 };
    pub const DISPLAY_BUFFER: TrxCapSet = TrxCapSet { lo: 1 << 23, hi: 0 };
    pub const DISPLAY_MODE: TrxCapSet = TrxCapSet { lo: 1 << 24, hi: 0 };
    pub const DISPLAY: TrxCapSet = TrxCapSet {
        lo: 0x1E0_0000,
        hi: 0,
    };

    // ── input domain (lo bits 25-27) ──
    pub const INPUT_KEYBOARD: TrxCapSet = TrxCapSet { lo: 1 << 25, hi: 0 };
    pub const INPUT_POINTER: TrxCapSet = TrxCapSet { lo: 1 << 26, hi: 0 };
    pub const INPUT_TOUCH: TrxCapSet = TrxCapSet { lo: 1 << 27, hi: 0 };
    pub const INPUT: TrxCapSet = TrxCapSet {
        lo: 0xE00_0000,
        hi: 0,
    };

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
    pub const ROOT: TrxCapSet = TrxCapSet {
        lo: 0xFFF_FFFF,
        hi: 0xFFF,
    };
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

/// Compositor layer for `trx_compositor_present`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct GenTrxLayer {
    pub surface_handle: i64,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub z_order: i32,
    pub flags: u32,
}

pub const GEN_TRX_LAYER_OPAQUE: u32 = 1 << 0;
pub const GEN_TRX_LAYER_CURSOR: u32 = 1 << 1;
