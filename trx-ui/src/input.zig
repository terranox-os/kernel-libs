/// Input event handling, hit-testing, and focus management.

const node_mod = @import("node.zig");
pub const Node = node_mod.Node;
pub const LayoutResult = node_mod.LayoutResult;

/// Input event types (matches GenTrxInputEvent.type).
pub const EventType = enum(u32) {
    key = 1,
    rel = 2, // relative movement (mouse)
    abs = 3, // absolute position (touch)
};

/// An input event from the platform.
pub const InputEvent = struct {
    timestamp_ns: u64 = 0,
    event_type: EventType = .key,
    code: u32 = 0,
    value: i32 = 0,
    device_id: u32 = 0,
};

/// Well-known key codes (subset matching Linux input-event-codes).
pub const Key = struct {
    pub const ESC: u32 = 1;
    pub const ENTER: u32 = 28;
    pub const SPACE: u32 = 57;
    pub const TAB: u32 = 15;
    pub const BACKSPACE: u32 = 14;
    pub const UP: u32 = 103;
    pub const DOWN: u32 = 108;
    pub const LEFT: u32 = 105;
    pub const RIGHT: u32 = 106;
    pub const A: u32 = 30;
    pub const BTN_LEFT: u32 = 0x110;
    pub const BTN_RIGHT: u32 = 0x111;
};

/// Focus state for keyboard navigation.
pub const FocusState = struct {
    focused_node: ?*const Node = null,

    pub fn setFocus(self: *FocusState, node: *const Node) void {
        self.focused_node = node;
    }

    pub fn clearFocus(self: *FocusState) void {
        self.focused_node = null;
    }

    pub fn hasFocus(self: *const FocusState, node: *const Node) bool {
        if (self.focused_node) |f| {
            return f == node;
        }
        return false;
    }
};

/// Hit-test: find the deepest node at pixel coordinates (px, py).
/// Returns null if no node contains the point.
pub fn hitTest(root: *const Node, px: f32, py: f32) ?*const Node {
    return hitTestRecursive(root, px, py);
}

fn hitTestRecursive(node: *const Node, px: f32, py: f32) ?*const Node {
    const l = node.layout;
    // Check if point is inside this node's bounds
    if (px < l.x or py < l.y or
        px >= l.x + l.width or py >= l.y + l.height)
    {
        return null;
    }

    // Check children in reverse order (front-to-back)
    if (node.children.len > 0) {
        var i = node.children.len;
        while (i > 0) {
            i -= 1;
            if (hitTestRecursive(node.children[i], px, py)) |hit| {
                return hit;
            }
        }
    }

    // This node contains the point and no child claimed it
    return node;
}

/// Dispatch a click event to the node at (px, py) in the tree.
/// Returns true if a handler was invoked.
pub fn dispatchClick(root: *const Node, px: f32, py: f32) bool {
    const target = hitTest(root, px, py) orelse return false;
    if (target.on_click) |handler| {
        handler();
        return true;
    }
    return false;
}

// ── Tests ──────────────────────────────────────────────────────────

const testing = @import("std").testing;

test "hitTest finds node at point" {
    const child = Node{
        .tag = .box,
        .layout = .{ .x = 10, .y = 10, .width = 50, .height = 50 },
    };
    const root = Node{
        .tag = .box,
        .layout = .{ .x = 0, .y = 0, .width = 100, .height = 100 },
        .children = &.{&child},
    };
    // Point inside child
    const hit = hitTest(&root, 25, 25);
    try testing.expect(hit != null);
    try testing.expectEqual(&child, hit.?);
}

test "hitTest returns root when no child matches" {
    const child = Node{
        .tag = .box,
        .layout = .{ .x = 10, .y = 10, .width = 50, .height = 50 },
    };
    const root = Node{
        .tag = .box,
        .layout = .{ .x = 0, .y = 0, .width = 100, .height = 100 },
        .children = &.{&child},
    };
    // Point outside child but inside root
    const hit = hitTest(&root, 80, 80);
    try testing.expect(hit != null);
    try testing.expectEqual(&root, hit.?);
}

test "hitTest returns null outside bounds" {
    const root = Node{
        .tag = .box,
        .layout = .{ .x = 0, .y = 0, .width = 100, .height = 100 },
    };
    try testing.expectEqual(@as(?*const Node, null), hitTest(&root, 200, 200));
}

test "focus state" {
    var focus = FocusState{};
    const n = Node{ .tag = .box };
    try testing.expect(!focus.hasFocus(&n));
    focus.setFocus(&n);
    try testing.expect(focus.hasFocus(&n));
    focus.clearFocus();
    try testing.expect(!focus.hasFocus(&n));
}

var click_count: u32 = 0;

fn testClickHandler() void {
    click_count += 1;
}

test "dispatchClick invokes handler" {
    click_count = 0;
    const root = Node{
        .tag = .box,
        .layout = .{ .x = 0, .y = 0, .width = 100, .height = 100 },
        .on_click = &testClickHandler,
    };
    const dispatched = dispatchClick(&root, 50, 50);
    try testing.expect(dispatched);
    try testing.expectEqual(@as(u32, 1), click_count);
}

test "dispatchClick returns false when no handler" {
    const root = Node{
        .tag = .box,
        .layout = .{ .x = 0, .y = 0, .width = 100, .height = 100 },
    };
    try testing.expect(!dispatchClick(&root, 50, 50));
}
