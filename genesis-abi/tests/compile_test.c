/*
 * compile_test.c — Verifies all genesis-abi C headers compile,
 * types have expected sizes, and inline helpers link correctly.
 */

#include "genesis_result.h"
#include "genesis_syscall.h"
#include "genesis_module.h"

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
_Static_assert(GEN_ERR_INVALID_FORMAT == -48, "GEN_ERR_INVALID_FORMAT must be -48");
_Static_assert(GEN_ERR_MODULE_LOAD_FAILED == -64, "GEN_ERR_MODULE_LOAD_FAILED must be -64");

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

int main(void)
{
    /* Exercise inline helpers to verify they link */
    GenResult r = GEN_OK;
    (void)gen_result_is_ok(r);
    (void)gen_result_is_error(r);
    (void)gen_result_name(r);
    (void)gen_result_name(GEN_ERR_INVALID_ARG);

    (void)gen_syscall_is_shared(GEN_SYS_EXIT);
    (void)gen_syscall_is_terranox(GEN_SYS_CAP_GRANT);
    (void)gen_syscall_is_genesisrt(GEN_SYS_RT_TASK_CREATE);
    (void)gen_syscall_is_hermetica(GEN_SYS_MOD_LOAD);

    (void)gen_cap_contains(GEN_CAP_ALL, GEN_CAP_MEM_READ);
    (void)gen_cap_contains(GEN_CAP_NONE, GEN_CAP_MEM_READ);

    return 0;
}
