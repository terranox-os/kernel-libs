const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    // --- static library (freestanding or host) ---
    const lib = b.addStaticLibrary(.{
        .name = "trx_ui",
        .root_source_file = b.path("src/root.zig"),
        .target = target,
        .optimize = optimize,
    });
    lib.addIncludePath(b.path("../genesis-abi/include"));
    b.installArtifact(lib);

    // --- unit tests (always host-native) ---
    const unit_tests = b.addTest(.{
        .root_source_file = b.path("src/root.zig"),
        .target = target,
        .optimize = optimize,
    });
    unit_tests.addIncludePath(b.path("../genesis-abi/include"));

    const run_unit_tests = b.addRunArtifact(unit_tests);
    const test_step = b.step("test", "Run trx-ui unit tests");
    test_step.dependOn(&run_unit_tests.step);

    // --- layout integration tests ---
    const layout_tests = b.addTest(.{
        .root_source_file = b.path("tests/layout_test.zig"),
        .target = target,
        .optimize = optimize,
    });
    layout_tests.addIncludePath(b.path("../genesis-abi/include"));
    layout_tests.root_module.addImport("trx_ui", lib.root_module);

    const run_layout_tests = b.addRunArtifact(layout_tests);
    test_step.dependOn(&run_layout_tests.step);

    // --- examples ---
    const counter_example = b.addExecutable(.{
        .name = "counter",
        .root_source_file = b.path("examples/counter.zig"),
        .target = target,
        .optimize = optimize,
    });
    counter_example.root_module.addImport("trx_ui", lib.root_module);

    const install_example = b.addInstallArtifact(counter_example, .{});
    const example_step = b.step("example", "Build the counter example");
    example_step.dependOn(&install_example.step);
}
