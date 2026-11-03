const std = @import("std");
const vt = @import("ghostty-vt");

const Stream = vt.TerminalStream;
const Handler = Stream.Handler;
const RenderState = vt.RenderState;
const Style = vt.Style;
const RGB = vt.color.RGB;
const input = vt.input;
const MouseSize = @FieldType(input.MouseEncodeOptions, "size");
const Coordinate = vt.Coordinate;

pub const std_options: std.Options = .{ .logFn = discardLog };

fn discardLog(
    comptime level: std.log.Level,
    comptime scope: @EnumLiteral(),
    comptime format: []const u8,
    args: anytype,
) void {
    _ = level;
    _ = scope;
    _ = format;
    _ = args;
}

const abi_version: u32 = 2;
const palette_entries = 259;

const tag_palette: u32 = 1 << 24;
const tag_rgb: u32 = 2 << 24;

pub const CellOut = extern struct {
    cp: u32,
    extra_start: u32,
    extra_len: u16,
    width: u8,
    underline: u8,
    fg: u32,
    bg: u32,
    underline_colour: u32,
    flags: u16,
};

pub const RowOut = extern struct {
    serial: u64,
    y: u16,
    dirty: u8,
    wrapped: u8,
};

pub const FrameOut = extern struct {
    default_fg: u32,
    default_bg: u32,
    cells_written: u32,
    graphemes_written: u32,
    cols: u16,
    rows: u16,
    cursor_x: u16,
    cursor_y: u16,
    cursor_visible: u8,
    cursor_shape: u8,
    cursor_blink: u8,
    held: u8,
    palette_changed: u8,
    reverse: u8,
};

pub const EventOut = extern struct {
    code: i32,
    kind: u8,
    has_code: u8,
};

pub const Sizes = extern struct {
    cell: u32,
    row: u32,
    frame: u32,
    event: u32,
};

const Flag = struct {
    const bold: u16 = 1 << 0;
    const italic: u16 = 1 << 1;
    const faint: u16 = 1 << 2;
    const blink: u16 = 1 << 3;
    const inverse: u16 = 1 << 4;
    const invisible: u16 = 1 << 5;
    const strikethrough: u16 = 1 << 6;
    const overline: u16 = 1 << 7;
    const hyperlink: u16 = 1 << 8;
};

const EventKind = struct {
    const prompt: u8 = 1;
    const input: u8 = 2;
    const command: u8 = 3;
    const done: u8 = 4;
    const hold_begin: u8 = 5;
    const hold_end: u8 = 6;
    const title: u8 = 7;
    const pwd: u8 = 8;
    const bell: u8 = 9;
};

const Mode = struct {
    const cursor_keys: u32 = 1 << 0;
    const keypad_keys: u32 = 1 << 1;
    const bracketed_paste: u32 = 1 << 2;
};

const Status = struct {
    const ok: u32 = 0;
    const out_of_memory: u32 = 1;
    const cells_too_small: u32 = 2;
    const rows_too_small: u32 = 3;
    const graphemes_too_small: u32 = 4;
};

const Vt = struct {
    alloc: std.mem.Allocator,
    tiny: vt.TinyIo,
    terminal: vt.Terminal,
    stream: Stream,
    state: RenderState,
    palette: [palette_entries]u32,
    reported_palette: [palette_entries]u32,
    reverse: bool,
    frozen: bool,
    cell_width: u32,
    cell_height: u32,
    replies: std.ArrayList(u8),
    events: std.ArrayList(EventOut),
    last_mouse_cell: ?Coordinate,
    buttons_down: u32,

    fn fromHandler(handler: *Handler) *Vt {
        const stream: *Stream = @fieldParentPtr("handler", handler);
        return @alignCast(@fieldParentPtr("stream", stream));
    }

    fn push(self: *Vt, kind: u8, code: ?i32) void {
        self.events.append(self.alloc, .{
            .code = code orelse 0,
            .kind = kind,
            .has_code = @intFromBool(code != null),
        }) catch {};
    }

    fn capture(self: *Vt) bool {
        self.state.update(self.alloc, &self.terminal) catch return false;
        self.reverse = self.terminal.modes.get(.reverse_colors);
        const colours = &self.terminal.colors;
        for (0..256) |i| {
            self.palette[i] = if (colours.palette.mask.isSet(i)) tagged(colours.palette.current[i]) else 0;
        }
        self.palette[256] = dynamic(colours.foreground);
        self.palette[257] = dynamic(colours.background);
        self.palette[258] = dynamic(colours.cursor);
        return true;
    }
};

fn tagged(rgb: RGB) u32 {
    return tag_rgb | (@as(u32, rgb.r) << 16) | (@as(u32, rgb.g) << 8) | @as(u32, rgb.b);
}

fn dynamic(colour: vt.color.DynamicRGB) u32 {
    return if (colour.override) |rgb| tagged(rgb) else 0;
}

fn styleColour(colour: Style.Color) u32 {
    return switch (colour) {
        .none => 0,
        .palette => |index| tag_palette | index,
        .rgb => |rgb| tagged(rgb),
    };
}

fn cellBackground(raw: vt.Cell, style: Style) u32 {
    return switch (raw.content_tag) {
        .bg_color_palette => tag_palette | raw.content.color_palette.data,
        .bg_color_rgb => tagged(.{ .r = raw.content.color_rgb.r, .g = raw.content.color_rgb.g, .b = raw.content.color_rgb.b }),
        else => styleColour(style.bg_color),
    };
}

fn cellFlags(raw: vt.Cell, style: Style) u16 {
    const f = style.flags;
    var flags: u16 = 0;
    if (f.bold) flags |= Flag.bold;
    if (f.italic) flags |= Flag.italic;
    if (f.faint) flags |= Flag.faint;
    if (f.blink) flags |= Flag.blink;
    if (f.inverse) flags |= Flag.inverse;
    if (f.invisible) flags |= Flag.invisible;
    if (f.strikethrough) flags |= Flag.strikethrough;
    if (f.overline) flags |= Flag.overline;
    if (raw.hyperlink) flags |= Flag.hyperlink;
    return flags;
}

fn onWritePty(handler: *Handler, data: []const u8) void {
    const self = Vt.fromHandler(handler);
    self.replies.appendSlice(self.alloc, data) catch {};
}

fn onDeviceAttributes(handler: *Handler) vt.device_attributes.Attributes {
    _ = handler;
    return .{};
}

fn onEnquiry(handler: *Handler) []const u8 {
    _ = handler;
    return "";
}

fn onXtversion(handler: *Handler) []const u8 {
    _ = handler;
    return "Opsis";
}

fn onSize(handler: *Handler) ?vt.size_report.Size {
    const self = Vt.fromHandler(handler);
    return .{
        .rows = self.terminal.rows,
        .columns = self.terminal.cols,
        .cell_width = self.cell_width,
        .cell_height = self.cell_height,
    };
}

fn onSemanticPrompt(handler: *Handler, event: Handler.SemanticPrompt) void {
    const self = Vt.fromHandler(handler);
    switch (event.kind) {
        .prompt_start => self.push(EventKind.prompt, null),
        .input_start => self.push(EventKind.input, null),
        .output_start => self.push(EventKind.command, null),
        .command_end => self.push(EventKind.done, event.exit_code),
        else => {},
    }
}

fn onRenderHold(handler: *Handler, held: bool) void {
    const self = Vt.fromHandler(handler);
    if (held) {
        if (self.capture()) self.frozen = true;
        self.push(EventKind.hold_begin, null);
    } else {
        self.frozen = false;
        self.push(EventKind.hold_end, null);
    }
}

fn onTitleChanged(handler: *Handler) void {
    Vt.fromHandler(handler).push(EventKind.title, null);
}

fn onPwdChanged(handler: *Handler) void {
    Vt.fromHandler(handler).push(EventKind.pwd, null);
}

fn onBell(handler: *Handler) void {
    Vt.fromHandler(handler).push(EventKind.bell, null);
}

fn copyText(text: ?[:0]const u8, out: ?[*]u8, cap: usize) usize {
    const value = text orelse return 0;
    const buffer = out orelse return value.len;
    const n = @min(cap, value.len);
    @memcpy(buffer[0..n], value[0..n]);
    return value.len;
}

export fn opsis_vt_abi() u32 {
    return abi_version;
}

export fn opsis_vt_sizes(out: *Sizes) void {
    out.* = .{
        .cell = @sizeOf(CellOut),
        .row = @sizeOf(RowOut),
        .frame = @sizeOf(FrameOut),
        .event = @sizeOf(EventOut),
    };
}

export fn opsis_vt_new(cols: u16, rows: u16, scrollback_rows: u32) ?*Vt {
    if (cols == 0 or rows == 0) return null;
    const alloc = std.heap.c_allocator;
    const self = alloc.create(Vt) catch return null;
    self.alloc = alloc;
    self.tiny = .init;
    self.terminal = vt.Terminal.init(self.tiny.io(), alloc, .{
        .cols = cols,
        .rows = rows,
        .max_scrollback_bytes = null,
        .max_scrollback_lines = scrollback_rows,
        .default_modes = .{ .grapheme_cluster = true },
    }) catch {
        alloc.destroy(self);
        return null;
    };
    var handler: Handler = self.terminal.vtHandler();
    handler.effects.write_pty = &onWritePty;
    handler.effects.device_attributes = &onDeviceAttributes;
    handler.effects.enquiry = &onEnquiry;
    handler.effects.xtversion = &onXtversion;
    handler.effects.size = &onSize;
    handler.effects.semantic_prompt = &onSemanticPrompt;
    handler.effects.render_hold = &onRenderHold;
    handler.effects.title_changed = &onTitleChanged;
    handler.effects.pwd_changed = &onPwdChanged;
    handler.effects.bell = &onBell;
    self.stream = Stream.init(.{ .allocator = alloc, .handler = handler });
    self.state = .empty;
    self.palette = @splat(0);
    self.reported_palette = @splat(0);
    self.reverse = false;
    self.frozen = false;
    self.cell_width = 0;
    self.cell_height = 0;
    self.replies = .empty;
    self.events = .empty;
    self.last_mouse_cell = null;
    self.buttons_down = 0;
    return self;
}

export fn opsis_vt_free(self: *Vt) void {
    self.state.deinit(self.alloc);
    self.stream.deinit();
    self.terminal.deinit(self.alloc);
    self.replies.deinit(self.alloc);
    self.events.deinit(self.alloc);
    self.alloc.destroy(self);
}

export fn opsis_vt_feed(self: *Vt, ptr: [*]const u8, len: usize) void {
    self.stream.nextSlice(ptr[0..len]);
}

export fn opsis_vt_set_cell_size(self: *Vt, width: u32, height: u32) void {
    self.cell_width = width;
    self.cell_height = height;
}

export fn opsis_vt_resize(self: *Vt, cols: u16, rows: u16) u32 {
    if (cols == 0 or rows == 0) return Status.out_of_memory;
    self.stream.handler.resize(.{
        .cols = cols,
        .rows = rows,
        .cell_size_px = .{ .width = self.cell_width, .height = self.cell_height },
    }) catch return Status.out_of_memory;
    return Status.ok;
}

export fn opsis_vt_scroll(self: *Vt, delta: i32) void {
    self.terminal.scrollViewport(.{ .delta = delta });
}

export fn opsis_vt_modes(self: *Vt) u32 {
    var modes: u32 = 0;
    if (self.terminal.modes.get(.cursor_keys)) modes |= Mode.cursor_keys;
    if (self.terminal.modes.get(.keypad_keys)) modes |= Mode.keypad_keys;
    if (self.terminal.modes.get(.bracketed_paste)) modes |= Mode.bracketed_paste;
    return modes;
}

export fn opsis_vt_take_replies(self: *Vt, out: [*]u8, cap: usize) usize {
    const n = @min(cap, self.replies.items.len);
    @memcpy(out[0..n], self.replies.items[0..n]);
    self.replies.replaceRangeAssumeCapacity(0, n, &.{});
    return n;
}

export fn opsis_vt_pending_replies(self: *Vt) usize {
    return self.replies.items.len;
}

export fn opsis_vt_take_events(self: *Vt, out: [*]EventOut, cap: usize) usize {
    const n = @min(cap, self.events.items.len);
    @memcpy(out[0..n], self.events.items[0..n]);
    self.events.replaceRangeAssumeCapacity(0, n, &.{});
    return n;
}

export fn opsis_vt_pending_events(self: *Vt) usize {
    return self.events.items.len;
}

export fn opsis_vt_title(self: *Vt, out: ?[*]u8, cap: usize) usize {
    return copyText(self.terminal.getTitle(), out, cap);
}

export fn opsis_vt_pwd(self: *Vt, out: ?[*]u8, cap: usize) usize {
    return copyText(self.terminal.getPwd(), out, cap);
}

export fn opsis_vt_release_hold(self: *Vt) void {
    if (!self.frozen) return;
    self.frozen = false;
    self.terminal.modes.set(.synchronized_output, false);
}

export fn opsis_vt_palette(self: *Vt, out: [*]u32) void {
    @memcpy(out[0..palette_entries], &self.palette);
}

export fn opsis_vt_frame(
    self: *Vt,
    frame: *FrameOut,
    cells: [*]CellOut,
    cells_cap: usize,
    rows: [*]RowOut,
    rows_cap: usize,
    graphemes: [*]u32,
    graphemes_cap: usize,
) u32 {
    if (!self.frozen and !self.capture()) return Status.out_of_memory;
    const st = &self.state;
    const row_count: usize = st.rows;
    const col_count: usize = st.cols;
    const viewport = st.cursor.viewport;
    frame.* = .{
        .default_fg = self.palette[256],
        .default_bg = self.palette[257],
        .cells_written = 0,
        .graphemes_written = 0,
        .cols = st.cols,
        .rows = st.rows,
        .cursor_x = if (viewport) |v| v.x else 0,
        .cursor_y = if (viewport) |v| v.y else 0,
        .cursor_visible = @intFromBool(st.cursor.visible and viewport != null),
        .cursor_shape = @backingInt(st.cursor.visual_style),
        .cursor_blink = @intFromBool(st.cursor.blinking),
        .held = @intFromBool(self.frozen),
        .palette_changed = @intFromBool(!std.mem.eql(u32, &self.palette, &self.reported_palette)),
        .reverse = @intFromBool(self.reverse),
    };
    if (rows_cap < row_count) return Status.rows_too_small;
    if (cells_cap < row_count * col_count) return Status.cells_too_small;

    const range = st.rowDataRange();
    const row_data = st.row_data.slice();
    const raw_rows = row_data.items(.raw);
    const row_cells = row_data.items(.cells);
    const dirty = row_data.items(.dirty);
    const serials = row_data.items(.serial);
    const pins = row_data.items(.pin);
    var n: usize = 0;
    var extra: usize = 0;
    for (range.start..range.end, 0..) |i, y| {
        rows[y] = .{
            .serial = serials[i],
            .y = pins[i].y,
            .dirty = @intFromBool(dirty[i]),
            .wrapped = @intFromBool(raw_rows[i].wrap),
        };
        const slice = row_cells[i].slice();
        const raws = slice.items(.raw);
        const styles = slice.items(.style);
        const clusters = slice.items(.grapheme);
        for (raws, 0..) |raw, x| {
            const style: Style = if (raw.style_id != 0) styles[x] else .{};
            var out: CellOut = .{
                .cp = 0,
                .extra_start = 0,
                .extra_len = 0,
                .width = 1,
                .underline = @backingInt(style.flags.underline),
                .fg = styleColour(style.fg_color),
                .bg = cellBackground(raw, style),
                .underline_colour = styleColour(style.underline_color),
                .flags = cellFlags(raw, style),
            };
            switch (raw.content_tag) {
                .codepoint => out.cp = raw.content.codepoint.data,
                .codepoint_grapheme => {
                    out.cp = raw.content.codepoint.data;
                    const cluster = clusters[x];
                    out.extra_start = @intCast(extra);
                    out.extra_len = @intCast(@min(cluster.len, std.math.maxInt(u16)));
                    for (cluster[0..out.extra_len]) |codepoint| {
                        if (extra < graphemes_cap) graphemes[extra] = codepoint;
                        extra += 1;
                    }
                },
                else => {},
            }
            switch (raw.wide) {
                .narrow => {},
                .wide => out.width = 2,
                .spacer_tail, .spacer_head => out.width = 0,
            }
            cells[n] = out;
            n += 1;
        }
    }
    frame.cells_written = @intCast(n);
    frame.graphemes_written = @intCast(extra);
    if (extra > graphemes_cap) return Status.graphemes_too_small;

    for (range.start..range.end) |i| dirty[i] = false;
    st.dirty = .false;
    self.reported_palette = self.palette;
    return Status.ok;
}

const Encoded = struct {
    const unknown_key: i32 = -1;
    const too_small: i32 = -2;
};

const mods_mask: u16 = 0xF;

fn modsOf(bits: u16) input.KeyMods {
    return @bitCast(bits & mods_mask);
}

fn written(writer: *std.Io.Writer) i32 {
    return @intCast(writer.end);
}

fn singleCodepoint(name: []const u8) ?u21 {
    const view = std.unicode.Utf8View.init(name) catch return null;
    var it = view.iterator();
    const cp = it.nextCodepoint() orelse return null;
    if (it.nextCodepoint() != null) return null;
    return cp;
}

fn lowerAscii(cp: u21) u21 {
    return if (cp >= 'A' and cp <= 'Z') cp + ('a' - 'A') else cp;
}

fn upperAscii(cp: u21) u21 {
    return if (cp >= 'a' and cp <= 'z') cp - ('a' - 'A') else cp;
}

export fn opsis_vt_encode_key(
    self: *Vt,
    name_ptr: [*]const u8,
    name_len: usize,
    mods_bits: u16,
    action: u8,
    out: [*]u8,
    cap: usize,
) i32 {
    const name = name_ptr[0..name_len];
    const mods = modsOf(mods_bits);
    var text: [4]u8 = undefined;
    var event: input.KeyEvent = .{
        .action = if (action == 1) .release else .press,
        .mods = mods,
    };
    if (singleCodepoint(name)) |typed| {
        const unshifted = lowerAscii(typed);
        const shown = if (mods.shift) upperAscii(unshifted) else typed;
        const len = std.unicode.utf8Encode(shown, &text) catch return Encoded.unknown_key;
        event.key = if (unshifted < 128) input.Key.fromASCII(@intCast(unshifted)) orelse .unidentified else .unidentified;
        event.utf8 = text[0..len];
        event.unshifted_codepoint = unshifted;
        if (mods.shift and shown != unshifted) event.consumed_mods = .{ .shift = true };
    } else {
        event.key = input.Key.fromW3C(name) orelse return Encoded.unknown_key;
        if (event.key == .space) {
            text[0] = ' ';
            event.utf8 = text[0..1];
            event.unshifted_codepoint = ' ';
        }
    }
    var writer: std.Io.Writer = .fixed(out[0..cap]);
    const opts: input.KeyEncodeOptions = .fromTerminal(&self.terminal);
    input.encodeKey(&writer, event, opts) catch return Encoded.too_small;
    return written(&writer);
}

export fn opsis_vt_mouse_reporting(self: *Vt) u8 {
    return @intFromBool(self.terminal.flags.mouse_event != .none);
}

fn mouseButton(code: u8) ?input.MouseButton {
    return switch (code) {
        1 => .left,
        2 => .right,
        3 => .middle,
        4 => .four,
        5 => .five,
        else => null,
    };
}

export fn opsis_vt_encode_mouse(
    self: *Vt,
    action: u8,
    button: u8,
    mods_bits: u16,
    col: u16,
    row: u16,
    out: [*]u8,
    cap: usize,
) i32 {
    const cell_w: u32 = @max(self.cell_width, 1);
    const cell_h: u32 = @max(self.cell_height, 1);
    const size: MouseSize = .{
        .screen = .{
            .width = cell_w * @as(u32, self.terminal.cols),
            .height = cell_h * @as(u32, self.terminal.rows),
        },
        .cell = .{ .width = cell_w, .height = cell_h },
        .padding = .{},
    };
    const half: f32 = 0.5;
    const event: input.MouseEncodeEvent = .{
        .action = switch (action) {
            1 => .release,
            2 => .motion,
            else => .press,
        },
        .button = mouseButton(button),
        .mods = modsOf(mods_bits),
        .pos = .{
            .x = (@as(f32, @floatFromInt(col)) + half) * @as(f32, @floatFromInt(cell_w)),
            .y = (@as(f32, @floatFromInt(row)) + half) * @as(f32, @floatFromInt(cell_h)),
        },
    };
    var opts: input.MouseEncodeOptions = .fromTerminal(&self.terminal, size);
    opts.any_button_pressed = self.buttons_down > 0;
    opts.last_cell = &self.last_mouse_cell;
    var writer: std.Io.Writer = .fixed(out[0..cap]);
    input.encodeMouse(&writer, event, opts) catch return Encoded.too_small;
    const pressable = button >= 1 and button <= 3;
    if (pressable and event.action == .press) self.buttons_down += 1;
    if (pressable and event.action == .release and self.buttons_down > 0) self.buttons_down -= 1;
    return written(&writer);
}

export fn opsis_vt_paste_safe(self: *Vt, ptr: [*]const u8, len: usize) u8 {
    return @intFromBool(input.isSafePasteWith(ptr[0..len], .fromTerminal(&self.terminal)));
}

export fn opsis_vt_encode_paste(
    self: *Vt,
    ptr: [*]const u8,
    len: usize,
    out: [*]u8,
    cap: usize,
) i64 {
    var writer: std.Io.Writer = .fixed(out[0..cap]);
    input.encodePasteWriter(&writer, ptr[0..len], .fromTerminal(&self.terminal)) catch return Encoded.too_small;
    return @intCast(writer.end);
}

export fn opsis_vt_encode_focus(self: *Vt, gained: u8, out: [*]u8, cap: usize) i32 {
    if (!self.terminal.modes.get(.focus_event)) return 0;
    var writer: std.Io.Writer = .fixed(out[0..cap]);
    input.encodeFocus(&writer, if (gained != 0) .gained else .lost) catch return Encoded.too_small;
    return written(&writer);
}

export fn opsis_vt_pointer_shape(self: *Vt, out: [*]u8, cap: usize) usize {
    const name = @tagName(self.terminal.mouse_shape);
    const n = @min(cap, name.len);
    for (name[0..n], 0..) |ch, i| out[i] = if (ch == '_') '-' else ch;
    return name.len;
}
