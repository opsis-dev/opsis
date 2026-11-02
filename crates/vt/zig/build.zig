const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});
    const ghostty = b.dependency("ghostty", .{ .target = target, .optimize = optimize });
    const ghostty_vt = ghostty.module("ghostty-vt");

    const root = b.createModule(.{
        .root_source_file = b.path("src/vtwrap.zig"),
        .target = target,
        .optimize = optimize,
        .link_libc = true,
        .pic = true,
    });
    root.addImport("ghostty-vt", ghostty_vt);

    const lib = b.addLibrary(.{ .name = "opsis_vtwrap", .linkage = .static, .root_module = root });
    lib.bundle_compiler_rt = true;
    lib.bundle_ubsan_rt = true;
    if (target.result.os.tag == .windows) {
        lib.bundle_ubsan_rt = false;
        if (target.result.abi == .msvc) root.stack_protector = false;
    }
    b.installArtifact(lib);

    for (ghostty_vt.link_objects.items) |object| switch (object) {
        .other_step => |linked| if (linked.isStaticLibrary()) b.installArtifact(linked),
        else => {},
    };
}
