/*
 * compile_test.c — Verifies all genesis-abi C headers compile,
 * types have expected sizes, and inline helpers link correctly.
 */

#include "genesis_result.h"
#include "genesis_syscall.h"
#include "genesis_module.h"
#include "genesis_trx_types.h"

/* Type size checks */
_Static_assert(sizeof(GenResult) == 4, "GenResult must be 4 bytes");
_Static_assert(sizeof(GenSyscallNr) == 4, "GenSyscallNr must be 4 bytes");
_Static_assert(sizeof(GenCapability) == 8, "GenCapability must be 8 bytes");

/* Error code value checks — must match Rust mirror */
_Static_assert(GEN_OK == 0, "GEN_OK must be 0");
_Static_assert(GEN_ERR_INVALID_ARG == -1, "GEN_ERR_INVALID_ARG must be -1");
_Static_assert(GEN_ERR_OUT_OF_MEMORY == -2, "GEN_ERR_OUT_OF_MEMORY must be -2");
_Static_assert(GEN_ERR_PERMISSION_DENIED == -16, "GEN_ERR_PERMISSION_DENIED must be -16");
_Static_assert(GEN_ERR_IO == -32, "GEN_ERR_IO must be -32");
_Static_assert(GEN_ERR_CHANNEL_CLOSED == -35, "GEN_ERR_CHANNEL_CLOSED must be -35");
_Static_assert(GEN_ERR_DISPLAY_OFFLINE == -36, "GEN_ERR_DISPLAY_OFFLINE must be -36");
_Static_assert(GEN_ERR_GPU_ERROR == -37, "GEN_ERR_GPU_ERROR must be -37");
_Static_assert(GEN_ERR_INVALID_FORMAT == -48, "GEN_ERR_INVALID_FORMAT must be -48");
_Static_assert(GEN_ERR_MODULE_LOAD_FAILED == -64, "GEN_ERR_MODULE_LOAD_FAILED must be -64");
_Static_assert(GEN_ERR_DEADLINE_MISS == -80, "GEN_ERR_DEADLINE_MISS must be -80");
_Static_assert(GEN_ERR_BAD_SYSCALL == -96, "GEN_ERR_BAD_SYSCALL must be -96");
_Static_assert(GEN_ERR_HANDLE_LIMIT == -99, "GEN_ERR_HANDLE_LIMIT must be -99");

/* Syscall range checks */
_Static_assert(GEN_SYSCALL_SHARED_BASE == 0x0000, "Shared base");
_Static_assert(GEN_SYSCALL_TERRANOX_BASE == 0x0100, "Terranox base");
_Static_assert(GEN_SYSCALL_GENESISRT_BASE == 0x0200, "GenesisRT base");
_Static_assert(GEN_SYSCALL_HERMETICA_BASE == 0x0300, "Hermetica base");

/* Module magic */
_Static_assert(GEN_MODULE_MAGIC == 0x47454E4DU, "Module magic must be GENM");

/* GEN_STRUCT_HAS_FIELD compiles */
_Static_assert(
    GEN_STRUCT_HAS_FIELD(&(GenModuleDescriptor){0}, magic),
    "magic field must exist in GenModuleDescriptor"
);
_Static_assert(
    GEN_STRUCT_HAS_FIELD(&(GenModuleDescriptor){0}, fini),
    "fini field must exist in GenModuleDescriptor"
);

/* Capability bits are single-bit powers of two */
_Static_assert(GEN_CAP_MEM_READ == (1ULL << 0), "MEM_READ bit");
_Static_assert(GEN_CAP_CRYPTO == (1ULL << 15), "CRYPTO bit");

/* TrxCapSet size check */
_Static_assert(sizeof(TrxCapSet) == 16, "TrxCapSet must be 16 bytes");

/* TerranoxOS data structure size checks */
_Static_assert(sizeof(GenTrxCapToken) == 16, "GenTrxCapToken must be 16 bytes");
_Static_assert(sizeof(GenTrxCapTokenSet) == 8, "GenTrxCapTokenSet header must be 8 bytes");
_Static_assert(sizeof(GenTrxProcessInfo) == 40, "GenTrxProcessInfo must be 40 bytes");
_Static_assert(sizeof(GenTrxDisplayInfo) == 56, "GenTrxDisplayInfo must be 56 bytes");
_Static_assert(sizeof(GenTrxInputEvent) == 24, "GenTrxInputEvent must be 24 bytes");
_Static_assert(sizeof(GenTrxTouchEvent) == 32, "GenTrxTouchEvent must be 32 bytes");
_Static_assert(sizeof(GenTrxWaitItem) == 16, "GenTrxWaitItem must be 16 bytes");
_Static_assert(sizeof(GenTrxTimespec) == 16, "GenTrxTimespec must be 16 bytes");
_Static_assert(sizeof(GenTrxGpuInfo) == 88, "GenTrxGpuInfo must be 88 bytes");
_Static_assert(sizeof(GenTrxAuditEntry) == 48, "GenTrxAuditEntry must be 48 bytes");

int main(void)
{
    /* Exercise inline helpers to verify they link */
    GenResult r = GEN_OK;
    (void)gen_result_is_ok(r);
    (void)gen_result_is_error(r);
    (void)gen_result_name(r);
    (void)gen_result_name(GEN_ERR_INVALID_ARG);

    (void)gen_syscall_is_shared(GEN_SYS_EXIT);
    (void)gen_syscall_is_terranox(GEN_SYS_TRX_PROCESS_CAP_GRANT);
    (void)gen_syscall_is_genesisrt(GEN_SYS_RT_TASK_CREATE);
    (void)gen_syscall_is_hermetica(GEN_SYS_MOD_LOAD);

    /* Deprecated aliases still work */
    if (GEN_SYS_CAP_GRANT != GEN_SYS_TRX_PROCESS_CAP_GRANT) return 1;
    if (GEN_SYS_AUDIT_LOG != GEN_SYS_TRX_AUDIT_READ) return 1;

    /* Verify subsystem classification */
    if (gen_syscall_trx_subsystem(GEN_SYS_TRX_PROCESS_CREATE) != 0) return 1;
    if (gen_syscall_trx_subsystem(GEN_SYS_TRX_GPU_OPEN) != 7) return 1;
    if (gen_syscall_trx_subsystem(GEN_SYS_TRX_SIGIL_SIGN) != 11) return 1;
    if (gen_syscall_trx_subsystem(GEN_SYS_EXIT) != -1) return 1;

    /* Spot-check new syscall values */
    if (!gen_syscall_is_terranox(GEN_SYS_TRX_CHANNEL_CREATE)) return 1;
    if (!gen_syscall_is_terranox(GEN_SYS_TRX_NET_SOCKET)) return 1;
    if (!gen_syscall_is_terranox(GEN_SYS_TRX_DISPLAY_ENUMERATE)) return 1;

    (void)gen_cap_contains(GEN_CAP_ALL, GEN_CAP_MEM_READ);
    (void)gen_cap_contains(GEN_CAP_NONE, GEN_CAP_MEM_READ);

    /* Exercise errno mapping helpers */
    if (gen_result_to_errno(GEN_OK) != 0) return 1;
    if (gen_result_to_errno(GEN_ERR_INVALID_ARG) != GEN_POSIX_EINVAL) return 1;
    if (gen_result_to_errno(GEN_ERR_BAD_HANDLE) != GEN_POSIX_EBADF) return 1;
    if (gen_result_from_errno(0) != GEN_OK) return 1;
    if (gen_result_from_errno(GEN_POSIX_ENOMEM) != GEN_ERR_OUT_OF_MEMORY) return 1;

    /* Exercise TrxCapSet helpers */
    TrxCapSet proc = TRX_CAP_PROCESS;
    TrxCapSet proc_create = TRX_CAP_PROCESS_CREATE;
    (void)trx_cap_contains(proc, proc_create);
    (void)trx_cap_is_empty(TRX_CAP_NONE);

    TrxCapSet u = trx_cap_union(TRX_CAP_PROCESS, TRX_CAP_MEMORY);
    TrxCapSet i = trx_cap_intersection(u, TRX_CAP_PROCESS);
    TrxCapSet d = trx_cap_difference(u, TRX_CAP_PROCESS);
    (void)u; (void)i; (void)d;

    /* Verify ROOT contains all leaf domains */
    TrxCapSet root = TRX_CAP_ROOT;
    if (!trx_cap_contains(root, TRX_CAP_PROCESS)) return 1;
    if (!trx_cap_contains(root, TRX_CAP_GPU))     return 1;
    if (!trx_cap_contains(root, TRX_CAP_SYSTEM))  return 1;

    /* Verify NONE is empty */
    if (!trx_cap_is_empty(TRX_CAP_NONE)) return 1;
    if (trx_cap_is_empty(TRX_CAP_ROOT))  return 1;

    return 0;
}
