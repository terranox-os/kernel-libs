/// trx-ui — TerranoxOS UI toolkit.
///
/// Provides declarative UI construction, flexbox layout, software
/// rendering, input handling, and damage tracking for native
/// TerranoxOS applications.
///
/// ```zig
/// const ui = @import("trx_ui");
///
/// fn build() *ui.Node {
///     return &ui.box(.{
///         .style = .{ .direction = .column, .bg = ui.color.BG_PRIMARY },
///         .children = &.{
///             &ui.text("Hello TerranoxOS", .{ .font_size = 32 }),
///         },
///     });
/// }
///
/// pub fn main() !void {
///     var app = ui.App.init("Demo", 800, 600, &build);
///     try app.setup();
///     app.run();
///     app.shutdown();
/// }
/// ```

// Re-export public API

pub const color = @import("color.zig");
pub const Color = color.Color;

pub const style = @import("style.zig");
pub const Style = style.Style;
pub const Dimension = style.Dimension;
pub const Edges = style.Edges;
pub const FlexDirection = style.FlexDirection;
pub const AlignItems = style.AlignItems;
pub const JustifyContent = style.JustifyContent;
pub const Overflow = style.Overflow;

const node_mod = @import("node.zig");
pub const Node = node_mod.Node;
pub const NodeTag = node_mod.NodeTag;
pub const LayoutResult = node_mod.LayoutResult;
pub const BoxOptions = node_mod.BoxOptions;
pub const TextOptions = node_mod.TextOptions;
pub const ScrollState = node_mod.ScrollState;
pub const box = node_mod.box;
pub const text = node_mod.text;

pub const layout = @import("layout.zig");
pub const computeLayout = layout.computeLayout;

pub const text_render = @import("text.zig");
pub const measureTextWidth = text_render.measureTextWidth;
pub const measureTextWrapped = text_render.measureTextWrapped;
pub const renderTextWrapped = text_render.renderTextWrapped;

pub const render = @import("render.zig");
pub const Framebuffer = render.Framebuffer;
pub const ClipRect = render.ClipRect;
pub const renderTree = render.renderTree;
pub const GpuRenderState = render.GpuRenderState;
pub const renderTreeGpu = render.renderTreeGpu;

pub const damage = @import("damage.zig");
pub const DamageTracker = damage.DamageTracker;
pub const Rect = damage.Rect;

pub const input = @import("input.zig");
pub const InputEvent = input.InputEvent;
pub const EventType = input.EventType;
pub const FocusState = input.FocusState;
pub const Key = input.Key;
pub const hitTest = input.hitTest;
pub const dispatchClick = input.dispatchClick;
pub const dispatchScroll = input.dispatchScroll;

pub const platform = @import("platform.zig");
pub const SYS = platform.SYS;

pub const gpu = @import("gpu.zig");
pub const GpuContext = gpu.GpuContext;
pub const GpuBuffer = gpu.GpuBuffer;
pub const GpuError = gpu.GpuError;
pub const BufferUsage = gpu.BufferUsage;

pub const atlas = @import("atlas.zig");
pub const GlyphAtlas = atlas.GlyphAtlas;
pub const GlyphUV = atlas.GlyphUV;

pub const batch = @import("batch.zig");
pub const Batch = batch.Batch;
pub const CmdDrawRect = batch.CmdDrawRect;
pub const CmdDrawGlyph = batch.CmdDrawGlyph;
pub const CmdSetClip = batch.CmdSetClip;

const app_mod = @import("app.zig");
pub const App = app_mod.App;
pub const GpuConfig = app_mod.GpuConfig;

// Force analysis of all modules for test discovery
comptime {
    _ = @import("color.zig");
    _ = @import("style.zig");
    _ = @import("node.zig");
    _ = @import("layout.zig");
    _ = @import("text.zig");
    _ = @import("render.zig");
    _ = @import("damage.zig");
    _ = @import("input.zig");
    _ = @import("platform.zig");
    _ = @import("gpu.zig");
    _ = @import("atlas.zig");
    _ = @import("batch.zig");
    _ = @import("app.zig");
}
