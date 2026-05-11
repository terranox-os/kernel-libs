//! Freestanding ABI types for TerranoxOS, GenesisOS-RT, and HermeticaOS.
//!
//! This crate mirrors the C headers in `include/`. CI checks must ensure
//! the Rust definitions do not drift from the C source of truth.
//!
//! Module layout mirrors the C header structure:
//! - [`result`]  ↔ `genesis_result.h`     (errors + POSIX errno mapping)
//! - [`syscall`] ↔ `genesis_syscall.h`    (syscall numbers + range helpers)
//! - [`module`]  ↔ `genesis_module.h`     (capabilities + module ABI)
//! - [`trx`]     ↔ `genesis_trx_types.h`  (TerranoxOS-specific types + TrxCapSet)
//!
//! Crate-root re-exports below preserve the v0.1.0 public API: existing
//! callers like `genesis_abi::GenResult`, `genesis_abi::syscall::SYS_EXIT`,
//! `genesis_abi::posix_errno::EINVAL`, and `genesis_abi::trx_cap::PROCESS`
//! continue to resolve unchanged.

#![no_std]
#![forbid(unsafe_op_in_unsafe_fn)]

pub mod module;
pub mod result;
pub mod syscall;
pub mod trx;

pub use module::{
    GenCapability, GenKernelAPI, GenModuleDescriptor, MODULE_ABI_VERSION_MAJOR,
    MODULE_ABI_VERSION_MINOR, MODULE_MAGIC, MODULE_NAME_MAX, MODULE_SECTION,
};
pub use result::{posix_errno, GenResult};
pub use syscall::GenSyscallNr;
pub use trx::{
    trx_cap, GenTrxAuditEntry, GenTrxCapToken, GenTrxCapTokenSet, GenTrxDisplayInfo, GenTrxGpuInfo,
    GenTrxInputEvent, GenTrxLayer, GenTrxProcessInfo, GenTrxTimespec, GenTrxTouchEvent,
    GenTrxWaitItem, TrxCapSet, GEN_TRX_LAYER_CURSOR, GEN_TRX_LAYER_OPAQUE,
};

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
        assert_eq!(
            GenResult::from_errno(posix_errno::EINVAL),
            GenResult::ERR_INVALID_ARG
        );
        assert_eq!(
            GenResult::from_errno(posix_errno::ENOMEM),
            GenResult::ERR_OUT_OF_MEMORY
        );
        assert_eq!(
            GenResult::from_errno(posix_errno::EPERM),
            GenResult::ERR_PERMISSION_DENIED
        );
        assert_eq!(
            GenResult::from_errno(posix_errno::EBADF),
            GenResult::ERR_BAD_HANDLE
        );
        assert_eq!(
            GenResult::from_errno(posix_errno::ETIMEDOUT),
            GenResult::ERR_TIMEOUT
        );
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
            syscall::SYS_TRX_PROCESS_CREATE,
            syscall::SYS_TRX_PROCESS_KILL,
            syscall::SYS_TRX_PROCESS_INFO,
            syscall::SYS_TRX_PROCESS_CAP_GRANT,
            syscall::SYS_TRX_PROCESS_CAP_REVOKE,
            syscall::SYS_TRX_PROCESS_CAP_QUERY,
            // Thread
            syscall::SYS_TRX_THREAD_CREATE,
            syscall::SYS_TRX_THREAD_EXIT,
            syscall::SYS_TRX_THREAD_JOIN,
            syscall::SYS_TRX_THREAD_SET_AFFINITY,
            syscall::SYS_TRX_THREAD_GET_AFFINITY,
            syscall::SYS_TRX_THREAD_SET_NAME,
            syscall::SYS_TRX_FUTEX_WAIT,
            syscall::SYS_TRX_FUTEX_WAKE,
            // Memory
            syscall::SYS_TRX_MEM_PROTECT,
            syscall::SYS_TRX_MEM_MAP,
            syscall::SYS_TRX_MEM_UNMAP,
            syscall::SYS_TRX_MEM_SHARE_CREATE,
            syscall::SYS_TRX_MEM_SHARE_MAP,
            syscall::SYS_TRX_MEM_SHARE_UNMAP,
            syscall::SYS_TRX_MEM_DMA_ALLOC,
            syscall::SYS_TRX_MEM_DMA_FREE,
            // IPC
            syscall::SYS_TRX_CHANNEL_CREATE,
            syscall::SYS_TRX_CHANNEL_SEND,
            syscall::SYS_TRX_CHANNEL_RECV,
            syscall::SYS_TRX_CHANNEL_CLOSE,
            syscall::SYS_TRX_CHANNEL_POLL,
            syscall::SYS_TRX_SIGNAL_CREATE,
            syscall::SYS_TRX_SIGNAL_RAISE,
            syscall::SYS_TRX_SIGNAL_WAIT,
            syscall::SYS_TRX_SIGNAL_CLEAR,
            syscall::SYS_TRX_EVENT_WAIT_MANY,
            // FS
            syscall::SYS_TRX_FS_MKDIR,
            syscall::SYS_TRX_FS_UNLINK,
            syscall::SYS_TRX_FS_RENAME,
            // Display
            syscall::SYS_TRX_DISPLAY_ENUMERATE,
            syscall::SYS_TRX_DISPLAY_SET_MODE,
            syscall::SYS_TRX_COMPOSITOR_CREATE,
            syscall::SYS_TRX_COMPOSITOR_PRESENT,
            syscall::SYS_TRX_SURFACE_CREATE,
            syscall::SYS_TRX_SURFACE_DESTROY,
            syscall::SYS_TRX_SURFACE_RESIZE,
            syscall::SYS_TRX_BUFFER_CREATE,
            syscall::SYS_TRX_BUFFER_MAP,
            syscall::SYS_TRX_BUFFER_UNMAP,
            // Input
            syscall::SYS_TRX_INPUT_ENUMERATE,
            syscall::SYS_TRX_INPUT_OPEN,
            syscall::SYS_TRX_INPUT_CLOSE,
            syscall::SYS_TRX_INPUT_READ_EVENTS,
            syscall::SYS_TRX_INPUT_GRAB,
            syscall::SYS_TRX_INPUT_UNGRAB,
            syscall::SYS_TRX_INPUT_SET_KEYMAP,
            syscall::SYS_TRX_TOUCH_READ_EVENTS,
            syscall::SYS_TRX_INPUT_SET_ACCEL,
            // GPU
            syscall::SYS_TRX_GPU_OPEN,
            syscall::SYS_TRX_GPU_CLOSE,
            syscall::SYS_TRX_GPU_ALLOC_BO,
            syscall::SYS_TRX_GPU_FREE_BO,
            syscall::SYS_TRX_GPU_MAP_BO,
            syscall::SYS_TRX_GPU_SUBMIT,
            syscall::SYS_TRX_GPU_WAIT_FENCE,
            syscall::SYS_TRX_GPU_EXPORT_DMABUF,
            syscall::SYS_TRX_GPU_IMPORT_DMABUF,
            syscall::SYS_TRX_GPU_GET_INFO,
            // Net
            syscall::SYS_TRX_NET_SOCKET,
            syscall::SYS_TRX_NET_BIND,
            syscall::SYS_TRX_NET_LISTEN,
            syscall::SYS_TRX_NET_ACCEPT,
            syscall::SYS_TRX_NET_CONNECT,
            syscall::SYS_TRX_NET_SENDMSG,
            syscall::SYS_TRX_NET_RECVMSG,
            // Time
            syscall::SYS_TRX_TIMER_CREATE,
            syscall::SYS_TRX_TIMER_SET,
            // System
            syscall::SYS_TRX_SYSTEM_REBOOT,
            syscall::SYS_TRX_MODULE_LOAD,
            syscall::SYS_TRX_MODULE_UNLOAD,
            syscall::SYS_TRX_AUDIT_READ,
            syscall::SYS_TRX_AUDIT_SET_POLICY,
            syscall::SYS_TRX_AUDIT_WRITE,
            // Sigil/sandbox
            syscall::SYS_TRX_SIGIL_SIGN,
            syscall::SYS_TRX_SIGIL_VERIFY,
            syscall::SYS_TRX_SANDBOX_CREATE,
            syscall::SYS_TRX_SANDBOX_ENTER,
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
        let trx_nrs: [u32; 83] = [
            0x0100, 0x0103, 0x0104, 0x0105, 0x0106, 0x0107, 0x0110, 0x0111, 0x0112, 0x0114, 0x0115,
            0x0116, 0x0117, 0x0118, 0x0122, 0x0123, 0x0124, 0x0125, 0x0126, 0x0127, 0x0128, 0x0129,
            0x0130, 0x0131, 0x0132, 0x0133, 0x0134, 0x0135, 0x0136, 0x0137, 0x0138, 0x0139, 0x0147,
            0x0148, 0x0149, 0x0150, 0x0151, 0x0152, 0x0153, 0x0154, 0x0155, 0x0156, 0x0157, 0x0158,
            0x0159, 0x0160, 0x0161, 0x0162, 0x0163, 0x0164, 0x0165, 0x0166, 0x0167, 0x0168, 0x0170,
            0x0171, 0x0172, 0x0173, 0x0174, 0x0175, 0x0176, 0x0177, 0x0178, 0x0179, 0x0180, 0x0181,
            0x0182, 0x0183, 0x0184, 0x0185, 0x0186, 0x0192, 0x0193, 0x01A0, 0x01A1, 0x01A2, 0x01A3,
            0x01A4, 0x01A5, 0x01B0, 0x01B1, 0x01B2, 0x01B3,
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
        assert_eq!(syscall::SYS_AUDIT_LOG, syscall::SYS_TRX_AUDIT_WRITE);
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
            (
                trx_cap::DISPLAY_COMPOSITOR.lo,
                trx_cap::DISPLAY_COMPOSITOR.hi,
            ),
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
                assert!(a != b, "Duplicate leaf cap: lo={:#x} hi={:#x}", a.0, a.1);
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
    fn trx_capset_root_is_complete() {
        let computed = trx_cap::PROCESS
            .union(trx_cap::MEMORY)
            .union(trx_cap::THREAD)
            .union(trx_cap::IPC)
            .union(trx_cap::FS)
            .union(trx_cap::IO)
            .union(trx_cap::DISPLAY)
            .union(trx_cap::INPUT)
            .union(trx_cap::GPU)
            .union(trx_cap::NET)
            .union(trx_cap::TIME)
            .union(trx_cap::SYSTEM);
        assert_eq!(
            computed,
            trx_cap::ROOT,
            "ROOT must equal union of all 12 domain parents"
        );
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
                    a.lo,
                    a.hi,
                    b.lo,
                    b.hi,
                    inter.lo,
                    inter.hi
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
    fn trx_layer_size() {
        assert_eq!(mem::size_of::<GenTrxLayer>(), 32);
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
