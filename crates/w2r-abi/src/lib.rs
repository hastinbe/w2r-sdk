//! Rust bindings for the W2R Mod Loader plugin ABI, version 1.
//!
//! `include/w2r_abi.h` is the source of truth; this file mirrors it by hand. The layout tests at
//! the bottom check the same facts as `tests/abi_layout.c`, so a drift in either direction fails.
//!
//! ADD-ONLY: fields are only ever appended, constants are never renumbered, nothing is removed.
//! Every host function and plugin callback runs on the game's main thread. `W2R_CALL` is
//! `__cdecl`, which is what `extern "C"` means on 32-bit Windows.

#![no_std]

use core::ffi::{c_char, c_void};

pub const W2R_ABI_VERSION: u32 = 1;

// ---- status codes --------------------------------------------------------------------------
pub type W2rStatus = i32;
pub const W2R_OK: W2rStatus = 0;
/// capability not approved
pub const W2R_E_DENIED: W2rStatus = -1;
/// bad argument, or access to game memory failed
pub const W2R_E_INVALID: W2rStatus = -2;
/// a size or count limit was exceeded
pub const W2R_E_LIMIT: W2rStatus = -3;
/// not allowed right now (wrong thread, not in a lobby, ...)
pub const W2R_E_STATE: W2rStatus = -4;
/// host does not provide this (older host)
pub const W2R_E_UNSUPPORTED: W2rStatus = -5;
pub const W2R_E_NOT_FOUND: W2rStatus = -6;
/// rate limit hit; the call had no effect
pub const W2R_E_RATE: W2rStatus = -7;
/// hook overlaps an existing patch
pub const W2R_E_CONFLICT: W2rStatus = -8;
/// not ready yet, ask again on a later frame (the map index is still being built)
pub const W2R_E_BUSY: W2rStatus = -9;

// ---- value types ---------------------------------------------------------------------------
/// UTF-8, NOT NUL-terminated.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct W2rStr {
    pub ptr: *const c_char,
    pub len: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct W2rBuf {
    pub ptr: *const u8,
    pub len: u32,
}

/// host-issued, 0 is never valid
pub type W2rHandle = u32;

pub const W2R_LOG_DEBUG: u32 = 0;
pub const W2R_LOG_INFO: u32 = 1;
pub const W2R_LOG_WARN: u32 = 2;
pub const W2R_LOG_ERROR: u32 = 3;

pub const W2R_FILE_READ: u32 = 1;
/// create or truncate
pub const W2R_FILE_WRITE: u32 = 2;

/// peer_send: every peer that announced the handshake
pub const W2R_SLOT_ALL: u32 = 0xFFFF_FFFF;

// ---- limits (also in docs/abi.md) ----------------------------------------------------------
pub const W2R_MAX_LOG_BYTES: u32 = 512;
pub const W2R_MAX_SETTING_KEY: u32 = 64;
pub const W2R_MAX_SETTING_STR: u32 = 1024;
pub const W2R_MAX_FILE_NAME: u32 = 64;
pub const W2R_MAX_IO_BYTES: u32 = 65536;
pub const W2R_MAX_SYMBOL_NAME: u32 = 64;
pub const W2R_MAX_READ_BYTES: u32 = 4096;
pub const W2R_MAX_CHAT_BYTES: u32 = 200;

pub const W2R_MAX_SETTING_LABEL: u32 = 64;
pub const W2R_MAX_SETTING_DESCRIPTION: u32 = 256;
pub const W2R_MAX_SETTING_CATEGORY: u32 = 32;
pub const W2R_MAX_SETTING_CHOICE: u32 = 64;
pub const W2R_MAX_SETTING_CHOICE_COUNT: u32 = 16;
/// 0xE7 'W' '2' 'R' 'P' tag[4]
pub const W2R_PEER_HEADER_BYTES: u32 = 9;
pub const W2R_MAX_PEER_PAYLOAD: u32 = 960;

// ---- events --------------------------------------------------------------------------------
pub type W2rEventId = u32;
pub const W2R_EVT_LOBBY_ENTER: W2rEventId = 1;
pub const W2R_EVT_LOBBY_LEAVE: W2rEventId = 2;
/// payload W2rEvtPlayer; name empty without chat.read
pub const W2R_EVT_LOBBY_PLAYER_JOIN: W2rEventId = 3;
/// payload W2rEvtPlayer
pub const W2R_EVT_LOBBY_PLAYER_LEAVE: W2rEventId = 4;
/// payload W2rEvtChat; needs chat.read
pub const W2R_EVT_CHAT_RECEIVED: W2rEventId = 5;
/// payload W2rEvtPeerMessage; needs net.send
pub const W2R_EVT_PEER_MESSAGE: W2rEventId = 6;
pub const W2R_EVT_MATCH_START: W2rEventId = 7;
pub const W2R_EVT_MATCH_END: W2rEventId = 8;
pub const W2R_EVT_SETTINGS_CHANGED: W2rEventId = 9;
pub const W2R_EVT_SHUTDOWN: W2rEventId = 10;
/// payload W2rEvtGameMode; the lobby's game mode for the next match
pub const W2R_EVT_GAME_MODE: W2rEventId = 11;
/// payload W2rEvtHotkey; sent to the plugin that registered the key
pub const W2R_EVT_HOTKEY: W2rEventId = 12;
/// payload W2rEvtCommand; sent to the plugin that registered the command
pub const W2R_EVT_COMMAND: W2rEventId = 13;
/// payload W2rEvtHttp; sent to the plugin that made the request
pub const W2R_EVT_HTTP_RESPONSE: W2rEventId = 14;
/// payload W2rEvtHotkeyAction: a key the plugin offered with `hotkey_offer` went down or up (only
/// while a hotkey manager is active, `hotkeys_managed`)
pub const W2R_EVT_HOTKEY_ACTION: W2rEventId = 15;
/// payload W2rEvtKey: the key the manager asked for with `key_capture`
pub const W2R_EVT_KEY_CAPTURED: W2rEventId = 16;
/// `W2rEvtGameMode::flags`: the Classic Balance mode is on
pub const W2R_GAME_MODE_CLASSIC_BALANCE: u32 = 1 << 0;
/// `W2rEvtGameMode::flags`: the mode is for a recording being played, which is not a networked game
pub const W2R_GAME_MODE_REPLAY: u32 = 1 << 1;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct W2rEvent {
    pub struct_size: u32,
    pub id: W2rEventId,
    /// NULL for events without one; valid only during the callback
    pub payload: *const c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct W2rEvtPlayer {
    pub struct_size: u32,
    pub net_slot: u32,
    /// sanitised: printable ASCII only
    pub name: W2rStr,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct W2rEvtChat {
    pub struct_size: u32,
    pub net_slot: u32,
    pub name: W2rStr,
    /// sanitised: printable ASCII only
    pub text: W2rStr,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct W2rEvtPeerMessage {
    pub struct_size: u32,
    /// the slot the transport reported, never one named inside the message
    pub net_slot: u32,
    /// the payload after the 9-byte header
    pub data: W2rBuf,
}

/// `W2rMatchPlayer::flags`
pub const W2R_PLAYER_HUMAN: u32 = 1 << 0;
pub const W2R_PLAYER_COMPUTER: u32 = 1 << 1;
pub const W2R_PLAYER_LOCAL: u32 = 1 << 2;
/// `W2rMatchPlayer::result`; any other code is passed through unchanged
pub const W2R_RESULT_NONE: u32 = 0;
pub const W2R_RESULT_DEFEAT: u32 = 2;
pub const W2R_RESULT_VICTORY: u32 = 3;
pub const W2R_RESULT_DRAW: u32 = 5;
/// `W2rEvtMatchEnd::flags`: the game flagged at least one turn as out of sync
pub const W2R_MATCH_DESYNC: u32 = 1 << 0;

/// One player's end-of-match tallies (`W2rEvtMatchEnd::players`).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct W2rMatchPlayer {
    pub player: u32,
    pub flags: u32,
    pub result: u32,
    pub units_killed: u32,
    pub buildings_razed: u32,
    pub units_lost: u32,
    pub buildings_lost: u32,
    pub gold_mined: u32,
    pub lumber_mined: u32,
    pub oil_mined: u32,
    /// sanitised: printable ASCII only; empty without `chat.read`
    pub name: W2rStr,
}

/// Payload of `W2R_EVT_MATCH_END`; NULL from a host that predates it.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct W2rEvtMatchEnd {
    pub struct_size: u32,
    pub local_player: u32,
    pub flags: u32,
    pub turns: u32,
    pub map: W2rStr,
    pub player_count: u32,
    /// `size_of::<W2rMatchPlayer>()` in the host: step by this
    pub player_size: u32,
    pub players: *const W2rMatchPlayer,
}

/// Payload of `W2R_EVT_HOTKEY`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct W2rEvtHotkey {
    pub struct_size: u32,
    /// the id given to `hotkey_register`
    pub id: u32,
}

/// Payload of `W2R_EVT_COMMAND`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct W2rEvtCommand {
    pub struct_size: u32,
    /// the id given to `command_register`
    pub id: u32,
    /// the text after `/name `, trimmed; printable ASCII, at most 200 bytes
    pub args: W2rStr,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct W2rEvtGameMode {
    pub struct_size: u32,
    /// `W2R_GAME_MODE_*`: the modes in force for the next match, on every machine
    pub flags: u32,
}

// ---- hotkeys (docs/abi.md, "Hotkeys") ------------------------------------------------------
/// A key: the SDL scancode (layout independent: `W2R_KEY_*` names a few) in the low 16 bits, and
/// the modifiers that must be held, exactly. 0 is no key.
pub type W2rKey = u32;
pub const W2R_KEY_SHIFT: W2rKey = 1 << 16;
pub const W2R_KEY_CTRL: W2rKey = 1 << 17;
pub const W2R_KEY_ALT: W2rKey = 1 << 18;
/// The scancode part of a `W2rKey`.
pub const W2R_KEY_CODE_MASK: W2rKey = 0xFFFF;
pub const W2R_KEY_A: W2rKey = 4;
pub const W2R_KEY_P: W2rKey = 19;
pub const W2R_KEY_GRAVE: W2rKey = 53;
pub const W2R_KEY_F1: W2rKey = 58;

/// `W2rHotkeyDef.flags`: the game sees the key too (by default a hotkey the service delivers is
/// kept from the game).
pub const W2R_HOTKEY_PASS: u32 = 1;
/// `W2rHotkeyDef.flags`: not while a command waits for its target, nor just after a command key
/// (a multicast holds Alt between spells).
pub const W2R_HOTKEY_SAFE_CAST: u32 = 2;
/// Hotkey flag: the plugin reads the key itself (its own hook or polling);
/// the service only keeps the binding, never fires `W2R_EVT_HOTKEY_ACTION` or keeps the key from the game.
/// Read it with `hotkey_get`.
pub const W2R_HOTKEY_POLL: u32 = 4;

/// One action a plugin offers for the hotkey manager's screen, as `hotkey_offer` takes it.
/// Every string is copied during the call.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct W2rHotkeyDef {
    /// `W2R_HOTKEY_DEF_V1_SIZE`.
    pub struct_size: u32,
    /// `W2R_HOTKEY_PASS`, `W2R_HOTKEY_SAFE_CAST` or 0.
    pub flags: u32,
    /// The key the action has until a manager binds another; 0 for none.
    pub default_key: W2rKey,
    /// A second key, or 0.
    pub default_alt: W2rKey,
    /// Unique within the plugin, the settings-key rule (`[a-z0-9_.]`, 1-64 bytes).
    pub id: W2rStr,
    /// What the manager's screen shows. 1 to 64 bytes, Title Case.
    pub label: W2rStr,
    /// The manager's heading for it; empty uses the plugin's name.
    pub category: W2rStr,
}
pub const W2R_HOTKEY_DEF_V1_SIZE: u32 = core::mem::size_of::<W2rHotkeyDef>() as u32;

/// `W2R_EVT_HOTKEY_ACTION`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct W2rEvtHotkeyAction {
    pub struct_size: u32,
    /// The handle `hotkey_offer` gave.
    pub action: W2rHandle,
    /// 1 when the key went down, 0 when it was let go. Key repeat is not delivered.
    pub down: u32,
}

/// `W2R_EVT_KEY_CAPTURED`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct W2rEvtKey {
    pub struct_size: u32,
    /// The key and the modifiers held with it; Esc alone is `41`.
    pub key: W2rKey,
}

/// What `hotkey_info` reports about one registered action. The strings point into the host's
/// copy and stay valid until the next `hotkey_info` call.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct W2rHotkeyInfo {
    pub struct_size: u32,
    pub flags: u32,
    /// The bound keys now, and the plugin's defaults.
    pub key: W2rKey,
    pub alt: W2rKey,
    pub default_key: W2rKey,
    pub default_alt: W2rKey,
    /// The owning plugin's id and name.
    pub plugin: W2rStr,
    pub plugin_name: W2rStr,
    pub id: W2rStr,
    pub label: W2rStr,
    pub category: W2rStr,
}
pub const W2R_HOTKEY_INFO_V1_SIZE: u32 = core::mem::size_of::<W2rHotkeyInfo>() as u32;

/// `W2rRemap.flags`: while a command waits for its target, or just after a command key, the key
/// is kept from the game instead (the Alt menu keys during a multicast).
pub const W2R_REMAP_SAFE_CAST: u32 = 1;

/// One entry of `key_remap_set`: pressing `from` gives the game `to`. `to` 0 keeps `from` from the
/// game; `to == from` with `W2R_REMAP_SAFE_CAST` only adds the safety.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct W2rRemap {
    pub from: W2rKey,
    pub to: W2rKey,
    pub flags: u32,
    /// Only while this command card (the game's button set index) is up; `W2R_ANY_CARD` always.
    pub card: u32,
}

/// `W2rRemap.card`: whatever card is up.
pub const W2R_ANY_CARD: u32 = 0xFFFF_FFFF;

// ---- plugin screen (only valid inside the screen callback) ---------------------------------
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct W2rRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// `W2rScreen.frame` styles.
pub const W2R_FRAME_WINDOW: u32 = 0;
pub const W2R_FRAME_PANEL: u32 = 1;
/// The game's title scroll (the art of the menus' title banner), stretched to the rect.
pub const W2R_FRAME_BANNER: u32 = 2;
/// The game's checkbox, off or ticked (with `W2R_FRAME_ORC` for the Orc art); lit under the mouse.
pub const W2R_FRAME_CHECK: u32 = 3;
pub const W2R_FRAME_CHECK_ON: u32 = 4;
/// The game's scroll bar track and its thumb (with `W2R_FRAME_ORC` for the Orc art); lit under the mouse,
/// and the thumb pressed with `W2R_FRAME_ACTIVE` (while it is dragged).
pub const W2R_FRAME_SCROLL: u32 = 5;
pub const W2R_FRAME_SCROLL_THUMB: u32 = 6;
/// A title band in the mod's own style: a dark strip with a gold rule along its bottom (the text
/// is the plugin's). Nothing in Classic graphics, whose titles stand on their own.
pub const W2R_FRAME_TITLE: u32 = 7;
pub const W2R_FRAME_ACTIVE: u32 = 0x200;
/// `W2rScreen.icons` bit: the game is in Classic graphics (the page draws the Classic look).
pub const W2R_SCREEN_CLASSIC: u32 = 2;
/// `W2rScreen.frame` / `button`: draw in the Orc skin instead of the Human one.
pub const W2R_FRAME_ORC: u32 = 0x100;
/// `W2rScreen.button` flags.
pub const W2R_BUTTON_GREYED: u32 = 1;
/// `W2rScreen.button` flag: the Orc Game Menu button (Human without it).
pub const W2R_BUTTON_ORC: u32 = 2;
/// `W2rScreen.icon` flags.
pub const W2R_ICON_SELECTED: u32 = 1;
/// `W2rScreen.icon` flag: the icon belongs to the Orc side (off a match the menus paint team parts in
/// the game's first team colour, red, for both sides).
pub const W2R_ICON_ORC: u32 = 2;
/// `W2rScreen.label` alignment.
pub const W2R_ALIGN_LEFT: u32 = 0x11;
pub const W2R_ALIGN_CENTRE: u32 = 0x12;
pub const W2R_ALIGN_RIGHT: u32 = 0x14;
/// `W2rScreen.label` flag: the text in the menus' own font (the Game Menu buttons'), for headings.
pub const W2R_TEXT_TITLE: u32 = 0x100;
/// `W2rScreen.mouse` bits.
pub const W2R_MOUSE_LEFT_DOWN: u32 = 1;
pub const W2R_MOUSE_LEFT_CLICK: u32 = 2;
/// The wheel turned this frame, away from you (up) or towards you (down).
pub const W2R_MOUSE_WHEEL_UP: u32 = 4;
pub const W2R_MOUSE_WHEEL_DOWN: u32 = 8;

/// Drawing calls for a manager's full-screen window (`screen_set`), in the window's own space:
/// (0, 0) is its top left, `width` by `height`. Inside `list_begin` / `list_end` rects are in the
/// list's content space, scrolled. Colours are `r | g << 8 | b << 16 | a << 24`; 0 is the
/// skin's text colour. Strings are at most 128 bytes.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct W2rScreen {
    pub struct_size: u32,
    pub ctx: *mut c_void,
    pub width: f32,
    pub height: f32,
    /// The mouse in the window's space, and `W2R_MOUSE_*` bits.
    pub mouse_x: f32,
    pub mouse_y: f32,
    pub mouse: u32,
    pub fill: unsafe extern "C" fn(ctx: *mut c_void, r: *const W2rRect, colour: u32, round: f32),
    /// A skinned frame (`W2R_FRAME_*`).
    pub frame: unsafe extern "C" fn(ctx: *mut c_void, r: *const W2rRect, style: u32),
    pub label: unsafe extern "C" fn(
        ctx: *mut c_void,
        r: *const W2rRect,
        text: W2rStr,
        align: u32,
        colour: u32,
    ),
    pub wrapped: unsafe extern "C" fn(ctx: *mut c_void, r: *const W2rRect, text: W2rStr, colour: u32),
    /// The game's menu button; 1 when clicked.
    pub button:
        unsafe extern "C" fn(ctx: *mut c_void, r: *const W2rRect, text: W2rStr, flags: u32) -> i32,
    /// Command button art `icon` (the button's icon number) in its frame; 1 when clicked.
    pub icon: unsafe extern "C" fn(ctx: *mut c_void, r: *const W2rRect, icon: u32, flags: u32) -> i32,
    /// 1 when the left button was clicked inside `r` this frame (in the current space).
    pub hit: unsafe extern "C" fn(ctx: *mut c_void, r: *const W2rRect) -> i32,
    /// 1 when the mouse is over `r`.
    pub hover: unsafe extern "C" fn(ctx: *mut c_void, r: *const W2rRect) -> i32,
    /// A scrolling region `content_height` tall; 1 when it opened (then draw and `list_end`).
    pub list_begin: unsafe extern "C" fn(
        ctx: *mut c_void,
        r: *const W2rRect,
        name: W2rStr,
        content_height: f32,
    ) -> i32,
    pub list_end: unsafe extern "C" fn(ctx: *mut c_void),
    /// 1 when command icon art is loaded (in a match); 0 in the menus, where `icon` draws an empty
    /// frame and the plugin may write a name instead.
    pub icons: u32,
    /// (V2) The width `text` takes on the screen, drawn as `label` with `flags` (`W2R_TEXT_TITLE` for
    /// the heading font): plugins centre and fit text with it. Present when `struct_size` >=
    /// `W2R_SCREEN_V2_SIZE`.
    pub measure: unsafe extern "C" fn(ctx: *mut c_void, text: W2rStr, flags: u32) -> f32,
}
/// The screen without `measure`.
pub const W2R_SCREEN_V1_SIZE: u32 = core::mem::offset_of!(W2rScreen, measure) as u32;
pub const W2R_SCREEN_V2_SIZE: u32 = core::mem::size_of::<W2rScreen>() as u32;

// ---- UI (only valid inside on_ui) ----------------------------------------------------------
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct W2rUi {
    pub struct_size: u32,
    pub ctx: *mut c_void,
    pub label: unsafe extern "C" fn(ctx: *mut c_void, text: W2rStr),
    /// 1 when pressed
    pub button: unsafe extern "C" fn(ctx: *mut c_void, text: W2rStr) -> i32,
    /// 1 when changed
    pub checkbox: unsafe extern "C" fn(ctx: *mut c_void, text: W2rStr, value: *mut i32) -> i32,
    pub slider_int: unsafe extern "C" fn(
        ctx: *mut c_void,
        text: W2rStr,
        value: *mut i32,
        min: i32,
        max: i32,
    ) -> i32,
    /// Appended after the first release: check `struct_size` covers this before using it.
    /// An edit box over the plugin's own NUL-terminated buffer of `cap` bytes; 1 when the text
    /// changed this frame.
    pub text_edit:
        unsafe extern "C" fn(ctx: *mut c_void, text: W2rStr, buf: *mut u8, cap: u32) -> i32,
    /// Appended with `text_edit`. 0 greys out and disables the rows that follow, 1 restores them.
    pub set_active: unsafe extern "C" fn(ctx: *mut c_void, active: i32),
    /// Appended after `set_active`: check `struct_size >= W2R_UI_COLOR_EDIT_SIZE` first. A color
    /// row: the caption, a swatch, and a `#RRGGBB` button that opens the host's color editor (the
    /// options screen's). `hex` is the plugin's own 7-byte buffer, `rrggbb` and a NUL; 1 on the
    /// frame the player pressed OK on a different color, with `hex` rewritten.
    pub color_edit: unsafe extern "C" fn(ctx: *mut c_void, text: W2rStr, hex: *mut u8) -> i32,
    /// Appended after `color_edit`: check `struct_size >= W2R_UI_COMBO_SIZE` first. A dropdown
    /// row like the options screen's: the caption and a dropdown of `count` choices (2 to
    /// `W2R_MAX_SETTING_CHOICE_COUNT`, each at most `W2R_MAX_SETTING_CHOICE` bytes), showing
    /// `*selected`. 1 on the frame the player picked a different entry, with `*selected` set to it.
    pub combo: unsafe extern "C" fn(
        ctx: *mut c_void,
        text: W2rStr,
        choices: *const W2rStr,
        count: u32,
        selected: *mut i32,
    ) -> i32,
}

/// `W2rUi.struct_size` with `text_edit` and `set_active`: a plugin that wants them checks
/// `ui.struct_size >= W2R_UI_TEXT_EDIT_SIZE`.
pub const W2R_UI_TEXT_EDIT_SIZE: u32 =
    (core::mem::offset_of!(W2rUi, set_active) + core::mem::size_of::<usize>()) as u32;

/// `W2rUi.struct_size` with `color_edit`: a plugin that wants it checks
/// `ui.struct_size >= W2R_UI_COLOR_EDIT_SIZE`.
pub const W2R_UI_COLOR_EDIT_SIZE: u32 =
    (core::mem::offset_of!(W2rUi, color_edit) + core::mem::size_of::<usize>()) as u32;

/// `W2rUi.struct_size` with `combo`: a plugin that wants it checks
/// `ui.struct_size >= W2R_UI_COMBO_SIZE`.
pub const W2R_UI_COMBO_SIZE: u32 = core::mem::size_of::<W2rUi>() as u32;

/// Bytes in a `color_edit` buffer: six hex digits and a NUL.
pub const W2R_COLOR_HEX_BYTES: u32 = 7;

// ---- host table: provided by the host, one instance per plugin -----------------------------
/// The host fills every version-1 field, so these pointers are never null for a version-1 host.
/// A plugin built against a later version must check `struct_size` before touching fields past
/// the version it was built for.
// ---- game snapshot --------------------------------------------------------------------------
/// `W2rGameSnapshot::flags`
pub const W2R_SNAPSHOT_IN_MATCH: u32 = 1 << 0;
pub const W2R_SNAPSHOT_PAUSED: u32 = 1 << 1;
/// a recording is being played back
pub const W2R_SNAPSHOT_REPLAY: u32 = 1 << 2;
/// the local player is watching, not playing
pub const W2R_SNAPSHOT_OBSERVER: u32 = 1 << 3;
/// `W2rPlayerInfo::flags`: on the local player's team
pub const W2R_PLAYER_ALLY: u32 = 1 << 3;
/// "no value" in the unsigned fields of the snapshot
pub const W2R_NONE: u32 = 0xFFFF_FFFF;
pub const W2R_MAX_PLAYERS: usize = 8;

/// One player in a [`W2rGameSnapshot`].
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct W2rPlayerInfo {
    pub player: u32,
    pub flags: u32,
    pub race: u32,
    pub team: u32,
    pub colour: u32,
    pub result: u32,
    pub gold: i32,
    pub lumber: i32,
    pub oil: i32,
    pub food_used: i32,
    pub food_cap: i32,
    /// valid until the next `game_snapshot` call
    pub name: W2rStr,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct W2rGameSnapshot {
    /// in: `size_of::<W2rGameSnapshot>()`
    pub struct_size: u32,
    pub flags: u32,
    pub game_ms: u32,
    pub turn: u32,
    pub local_player: u32,
    pub player_count: u32,
    pub players: [W2rPlayerInfo; W2R_MAX_PLAYERS],
}

impl W2rGameSnapshot {
    pub const fn zeroed() -> Self {
        const NO_STR: W2rStr = W2rStr { ptr: core::ptr::null(), len: 0 };
        const P: W2rPlayerInfo = W2rPlayerInfo {
            player: 0, flags: 0, race: 0, team: 0, colour: 0, result: 0,
            gold: 0, lumber: 0, oil: 0, food_used: 0, food_cap: 0, name: NO_STR,
        };
        W2rGameSnapshot {
            struct_size: core::mem::size_of::<W2rGameSnapshot>() as u32,
            flags: 0, game_ms: 0, turn: 0, local_player: 0, player_count: 0,
            players: [P; W2R_MAX_PLAYERS],
        }
    }
}

// ---- HUD text -------------------------------------------------------------------------------
/// `hud_text_set` places: above the APM counter and the clock (the original place)
pub const W2R_HUD_BOTTOM_RIGHT: u32 = 0;
/// below the resource bar
pub const W2R_HUD_TOP_LEFT: u32 = 1;
pub const W2R_HUD_TOP_RIGHT: u32 = 2;
pub const W2R_HUD_TOP_CENTER: u32 = 3;
/// lines per plugin, in all places together
pub const W2R_MAX_HUD_LINES: u32 = 4;
pub const W2R_MAX_HUD_KEY: u32 = 32;
pub const W2R_MAX_HUD_TEXT: u32 = 64;

// ---- units and orders -----------------------------------------------------------------------
/// `W2rUnit::flags`
pub const W2R_UNIT_BUILDING: u32 = 1 << 0;
pub const W2R_UNIT_WORKER: u32 = 1 << 1;
/// in the local player's current selection
pub const W2R_UNIT_SELECTED: u32 = 1 << 2;
/// a building that is completed
pub const W2R_UNIT_FINISHED: u32 = 1 << 3;
/// its order is stop or stand
pub const W2R_UNIT_IDLE: u32 = 1 << 4;

/// One unit as `units_query` reports it.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct W2rUnit {
    pub id: u32,
    pub serial: u32,
    pub type_id: u32,
    pub owner: u32,
    pub order: u32,
    pub flags: u32,
    pub tile_x: i32,
    pub tile_y: i32,
    pub hp: u32,
    pub mana: u32,
}

/// What `units_query` looks for.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct W2rUnitQuery {
    pub struct_size: u32,
    /// bit n selects player n, bit 15 neutral; 0 means any owner
    pub owners: u32,
    /// a unit type id, or `W2R_NONE` for any
    pub type_id: u32,
    pub flags_all: u32,
    pub flags_none: u32,
}

pub const W2R_ORDER_STOP: u32 = 2;
pub const W2R_ORDER_MOVE: u32 = 3;
pub const W2R_ORDER_PATROL: u32 = 5;
pub const W2R_ORDER_FOLLOW: u32 = 6;
/// with a target unit: attack it; otherwise attack-move to the tile
pub const W2R_ORDER_ATTACK: u32 = 8;
/// stand ground
pub const W2R_ORDER_STAND: u32 = 13;
pub const W2R_ORDER_HARVEST: u32 = 23;
pub const W2R_ORDER_RETURN: u32 = 24;
pub const W2R_ORDER_REPAIR: u32 = 27;
/// the game's selection limit
pub const W2R_MAX_ORDER_UNITS: usize = 12;

/// A unit named by slot and creation serial.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct W2rUnitRef {
    pub id: u32,
    pub serial: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct W2rOrderRequest {
    pub struct_size: u32,
    pub order: u32,
    pub tile_x: i32,
    pub tile_y: i32,
    /// `id == W2R_NONE`: no target unit
    pub target: W2rUnitRef,
    pub unit_count: u32,
    pub units: [W2rUnitRef; W2R_MAX_ORDER_UNITS],
}

// ---- HTTP -----------------------------------------------------------------------------------
pub const W2R_HTTP_GET: u32 = 0;
pub const W2R_HTTP_POST: u32 = 1;
/// `W2rEvtHttp::error`: an answer came back; see `status`
pub const W2R_HTTP_OK: u32 = 0;
/// no answer: could not connect, timed out, redirected or aborted
pub const W2R_HTTP_FAILED: u32 = 1;
/// the answer was longer than `W2R_MAX_HTTP_RESPONSE`
pub const W2R_HTTP_TOO_LARGE: u32 = 2;
pub const W2R_MAX_HTTP_URL: u32 = 512;
pub const W2R_MAX_HTTP_BODY: u32 = 16384;
pub const W2R_MAX_HTTP_RESPONSE: u32 = 262_144;
pub const W2R_MAX_HTTP_IN_FLIGHT: u32 = 4;

/// What `http_request` sends.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct W2rHttpRequest {
    pub struct_size: u32,
    /// chosen by the plugin; comes back in the answer
    pub id: u32,
    /// `W2R_HTTP_GET` or `W2R_HTTP_POST`
    pub method: u32,
    /// `https://host/path?query`, host from the manifest's `net_hosts`
    pub url: W2rStr,
    /// POST only; empty means `application/json`
    pub content_type: W2rStr,
    /// POST only
    pub body: W2rBuf,
}

/// Payload of `W2R_EVT_HTTP_RESPONSE`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct W2rEvtHttp {
    pub struct_size: u32,
    pub id: u32,
    /// the HTTP status, or 0 when `error` is not `W2R_HTTP_OK`
    pub status: u32,
    pub error: u32,
    /// valid only during the callback
    pub body: W2rBuf,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct W2rHost {
    pub struct_size: u32,
    pub abi_version: u32,
    /// opaque; pass back as the first argument of every call
    pub token: *mut c_void,

    // log (implicit)
    pub log: unsafe extern "C" fn(token: *mut c_void, level: u32, msg: W2rStr) -> W2rStatus,

    // settings (implicit): the plugin's own settings only
    pub settings_get_i64:
        unsafe extern "C" fn(token: *mut c_void, key: W2rStr, out: *mut i64) -> W2rStatus,
    pub settings_set_i64:
        unsafe extern "C" fn(token: *mut c_void, key: W2rStr, value: i64) -> W2rStatus,
    pub settings_get_str: unsafe extern "C" fn(
        token: *mut c_void,
        key: W2rStr,
        buf: *mut c_char,
        cap: u32,
        len_out: *mut u32,
    ) -> W2rStatus,
    pub settings_set_str:
        unsafe extern "C" fn(token: *mut c_void, key: W2rStr, value: W2rStr) -> W2rStatus,

    // fs.data (implicit): flat files in the plugin's own data folder, by host handle
    pub data_open: unsafe extern "C" fn(
        token: *mut c_void,
        name: W2rStr,
        flags: u32,
        out: *mut W2rHandle,
    ) -> W2rStatus,
    pub data_read: unsafe extern "C" fn(
        token: *mut c_void,
        file: W2rHandle,
        dst: *mut c_void,
        cap: u32,
        read_out: *mut u32,
    ) -> W2rStatus,
    pub data_write: unsafe extern "C" fn(
        token: *mut c_void,
        file: W2rHandle,
        src: *const c_void,
        len: u32,
    ) -> W2rStatus,
    pub data_close: unsafe extern "C" fn(token: *mut c_void, file: W2rHandle) -> W2rStatus,

    // events
    pub subscribe: unsafe extern "C" fn(token: *mut c_void, id: W2rEventId) -> W2rStatus,
    pub unsubscribe: unsafe extern "C" fn(token: *mut c_void, id: W2rEventId) -> W2rStatus,

    // game.read
    pub symbol_lookup:
        unsafe extern "C" fn(token: *mut c_void, name: W2rStr, out: *mut W2rHandle) -> W2rStatus,
    pub symbol_read: unsafe extern "C" fn(
        token: *mut c_void,
        symbol: W2rHandle,
        offset: u32,
        dst: *mut c_void,
        len: u32,
    ) -> W2rStatus,

    // chat.write
    pub chat_send: unsafe extern "C" fn(token: *mut c_void, text: W2rStr) -> W2rStatus,

    // net.send
    pub peer_send: unsafe extern "C" fn(
        token: *mut c_void,
        net_slot_or_all: u32,
        payload: W2rBuf,
    ) -> W2rStatus,

    // hooks.raw (dangerous)
    pub hook_install: unsafe extern "C" fn(
        token: *mut c_void,
        target: u32,
        detour: *mut c_void,
        original_out: *mut *mut c_void,
        out: *mut W2rHandle,
    ) -> W2rStatus,
    pub hook_remove: unsafe extern "C" fn(token: *mut c_void, hook: W2rHandle) -> W2rStatus,

    // hooks.raw (dangerous): the runtime address of any symbol, for a plugin that calls game
    // functions or writes game memory itself (docs/chat-plugin.md)
    pub symbol_address: unsafe extern "C" fn(
        token: *mut c_void,
        symbol: W2rHandle,
        address_out: *mut usize,
    ) -> W2rStatus,

    // Create-screen extension (implicit). Appended after the first release: check
    // `struct_size >= W2R_HOST_CREATE_UI_SIZE` before using these.
    /// Registers the plugin's one callback for the Create Multiplayer Game screen. The host calls
    /// it every frame the enhanced screen draws, with a `W2rUi` whose rows appear under the map
    /// preview. Registering again replaces the callback; null removes it.
    pub create_ui_register: unsafe extern "C" fn(
        token: *mut c_void,
        callback: Option<unsafe extern "C" fn(ui: *const W2rUi)>,
    ) -> W2rStatus,
    /// Only inside the create-screen callback. Counts the maps of the host's map index with
    /// `players` maximum players (0 = any). `W2R_E_BUSY` while the index is being built.
    pub create_map_count: unsafe extern "C" fn(
        token: *mut c_void,
        players: u32,
        count_out: *mut u32,
    ) -> W2rStatus,
    /// Only inside the create-screen callback. Makes the `nth` (0-based, in index order) map with
    /// `players` maximum players (0 = any) the screen's selected map.
    pub create_map_select:
        unsafe extern "C" fn(token: *mut c_void, players: u32, nth: u32) -> W2rStatus,

    // Appended after the create-screen functions: check `struct_size >= W2R_HOST_MAP_FOLDER_SIZE`.
    /// Only inside the create-screen callback. Limits `create_map_count` and `create_map_select`
    /// of this callback to the maps in `folder` (a path under the maps folder, `\` or `/`
    /// separated, subfolders included). An empty string removes the limit. The limit lasts until
    /// the callback returns, so call this first, every frame. A folder with no maps gives a count
    /// of 0.
    pub create_map_folder: unsafe extern "C" fn(token: *mut c_void, folder: W2rStr) -> W2rStatus,

    // Appended after `create_map_folder`: check `struct_size >= W2R_HOST_SETTINGS_DECLARE_SIZE`.
    /// Only inside `on_load`. Declares one of the plugin's own settings, so the host's options
    /// screen shows it under `category` / `subcategory` with the right widget and hover text, and
    /// edits it for the plugin. The value is stored as before (`settings_get_*` read it, and a
    /// declared key that was never stored reads as its default); the plugin hears of a change
    /// through `W2R_EVT_SETTINGS_CHANGED`. docs/abi.md, "Declared settings".
    pub settings_declare:
        unsafe extern "C" fn(token: *mut c_void, def: *const W2rSettingDef) -> W2rStatus,

    // Appended after `settings_declare`: check `struct_size >= W2R_HOST_OVERLAY_SIZE`.
    /// Replaces the plugin's overlay: `count` items (at most `W2R_MAX_OVERLAY_ITEMS`) the host
    /// draws on the map view during a match, in world pixels (tile * 32), until the next call.
    /// `count` 0 clears it. docs/abi.md, "Overlay".
    pub overlay_set: unsafe extern "C" fn(
        token: *mut c_void,
        items: *const W2rOverlayItem,
        count: u32,
    ) -> W2rStatus,

    // Appended after `overlay_set`: check `struct_size >= W2R_HOST_SET_CHOICES_SIZE`.
    /// Replaces the entries of one of the plugin's declared `W2R_SETTING_CHOICE` settings: `count`
    /// labels (2 to 16, 1 to 64 bytes each), copied during the call. The options screen shows them
    /// at once. The stored value is an index; a plugin that shortens the list sets it again.
    /// `W2R_E_NOT_FOUND` for a key that is not a declared choice. docs/abi.md, "Declared settings".
    pub settings_set_choices: unsafe extern "C" fn(
        token: *mut c_void,
        key: W2rStr,
        choices: *const W2rStr,
        count: u32,
    ) -> W2rStatus,

    // Appended after `settings_set_choices`: check `struct_size >= W2R_HOST_SNAPSHOT_SIZE`.
    /// Fills the snapshot with the match as the local player may know it (clock, pause, players with
    /// race, team, colour, result and resources). Needs `game.read`. Other players' resources are -1
    /// unless they are on the local player's team, the local player observes, or a recording plays.
    /// `W2R_E_STATE` outside a match. docs/abi.md, "Game snapshot".
    pub game_snapshot: unsafe extern "C" fn(token: *mut c_void, out: *mut W2rGameSnapshot) -> W2rStatus,

    // Appended after `game_snapshot`: check `struct_size >= W2R_HOST_HOTKEY_SIZE`.
    /// Asks for a key combination such as `"Ctrl+Alt+K"`; the plugin gets `W2R_EVT_HOTKEY` with `id`
    /// when it is pressed. `W2R_E_INVALID`, `W2R_E_CONFLICT` or `W2R_E_LIMIT` otherwise.
    /// docs/abi.md, "Hotkeys and commands".
    pub hotkey_register: unsafe extern "C" fn(token: *mut c_void, combo: W2rStr, id: u32) -> W2rStatus,

    // Appended after `hotkey_register`: check `struct_size >= W2R_HOST_COMMAND_SIZE`.
    /// Asks for the chat command `/name`; the plugin gets `W2R_EVT_COMMAND` with `id` and the
    /// arguments when the player types it, and the line is not sent. docs/abi.md, "Hotkeys and
    /// commands".
    pub command_register: unsafe extern "C" fn(token: *mut c_void, name: W2rStr, id: u32) -> W2rStatus,

    // Appended after `command_register`: check `struct_size >= W2R_HOST_HUD_TEXT_SIZE`.
    /// Sets (or, with empty `text`, removes) one line of the plugin's HUD text at `place`
    /// (`W2R_HUD_*`). docs/abi.md, "HUD text".
    pub hud_text_set: unsafe extern "C" fn(token: *mut c_void, key: W2rStr, text: W2rStr, place: u32) -> W2rStatus,

    // Appended after `hud_text_set`: check `struct_size >= W2R_HOST_UNITS_SIZE`.
    /// Lists the units the local player may know about that match `query` into `out[..cap]` and
    /// stores the match count in `*total`. Needs `game.read`. docs/abi.md, "Units and orders".
    pub units_query: unsafe extern "C" fn(
        token: *mut c_void,
        query: *const W2rUnitQuery,
        out: *mut W2rUnit,
        cap: u32,
        total: *mut u32,
    ) -> W2rStatus,

    // Appended after `units_query`: check `struct_size >= W2R_HOST_ORDER_SIZE`.
    /// Gives an order to up to 12 of the local player's own units as if clicked (through the
    /// game's command queue). Needs `game.control`. docs/abi.md, "Units and orders".
    pub order_issue: unsafe extern "C" fn(token: *mut c_void, request: *const W2rOrderRequest) -> W2rStatus,

    // Appended after `order_issue`: check `struct_size >= W2R_HOST_HTTP_SIZE`.
    /// Sends an HTTPS request to a server named in the manifest's `net_hosts`; the answer arrives
    /// as `W2R_EVT_HTTP_RESPONSE` with the same id. Needs `net.http`. docs/abi.md, "HTTP".
    pub http_request: unsafe extern "C" fn(token: *mut c_void, request: *const W2rHttpRequest) -> W2rStatus,

    // Hotkeys. Appended after `http_request`: check `struct_size >= W2R_HOST_HOTKEY_SERVICE_SIZE`.
    // docs/abi.md, "Hotkeys".
    /// Implicit, only inside `on_load`. Offers one action to the hotkey manager. While a manager
    /// is active the host reads its key and sends `W2R_EVT_HOTKEY_ACTION`; otherwise nothing is sent and
    /// the plugin reads its keys itself, as before.
    pub hotkey_offer: unsafe extern "C" fn(
        token: *mut c_void,
        def: *const W2rHotkeyDef,
        out: *mut W2rHandle,
    ) -> W2rStatus,
    /// Implicit. 1 while a hotkey manager plugin is active (then a plugin's own key settings
    /// should be hidden), else 0.
    pub hotkeys_managed: unsafe extern "C" fn(token: *mut c_void) -> i32,
    /// `hooks.raw`, only inside `on_load`. Makes this plugin the hotkey manager (one per session;
    /// `W2R_E_CONFLICT` for a second). The calls below need it (`W2R_E_STATE` otherwise).
    pub hotkey_manage: unsafe extern "C" fn(token: *mut c_void) -> W2rStatus,
    pub hotkey_count: unsafe extern "C" fn(token: *mut c_void, count_out: *mut u32) -> W2rStatus,
    pub hotkey_info:
        unsafe extern "C" fn(token: *mut c_void, index: u32, out: *mut W2rHotkeyInfo) -> W2rStatus,
    /// Binds action `index` to `key` and `alt` (0 for none) until the session ends.
    pub hotkey_bind:
        unsafe extern "C" fn(token: *mut c_void, index: u32, key: W2rKey, alt: W2rKey) -> W2rStatus,
    /// Replaces the whole key remap table (at most `W2R_MAX_REMAPS` entries; 0 clears it).
    pub key_remap_set:
        unsafe extern "C" fn(token: *mut c_void, entries: *const W2rRemap, count: u32) -> W2rStatus,
    /// 1: the next key pressed in a match is kept from the game and sent as
    /// `W2R_EVT_KEY_CAPTURED`, then capture ends. 0 cancels. While the screen (`screen_set`) is up,
    /// every key is sent that way and kept from the game.
    pub key_capture: unsafe extern "C" fn(token: *mut c_void, on: i32) -> W2rStatus,
    /// The manager's page in the Options screen (Options > Hotkeys, at the bottom of the list), drawn
    /// by `callback` every frame it is up; null removes it.
    pub screen_set: unsafe extern "C" fn(
        token: *mut c_void,
        callback: Option<unsafe extern "C" fn(screen: *const W2rScreen)>,
    ) -> W2rStatus,
    /// 1: opens the game's Options screen on the manager's page (a match only). 0: leaves the page,
    /// back to the Options categories.
    pub screen_open: unsafe extern "C" fn(token: *mut c_void, open: i32) -> W2rStatus,
    /// Appended after `screen_open` (`struct_size >= W2R_HOST_HOTKEY_GET_SIZE`): the key and alternate one of
    /// the caller's own actions is bound to now (its default while no manager has changed it). For
    /// an action the plugin polls itself (`W2R_HOTKEY_POLL`).
    pub hotkey_get: unsafe extern "C" fn(token: *mut c_void, action: W2rHandle, key_out: *mut W2rKey, alt_out: *mut W2rKey) -> W2rStatus,
}

/// `screen_open` codes besides 0 (leave the page) and 1 (open Options on the manager's page).
pub const W2R_SCREEN_CLOSE: i32 = 2;
/// `W2R_OK` while the Options screen is up, `W2R_E_STATE` otherwise.
pub const W2R_SCREEN_QUERY: i32 = 3;
/// `screen_open`: switch between Classic and HD graphics (as F5 in a match; off a match the
/// Classic Graphics choice for the next match).
pub const W2R_SCREEN_GRAPHICS: i32 = 4;
/// `screen_open`: keep the page's prepared art on disk (`w2rchepageart`) for faster later launches, or
/// not (and remove it). Off until a plugin turns it on.
pub const W2R_SCREEN_ART_CACHE_ON: i32 = 5;
pub const W2R_SCREEN_ART_CACHE_OFF: i32 = 6;
/// Open Options (or switch it) on this category.
pub const W2R_SCREEN_OPTIONS: i32 = 0x100;
pub const W2R_SCREEN_SOUND: i32 = 0x101;
pub const W2R_SCREEN_SPEED: i32 = 0x102;
pub const W2R_SCREEN_PREFERENCES: i32 = 0x103;
pub const W2R_SCREEN_GAMEPLAY: i32 = 0x104;
pub const W2R_SCREEN_MULTIPLAYER: i32 = 0x105;

/// At most this many `key_remap_set` entries.
pub const W2R_MAX_REMAPS: u32 = 512;
/// At most this many `hotkey_offer` actions per plugin.
pub const W2R_MAX_HOTKEYS: u32 = 64;


// ---- overlay --------------------------------------------------------------------------------
/// `W2rOverlayItem.kind`: a dotted line from (x0, y0) to (x1, y1).
pub const W2R_OVERLAY_LINE: u32 = 0;
/// `W2rOverlayItem.kind`: a filled circle with an outline centred on (x0, y0), `x1` its radius in
/// world pixels (0 draws a small solid dot); `y1` is ignored. `rgba` colours the fill; the outline
/// is the same colour, more opaque.
pub const W2R_OVERLAY_MARKER: u32 = 1;
/// `W2rOverlayItem.kind`: a solid line from (x0, y0) to (x1, y1) with small arrows along it
/// pointing from the first point to the second.
pub const W2R_OVERLAY_ARROWS: u32 = 2;
/// `W2rOverlayItem.kind`: a progress bar with its top-left corner at (x0, y0), `x1` pixels wide, 4
/// pixels high on a dark track; `y1` is the filled part in thousandths (0 to 1000), `rgba` its colour.
pub const W2R_OVERLAY_BAR: u32 = 3;
/// `W2rOverlayItem.kind`: a rectangle with its top-left corner at (x0, y0), `x1` by `y1` world pixels.
/// `rgba` colours the translucent fill; the outline is the same colour, opaque.
pub const W2R_OVERLAY_RECT: u32 = 4;
/// Items a plugin may have up at once.
pub const W2R_MAX_OVERLAY_ITEMS: u32 = 1024;

/// One thing to draw on the map view. Coordinates are world pixels (tile * 32; add 16 for a
/// tile's centre); `rgba` is 0xRRGGBBAA.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct W2rOverlayItem {
    pub kind: u32,
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
    pub rgba: u32,
}

// ---- declared settings ----------------------------------------------------------------------
/// A checkbox. Stored with `settings_set_i64` as 0 or 1.
pub const W2R_SETTING_BOOL: u32 = 0;
/// A whole number, `settings_set_i64`. A slider when `min < max`, a read-only row otherwise.
pub const W2R_SETTING_INT: u32 = 1;
/// A dropdown over `choices`. Stored with `settings_set_i64` as the chosen index.
pub const W2R_SETTING_CHOICE: u32 = 2;
/// A text box, `settings_set_str`.
pub const W2R_SETTING_STRING: u32 = 3;
/// A color picker. Stored with `settings_set_str` as six lowercase hex digits, `rrggbb`.
pub const W2R_SETTING_COLOR: u32 = 4;
/// A button. Pressing it sets the value to 1 and applies the screen at once, so the plugin gets
/// `W2R_EVT_SETTINGS_CHANGED` with the key at 1 (with any other pending changes). The plugin does
/// its action and sets the key back to 0 with `settings_set_i64`. Stored like a bool.
pub const W2R_SETTING_ACTION: u32 = 5;

/// `W2rSettingDef.flags`: loaded, saved and readable as usual, but not shown on the screen.
pub const W2R_SETTING_HIDDEN: u32 = 1;
/// `W2rSettingDef.flags`: a key setting the plugin hands to the hotkey manager: not shown while one
/// is active (`hotkeys_managed`), shown as usual otherwise.
pub const W2R_SETTING_HOTKEY: u32 = 4;
/// `W2rSettingDef.flags`: the screen applies as soon as this setting changes, as a button does, so
/// the plugin can react (a dropdown that fills other settings, say) without waiting for Apply.
pub const W2R_SETTING_LIVE: u32 = 2;

/// One setting, as `settings_declare` takes it. Every string is copied during the call, and fields
/// a kind does not use are ignored. The order keeps every field at the same offset under any 32-bit
/// compiler: each 64-bit field is 8-aligned and the size is a multiple of 8.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct W2rSettingDef {
    /// `W2R_SETTING_DEF_V1_SIZE`.
    pub struct_size: u32,
    /// `W2R_SETTING_BOOL`, `_INT`, `_CHOICE`, `_STRING` or `_COLOR`.
    pub kind: u32,
    /// The default: 0 or 1 for a bool, the number for an int, the index for a choice.
    pub default_int: i64,
    /// `W2R_SETTING_INT` only: the slider's range, inclusive. `min == max` draws no slider.
    pub min: i64,
    pub max: i64,
    /// `W2R_SETTING_HIDDEN` or 0.
    pub flags: u32,
    pub choice_count: u32,
    /// `W2R_SETTING_CHOICE` only: 2 to `W2R_MAX_SETTING_CHOICE_COUNT` labels, in dropdown order.
    pub choices: *const W2rStr,
    /// Must be 0.
    pub reserved: u32,
    /// The settings key, under the same rule as `settings_set_*`.
    pub key: W2rStr,
    /// What the row says. Title Case, like the rest of the screen.
    pub label: W2rStr,
    /// Shown when the mouse is over the row; may be empty.
    pub description: W2rStr,
    /// The entry in the screen's left-hand list (`"Chat"`); empty uses the plugin's name. Several
    /// plugins may share one, which is the point: similar settings end up together.
    pub category: W2rStr,
    /// The heading inside the category (`"Behavior"`); empty uses `"General"`.
    pub subcategory: W2rStr,
    /// `W2R_SETTING_STRING`: the default text. `W2R_SETTING_COLOR`: the default, `rrggbb`.
    pub default_str: W2rStr,
    /// Greys the row out while a checkbox is off: the key of a `W2R_SETTING_BOOL` this plugin
    /// declared earlier, or `host:<key>` for one of the host's own. Empty = always active.
    pub active_when: W2rStr,
}

/// `W2rSettingDef.struct_size` of the first version.
pub const W2R_SETTING_DEF_V1_SIZE: u32 = core::mem::size_of::<W2rSettingDef>() as u32;

impl W2rStr {
    /// A view of `text`; the host copies what it needs during the call it is passed to.
    pub const fn new(text: &str) -> W2rStr {
        W2rStr {
            ptr: text.as_ptr().cast(),
            len: text.len() as u32,
        }
    }
}

impl W2rSettingDef {
    /// A definition of `kind` with no default, range, choices, category or condition; set what the
    /// kind uses on the result. `category` and `subcategory` may be empty.
    pub const fn new(
        kind: u32,
        key: &str,
        label: &str,
        description: &str,
        category: &str,
        subcategory: &str,
    ) -> W2rSettingDef {
        W2rSettingDef {
            struct_size: W2R_SETTING_DEF_V1_SIZE,
            kind,
            default_int: 0,
            min: 0,
            max: 0,
            flags: 0,
            choice_count: 0,
            choices: core::ptr::null(),
            reserved: 0,
            key: W2rStr::new(key),
            label: W2rStr::new(label),
            description: W2rStr::new(description),
            category: W2rStr::new(category),
            subcategory: W2rStr::new(subcategory),
            default_str: W2rStr::new(""),
            active_when: W2rStr::new(""),
        }
    }
}

/// Declares `def` on a host that has `settings_declare`; `W2R_E_UNSUPPORTED` on an older one.
/// Call it only from `on_load`.
///
/// # Safety
/// `host` must be the pointer the host passed to `on_load`.
pub unsafe fn settings_declare(host: *const W2rHost, def: &W2rSettingDef) -> W2rStatus {
    let Some(h) = (unsafe { host.as_ref() }) else {
        return W2R_E_INVALID;
    };
    if h.struct_size < W2R_HOST_SETTINGS_DECLARE_SIZE {
        return W2R_E_UNSUPPORTED;
    }
    unsafe { (h.settings_declare)(h.token, def) }
}

/// Replaces the plugin's overlay on a host that has `overlay_set`; `W2R_E_UNSUPPORTED` on an
/// older one.
///
/// # Safety
/// `host` must be the pointer the host passed to `on_load`.
pub unsafe fn overlay_set(host: *const W2rHost, items: &[W2rOverlayItem]) -> W2rStatus {
    let Some(h) = (unsafe { host.as_ref() }) else {
        return W2R_E_INVALID;
    };
    if h.struct_size < W2R_HOST_OVERLAY_SIZE {
        return W2R_E_UNSUPPORTED;
    }
    unsafe { (h.overlay_set)(h.token, items.as_ptr(), items.len() as u32) }
}

/// Replaces a declared choice's entries on a host that has `settings_set_choices`;
/// `W2R_E_UNSUPPORTED` on an older one.
///
/// # Safety
/// `host` must be the pointer the host passed to `on_load`.
pub unsafe fn settings_set_choices(host: *const W2rHost, key: &str, choices: &[W2rStr]) -> W2rStatus {
    let Some(h) = (unsafe { host.as_ref() }) else {
        return W2R_E_INVALID;
    };
    if h.struct_size < W2R_HOST_SET_CHOICES_SIZE {
        return W2R_E_UNSUPPORTED;
    }
    let key = W2rStr {
        ptr: key.as_ptr().cast(),
        len: key.len() as u32,
    };
    unsafe { (h.settings_set_choices)(h.token, key, choices.as_ptr(), choices.len() as u32) }
}

/// Fills a snapshot on a host that has `game_snapshot`; `W2R_E_UNSUPPORTED` on an older one.
///
/// # Safety
/// `host` must be the pointer the host passed to `on_load`.
pub unsafe fn game_snapshot(host: *const W2rHost, out: &mut W2rGameSnapshot) -> W2rStatus {
    let Some(h) = (unsafe { host.as_ref() }) else {
        return W2R_E_INVALID;
    };
    if h.struct_size < W2R_HOST_SNAPSHOT_SIZE {
        return W2R_E_UNSUPPORTED;
    }
    out.struct_size = core::mem::size_of::<W2rGameSnapshot>() as u32;
    unsafe { (h.game_snapshot)(h.token, out) }
}

/// Registers a hotkey on a host that has `hotkey_register`; `W2R_E_UNSUPPORTED` on an older one.
///
/// # Safety
/// `host` must be the pointer the host passed to `on_load`.
pub unsafe fn hotkey_register(host: *const W2rHost, combo: &str, id: u32) -> W2rStatus {
    let Some(h) = (unsafe { host.as_ref() }) else {
        return W2R_E_INVALID;
    };
    if h.struct_size < W2R_HOST_HOTKEY_SIZE {
        return W2R_E_UNSUPPORTED;
    }
    let combo = W2rStr {
        ptr: combo.as_ptr().cast(),
        len: combo.len() as u32,
    };
    unsafe { (h.hotkey_register)(h.token, combo, id) }
}

/// Registers a chat command on a host that has `command_register`; `W2R_E_UNSUPPORTED` on an
/// older one.
///
/// # Safety
/// `host` must be the pointer the host passed to `on_load`.
pub unsafe fn command_register(host: *const W2rHost, name: &str, id: u32) -> W2rStatus {
    let Some(h) = (unsafe { host.as_ref() }) else {
        return W2R_E_INVALID;
    };
    if h.struct_size < W2R_HOST_COMMAND_SIZE {
        return W2R_E_UNSUPPORTED;
    }
    let name = W2rStr {
        ptr: name.as_ptr().cast(),
        len: name.len() as u32,
    };
    unsafe { (h.command_register)(h.token, name, id) }
}

/// Sets or removes a HUD line on a host that has `hud_text_set`; `W2R_E_UNSUPPORTED` on an older
/// one. An empty `text` removes the line.
///
/// # Safety
/// `host` must be the pointer the host passed to `on_load`.
pub unsafe fn hud_text_set(host: *const W2rHost, key: &str, text: &str, place: u32) -> W2rStatus {
    let Some(h) = (unsafe { host.as_ref() }) else {
        return W2R_E_INVALID;
    };
    if h.struct_size < W2R_HOST_HUD_TEXT_SIZE {
        return W2R_E_UNSUPPORTED;
    }
    let s = |t: &str| W2rStr {
        ptr: t.as_ptr().cast(),
        len: t.len() as u32,
    };
    unsafe { (h.hud_text_set)(h.token, s(key), s(text), place) }
}

/// Lists matching units on a host that has `units_query`; `W2R_E_UNSUPPORTED` on an older one.
/// Returns the status and how many units matched (which may exceed `out.len()`).
///
/// # Safety
/// `host` must be the pointer the host passed to `on_load`.
pub unsafe fn units_query(
    host: *const W2rHost,
    query: &W2rUnitQuery,
    out: &mut [W2rUnit],
) -> (W2rStatus, u32) {
    let Some(h) = (unsafe { host.as_ref() }) else {
        return (W2R_E_INVALID, 0);
    };
    if h.struct_size < W2R_HOST_UNITS_SIZE {
        return (W2R_E_UNSUPPORTED, 0);
    }
    let mut total = 0u32;
    let status =
        unsafe { (h.units_query)(h.token, query, out.as_mut_ptr(), out.len() as u32, &mut total) };
    (status, total)
}

/// Gives an order on a host that has `order_issue`; `W2R_E_UNSUPPORTED` on an older one.
///
/// # Safety
/// `host` must be the pointer the host passed to `on_load`.
pub unsafe fn order_issue(host: *const W2rHost, request: &W2rOrderRequest) -> W2rStatus {
    let Some(h) = (unsafe { host.as_ref() }) else {
        return W2R_E_INVALID;
    };
    if h.struct_size < W2R_HOST_ORDER_SIZE {
        return W2R_E_UNSUPPORTED;
    }
    unsafe { (h.order_issue)(h.token, request) }
}

/// Sends a request on a host that has `http_request`; `W2R_E_UNSUPPORTED` on an older one.
///
/// # Safety
/// `host` must be the pointer the host passed to `on_load`.
pub unsafe fn http_request(host: *const W2rHost, request: &W2rHttpRequest) -> W2rStatus {
    let Some(h) = (unsafe { host.as_ref() }) else {
        return W2R_E_INVALID;
    };
    if h.struct_size < W2R_HOST_HTTP_SIZE {
        return W2R_E_UNSUPPORTED;
    }
    unsafe { (h.http_request)(h.token, request) }
}

// ---- plugin table: returned by the plugin --------------------------------------------------
/// Callbacks are `Option` because a plugin may leave any of them NULL; the host checks
/// `on_load` is present (`E_ENTRY` otherwise) and skips the others when absent.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct W2rPlugin {
    pub struct_size: u32,
    /// non-zero fails the load (E_LOAD)
    pub on_load: Option<unsafe extern "C" fn(host: *const W2rHost) -> W2rStatus>,
    pub on_event: Option<unsafe extern "C" fn(event: *const W2rEvent)>,
    pub on_ui: Option<unsafe extern "C" fn(ui: *const W2rUi)>,
    /// best effort
    pub on_unload: Option<unsafe extern "C" fn()>,
}

/// The single export every plugin DLL provides, named `w2r_plugin_main`.
pub type W2rPluginMain = unsafe extern "C" fn(host: *const W2rHost) -> *const W2rPlugin;
pub const W2R_PLUGIN_MAIN_SYMBOL: &core::ffi::CStr = c"w2r_plugin_main";

/// Minimum `struct_size` a version-1 `W2rPlugin` may report.
pub const W2R_PLUGIN_V1_SIZE: u32 = core::mem::size_of::<W2rPlugin>() as u32;
/// `struct_size` a version-1 host reports.
pub const W2R_HOST_V1_SIZE: u32 =
    (core::mem::offset_of!(W2rHost, symbol_address) + core::mem::size_of::<usize>()) as u32;
/// `struct_size` of a host that has the create-screen functions.
pub const W2R_HOST_CREATE_UI_SIZE: u32 =
    (core::mem::offset_of!(W2rHost, create_map_select) + core::mem::size_of::<usize>()) as u32;
/// `struct_size` of a host that also has `create_map_folder`.
pub const W2R_HOST_MAP_FOLDER_SIZE: u32 =
    (core::mem::offset_of!(W2rHost, create_map_folder) + core::mem::size_of::<usize>()) as u32;
/// `struct_size` of a host that also has `settings_declare`.
pub const W2R_HOST_SETTINGS_DECLARE_SIZE: u32 =
    (core::mem::offset_of!(W2rHost, settings_declare) + core::mem::size_of::<usize>()) as u32;
/// `struct_size` of a host that also has `overlay_set`.
pub const W2R_HOST_OVERLAY_SIZE: u32 =
    (core::mem::offset_of!(W2rHost, overlay_set) + core::mem::size_of::<usize>()) as u32;
/// `struct_size` of a host that also has `settings_set_choices`.
pub const W2R_HOST_SET_CHOICES_SIZE: u32 =
    (core::mem::offset_of!(W2rHost, settings_set_choices) + core::mem::size_of::<usize>()) as u32;

/// `struct_size` of a host that has `game_snapshot` (and everything before it).
pub const W2R_HOST_SNAPSHOT_SIZE: u32 =
    (core::mem::offset_of!(W2rHost, game_snapshot) + core::mem::size_of::<usize>()) as u32;

/// `struct_size` of a host that has `hotkey_register` (and everything before it).
pub const W2R_HOST_HOTKEY_SIZE: u32 =
    (core::mem::offset_of!(W2rHost, hotkey_register) + core::mem::size_of::<usize>()) as u32;

/// `struct_size` of a host that has `command_register` (and everything before it).
pub const W2R_HOST_COMMAND_SIZE: u32 =
    (core::mem::offset_of!(W2rHost, command_register) + core::mem::size_of::<usize>()) as u32;

/// `struct_size` of a host that has `hud_text_set` (and everything before it).
pub const W2R_HOST_HUD_TEXT_SIZE: u32 =
    (core::mem::offset_of!(W2rHost, hud_text_set) + core::mem::size_of::<usize>()) as u32;

/// `struct_size` of a host that has `units_query` (and everything before it).
pub const W2R_HOST_UNITS_SIZE: u32 =
    (core::mem::offset_of!(W2rHost, units_query) + core::mem::size_of::<usize>()) as u32;

/// `struct_size` of a host that has `order_issue` (and everything before it).
pub const W2R_HOST_ORDER_SIZE: u32 =
    (core::mem::offset_of!(W2rHost, order_issue) + core::mem::size_of::<usize>()) as u32;

/// `struct_size` of a host that has `http_request` (and everything before it).
pub const W2R_HOST_HTTP_SIZE: u32 =
    (core::mem::offset_of!(W2rHost, http_request) + core::mem::size_of::<usize>()) as u32;
/// `struct_size` of a host that has the hotkey service (`screen_open` and everything before it).
pub const W2R_HOST_HOTKEY_SERVICE_SIZE: u32 =
    (core::mem::offset_of!(W2rHost, screen_open) + core::mem::size_of::<usize>()) as u32;
/// `struct_size` of a host that also has `hotkey_get`.
pub const W2R_HOST_HOTKEY_GET_SIZE: u32 = core::mem::size_of::<W2rHost>() as u32;

// Mirrors tests/abi_layout.c. Keep both in sync.
const _: () = {
    use core::mem::{offset_of, size_of};
    assert!(W2R_ABI_VERSION == 1);
    assert!(offset_of!(W2rHost, struct_size) == 0);
    assert!(offset_of!(W2rHost, abi_version) == 4);
    assert!(offset_of!(W2rHost, token) == 8);
    assert!(offset_of!(W2rPlugin, struct_size) == 0);
    assert!(offset_of!(W2rUi, struct_size) == 0);
    assert!(offset_of!(W2rEvent, struct_size) == 0);
    assert!(offset_of!(W2rStr, len) == size_of::<*const c_void>());
    assert!(size_of::<W2rStatus>() == 4);
    assert!(W2R_OK == 0);
    assert!(W2R_E_DENIED < 0 && W2R_E_BUSY < 0);
    assert!(W2R_MAX_PEER_PAYLOAD + W2R_PEER_HEADER_BYTES <= 989);
    assert!(offset_of!(W2rSettingDef, struct_size) == 0);
    assert!(offset_of!(W2rSettingDef, default_int) == 8);
    assert!(offset_of!(W2rSettingDef, flags) == 32);
    assert!(offset_of!(W2rSettingDef, choices) == 40);
    assert!(offset_of!(W2rSettingDef, key) == if size_of::<usize>() == 4 { 48 } else { 56 });
    assert!(offset_of!(W2rSettingDef, active_when) == if size_of::<usize>() == 4 { 96 } else { 152 });
    // Option<fn> must be pointer-sized with NULL as None for the plugin table to match C.
    assert!(size_of::<Option<unsafe extern "C" fn()>>() == size_of::<*const c_void>());
};

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::size_of;

    // The game target. 23 host function pointers after a 12-byte header.
    #[cfg(target_pointer_width = "32")]
    #[test]
    fn sizes_match_the_c_header_on_x86() {
        assert_eq!(size_of::<W2rStr>(), 8);
        assert_eq!(size_of::<W2rEvent>(), 12);
        assert_eq!(size_of::<W2rPlugin>(), 20);
        assert_eq!(size_of::<W2rUi>(), 40);
        assert_eq!(W2R_UI_TEXT_EDIT_SIZE, 32);
        assert_eq!(W2R_UI_COLOR_EDIT_SIZE, 36);
        assert_eq!(W2R_UI_COMBO_SIZE, 40);
        assert_eq!(size_of::<W2rHost>(), 184);
        assert_eq!(W2R_HOST_HOTKEY_SERVICE_SIZE, 180);
        assert_eq!(W2R_HOST_HOTKEY_GET_SIZE, 184);
        assert_eq!(size_of::<W2rHotkeyDef>(), 40);
        assert_eq!(size_of::<W2rHotkeyInfo>(), 64);
        assert_eq!(size_of::<W2rRemap>(), 16);
        assert_eq!(size_of::<W2rScreen>(), 76);
        assert_eq!(W2R_SCREEN_V1_SIZE, 72);
        assert_eq!(size_of::<W2rOverlayItem>(), 24);
        assert_eq!(W2R_HOST_OVERLAY_SIZE, 108);
        assert_eq!(W2R_HOST_SET_CHOICES_SIZE, 112);
        assert_eq!(W2R_HOST_SNAPSHOT_SIZE, 116);
        assert_eq!(W2R_HOST_V1_SIZE, 84);
        assert_eq!(W2R_PLUGIN_V1_SIZE, 20);
        assert_eq!(W2R_HOST_CREATE_UI_SIZE, 96);
        assert_eq!(W2R_HOST_MAP_FOLDER_SIZE, 100);
        assert_eq!(W2R_HOST_SETTINGS_DECLARE_SIZE, 104);
        assert_eq!(size_of::<W2rSettingDef>(), 104);
    }

    // Not a shipping target, but the tests run here too; pointer alignment pads the u32 headers.
    #[cfg(target_pointer_width = "64")]
    #[test]
    fn sizes_match_the_c_header_on_x64() {
        assert_eq!(size_of::<W2rStr>(), 16);
        assert_eq!(size_of::<W2rEvent>(), 16);
        assert_eq!(size_of::<W2rPlugin>(), 40);
        assert_eq!(size_of::<W2rUi>(), 80);
        assert_eq!(W2R_UI_TEXT_EDIT_SIZE, 64);
        assert_eq!(size_of::<W2rHost>(), 360);
        assert_eq!(W2R_HOST_MAP_FOLDER_SIZE, 192);
        assert_eq!(size_of::<W2rSettingDef>(), 168);
    }
}
