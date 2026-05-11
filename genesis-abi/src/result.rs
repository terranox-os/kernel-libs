//! Result codes and POSIX errno mapping. Mirrors `genesis_result.h`.

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
    pub const EINTR: i32 = 4;
    pub const EIO: i32 = 5;
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
            -1 => posix_errno::EINVAL,     // INVALID_ARG
            -2 => posix_errno::ENOMEM,     // OUT_OF_MEMORY
            -3 => posix_errno::ENOENT,     // NOT_FOUND
            -4 => posix_errno::EEXIST,     // ALREADY_EXISTS
            -5 => posix_errno::EINVAL,     // BUFFER_TOO_SMALL
            -6 => posix_errno::ENOSYS,     // NOT_SUPPORTED
            -7 => posix_errno::EBUSY,      // BUSY
            -8 => posix_errno::ETIMEDOUT,  // TIMEOUT
            -9 => posix_errno::EINTR,      // INTERRUPTED
            -10 => posix_errno::EINVAL,    // OVERFLOW
            -16 => posix_errno::EPERM,     // PERMISSION_DENIED
            -17 => posix_errno::EACCES,    // ACCESS_VIOLATION
            -18 => posix_errno::EPERM,     // INVALID_CAPABILITY
            -32 => posix_errno::EIO,       // IO
            -34 => posix_errno::EFAULT,    // BAD_ADDRESS
            -33 => posix_errno::ENOENT,    // DEVICE_OFFLINE
            -35 => posix_errno::EPIPE,     // CHANNEL_CLOSED
            -36 => posix_errno::ENOENT,    // DISPLAY_OFFLINE
            -37 => posix_errno::EPIPE,     // GPU_ERROR
            -48 => posix_errno::EINVAL,    // INVALID_FORMAT
            -49 => posix_errno::EINVAL,    // CHECKSUM_MISMATCH
            -50 => posix_errno::EINVAL,    // VERSION_MISMATCH
            -64 => posix_errno::ENOENT,    // MODULE_LOAD_FAILED
            -65 => posix_errno::EPERM,     // MODULE_INIT_FAILED
            -66 => posix_errno::ENOENT,    // MODULE_NOT_FOUND
            -67 => posix_errno::EINVAL,    // MODULE_INCOMPATIBLE
            -80 => posix_errno::ETIMEDOUT, // DEADLINE_MISS
            -81 => posix_errno::EAGAIN,    // PRIORITY_INV
            -82 => posix_errno::ENOMEM,    // STACK_OVERFLOW
            -96 => posix_errno::ENOSYS,    // BAD_SYSCALL
            -97 => posix_errno::EBADF,     // BAD_HANDLE
            -98 => posix_errno::EINTR,     // SYSCALL_INTERRUPTED
            -99 => posix_errno::ENOMEM,    // HANDLE_LIMIT
            _ => posix_errno::EINVAL,
        }
    }

    /// Convert a POSIX errno value to a GenResult.
    pub const fn from_errno(e: i32) -> Self {
        match e {
            0 => Self::OK,
            1 => Self::ERR_PERMISSION_DENIED, // EPERM
            2 => Self::ERR_NOT_FOUND,         // ENOENT
            3 => Self::ERR_NOT_FOUND,         // ESRCH
            9 => Self::ERR_BAD_HANDLE,        // EBADF
            4 => Self::ERR_INTERRUPTED,       // EINTR
            5 => Self::ERR_IO,                // EIO
            11 => Self::ERR_INTERRUPTED,      // EAGAIN
            12 => Self::ERR_OUT_OF_MEMORY,    // ENOMEM
            13 => Self::ERR_ACCESS_VIOLATION, // EACCES
            14 => Self::ERR_BAD_ADDRESS,      // EFAULT
            16 => Self::ERR_BUSY,             // EBUSY
            17 => Self::ERR_ALREADY_EXISTS,   // EEXIST
            22 => Self::ERR_INVALID_ARG,      // EINVAL
            32 => Self::ERR_CHANNEL_CLOSED,   // EPIPE
            38 => Self::ERR_BAD_SYSCALL,      // ENOSYS
            110 => Self::ERR_TIMEOUT,         // ETIMEDOUT
            _ => Self::ERR_NOT_SUPPORTED,
        }
    }
}
