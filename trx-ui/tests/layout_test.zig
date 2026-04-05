/// Integration tests for the trx-ui layout engine.
///
/// These tests build realistic UI trees and verify the computed
/// layout matches expected pixel positions.

const ui = @import("trx_ui");
const testing = @import("std").testing;

test "dashboard layout: sidebar + content" {
    // Sidebar (200px fixed) + Content (flex-grow)
    var sidebar = ui.Node{
        .tag = .box,
        .style = .{
            .width = .{ .px = 200 },
            .height = .{ .fraction = 1.0 },
            .bg = ui.Color.BG_SECONDARY,
        },
    };
    var content = ui.Node{
        .tag = .box,
        .style = .{
            .flex_grow = 1,
            .height = .{ .fraction = 1.0 },
            .bg = ui.Color.BG_PRIMARY,
        },
    };
    var root = ui.Node{
        .tag = .box,
        .style = .{
            .direction = .row,
            .width = .{ .px = 1024 },
            .height = .{ .px = 768 },
        },
        .children = &.{ &sidebar, &content },
    };

    ui.computeLayout(&root, 1024, 768);

    // Sidebar: x=0, width=200
    try testing.expectEqual(@as(f32, 0), sidebar.layout.x);
    try testing.expectEqual(@as(f32, 200), sidebar.layout.width);

    // Content: x=200, width=824 (1024-200)
    try testing.expectEqual(@as(f32, 200), content.layout.x);
    try testing.expectEqual(@as(f32, 824), content.layout.width);
}

test "toolbar with centered items" {
    var btn1 = ui.Node{
        .tag = .box,
        .style = .{ .width = .{ .px = 40 }, .height = .{ .px = 40 } },
    };
    var btn2 = ui.Node{
        .tag = .box,
        .style = .{ .width = .{ .px = 40 }, .height = .{ .px = 40 } },
    };
    var toolbar = ui.Node{
        .tag = .box,
        .style = .{
            .direction = .row,
            .justify_content = .center,
            .align_items = .center,
            .width = .{ .px = 400 },
            .height = .{ .px = 60 },
            .gap = 10,
        },
        .children = &.{ &btn1, &btn2 },
    };

    ui.computeLayout(&toolbar, 400, 60);

    // Two 40px buttons with 10px gap = 90px total
    // Centered in 400px => offset = (400-90)/2 = 155
    try testing.expectEqual(@as(f32, 155), btn1.layout.x);
    try testing.expectEqual(@as(f32, 205), btn2.layout.x); // 155+40+10

    // Vertically centered in 60px: (60-40)/2 = 10
    try testing.expectEqual(@as(f32, 10), btn1.layout.y);
    try testing.expectEqual(@as(f32, 10), btn2.layout.y);
}

test "vertical list with padding" {
    var item1 = ui.Node{
        .tag = .text,
        .style = .{ .font_size = 16 },
        .text_content = "Item 1",
    };
    var item2 = ui.Node{
        .tag = .text,
        .style = .{ .font_size = 16 },
        .text_content = "Item 2",
    };
    var list = ui.Node{
        .tag = .box,
        .style = .{
            .direction = .column,
            .padding = ui.Edges.all(10),
            .gap = 5,
            .width = .{ .px = 200 },
            .height = .{ .px = 200 },
        },
        .children = &.{ &item1, &item2 },
    };

    ui.computeLayout(&list, 200, 200);

    // Item 1 at (10, 10) — padded from top-left
    try testing.expectEqual(@as(f32, 10), item1.layout.x);
    try testing.expectEqual(@as(f32, 10), item1.layout.y);

    // Item 2 at (10, 31) — 10 padding + 16 text height + 5 gap
    try testing.expectEqual(@as(f32, 10), item2.layout.x);
    try testing.expectEqual(@as(f32, 31), item2.layout.y);
}

test "space_around distributes evenly" {
    var a = ui.Node{ .tag = .box, .style = .{ .width = .{ .px = 50 }, .height = .{ .px = 50 } } };
    var b = ui.Node{ .tag = .box, .style = .{ .width = .{ .px = 50 }, .height = .{ .px = 50 } } };
    var c = ui.Node{ .tag = .box, .style = .{ .width = .{ .px = 50 }, .height = .{ .px = 50 } } };
    var root = ui.Node{
        .tag = .box,
        .style = .{
            .direction = .row,
            .justify_content = .space_around,
            .width = .{ .px = 300 },
            .height = .{ .px = 100 },
        },
        .children = &.{ &a, &b, &c },
    };

    ui.computeLayout(&root, 300, 100);

    // 300 - 150 (3*50) = 150 free space
    // space_around: each item gets 150/3 = 50 slot
    // First item offset = slot/2 = 25
    try testing.expectEqual(@as(f32, 25), a.layout.x);
    try testing.expectEqual(@as(f32, 125), b.layout.x); // 25+50+50
    try testing.expectEqual(@as(f32, 225), c.layout.x); // 125+50+50
}

test "nested layout" {
    var inner_child = ui.Node{
        .tag = .box,
        .style = .{ .width = .{ .px = 30 }, .height = .{ .px = 30 } },
    };
    var inner = ui.Node{
        .tag = .box,
        .style = .{
            .direction = .row,
            .justify_content = .center,
            .width = .{ .px = 100 },
            .height = .{ .px = 100 },
        },
        .children = &.{&inner_child},
    };
    var root = ui.Node{
        .tag = .box,
        .style = .{
            .direction = .row,
            .padding = ui.Edges.all(10),
            .width = .{ .px = 200 },
            .height = .{ .px = 200 },
        },
        .children = &.{&inner},
    };

    ui.computeLayout(&root, 200, 200);

    // Inner box at (10, 10) from padding
    try testing.expectEqual(@as(f32, 10), inner.layout.x);
    try testing.expectEqual(@as(f32, 10), inner.layout.y);

    // Inner child centered: inner.x + (100-30)/2 = 10+35 = 45
    try testing.expectEqual(@as(f32, 45), inner_child.layout.x);
}

test "full render pipeline" {
    var label = ui.Node{
        .tag = .text,
        .style = .{ .font_size = 16, .color = ui.Color.white },
        .text_content = "Hello",
    };
    var container = ui.Node{
        .tag = .box,
        .style = .{
            .direction = .column,
            .align_items = .center,
            .bg = ui.Color.BG_SECONDARY,
            .padding = ui.Edges.all(20),
            .width = .{ .px = 200 },
            .height = .{ .px = 100 },
        },
        .children = &.{&label},
    };

    // Layout
    ui.computeLayout(&container, 200, 100);

    // Render into a small buffer
    var pixels: [200 * 100]u32 = [_]u32{0} ** (200 * 100);
    var fb = ui.Framebuffer.init(&pixels, 200, 100);
    fb.clear(ui.Color.BG_PRIMARY);
    ui.renderTree(&fb, &container);

    // Background should be BG_SECONDARY
    try testing.expectEqual(ui.Color.BG_SECONDARY.toArgb32(), pixels[0]);
}
