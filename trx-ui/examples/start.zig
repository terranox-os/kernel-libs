/// Minimal freestanding entry point for TerranoxOS userspace.
/// Calls main() directly without parsing argc/argv from the stack.

const app = @import("counter.zig");

export fn _start() callconv(.C) noreturn {
    app.main() catch {};
    // exit(0)
    asm volatile ("syscall"
        :
        : [nr] "{rax}" (@as(u64, 60)),
          [a1] "{rdi}" (@as(u64, 0)),
        : "rcx", "r11", "memory"
    );
    unreachable;
}
