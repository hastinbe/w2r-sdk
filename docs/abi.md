# Plugin ABI (version 1)

The plugin interface is a C ABI so plugin authors can use any language. The header is
[`crates/w2r-abi/include/w2r_abi.h`](../crates/w2r-abi/include/w2r_abi.h); this document gives the
rules and limits the header cannot express. The ABI is **add-only**.

Capabilities are a consent prompt and a guardrail, **not a sandbox**. A native plugin runs inside
the game process and can do anything that process can. The host enforces the limits below on the
functions it provides; it cannot stop a plugin from using the operating system directly.

## Conventions

- Calling convention `__cdecl` on 32-bit Windows (`W2R_CALL`). Only fixed-width C types cross the
  boundary; no C++ or Rust types.
- **Threading:** every host function and every plugin callback runs on the game's main thread. A
  host function called from any other thread returns `W2R_E_STATE`.
- Status codes (`W2rStatus`, `int32_t`): `0` is success, every failure is negative.

| Code | Meaning |
|---|---|
| `W2R_OK` 0 | success |
| `W2R_E_DENIED` -1 | the capability was not approved |
| `W2R_E_INVALID` -2 | bad argument, or a game-memory access failed |
| `W2R_E_LIMIT` -3 | a size or count limit was exceeded |
| `W2R_E_STATE` -4 | not allowed right now (wrong thread, not in a lobby, ...) |
| `W2R_E_UNSUPPORTED` -5 | the host knows the function but cannot do it |
| `W2R_E_NOT_FOUND` -6 | no such symbol, handle or event |
| `W2R_E_RATE` -7 | rate limit hit; the call had no effect |
| `W2R_E_CONFLICT` -8 | the hook overlaps an existing patch |
| `W2R_E_BUSY` -9 | not ready yet, ask again on a later frame |

## Loading and versioning

1. The host loads the DLL (after the checks in [load-policy.md](load-policy.md)) and calls its one
   export, `w2r_plugin_main(const W2rHost *host)`.
2. The plugin returns a pointer to a static `W2rPlugin` table. Null means "refuse to load".
3. The host calls `on_load(host)`. A non-zero return fails the load (`E_LOAD`).

`W2rHost`, `W2rPlugin`, `W2rUi` and `W2rEvent` begin with `struct_size`. Fields are only ever
appended; nothing is removed, reordered or resized.

- A plugin must check that `host->struct_size` covers a field before using it (and that the
  function pointer is non-null). A newer plugin on an older host sees a smaller `struct_size`.
- The host calls a plugin callback only if the plugin's `struct_size` covers it and it is non-null.
- A function the host provides but whose capability was not approved returns `W2R_E_DENIED`; the
  host logs this once per function per plugin.
- `W2R_ABI_VERSION` increases only for a change that cannot be made by appending. The manifest `abi`
  field is the minimum ABI the plugin needs; the host refuses a greater value (`E_ABI`).
- Each plugin receives its own `W2rHost` instance. Pass `host->token` back as the first argument of
  every call; it tells the host which plugin is calling.

## Host functions

Implicit capabilities need no approval: `log`, `settings`, `ui`, `fs.data`.

| Function | Capability | Rules |
|---|---|---|
| `log` | implicit | message at most 512 bytes; throttled to 20 lines/s per plugin with a burst of 40; excess lines are dropped and one summary line per second is logged |
| `settings_get_i64`, `settings_set_i64`, `settings_get_str`, `settings_set_str` | implicit | the plugin's own settings only; key 1-64 bytes of `[a-z0-9_.]`; at most 128 keys; string value at most 1,024 bytes; stored in `config\<plugin-id>.json`, at most 256 KiB. Statuses: a missing key is `W2R_E_NOT_FOUND`, a key holding the other type is `W2R_E_INVALID`, a bad key `W2R_E_INVALID` (too long `W2R_E_LIMIT`), the 129th key or an oversized value `W2R_E_LIMIT`, a failed save `W2R_E_STATE` (and the old value stays). `settings_get_str` copies the value with no terminator and always sets `len_out` to the full length; a value longer than `cap` leaves `buf` untouched and returns `W2R_E_LIMIT`. Changes are written to disk immediately |
| `data_open`, `data_read`, `data_write`, `data_close` | implicit (`fs.data`) | flat files in the plugin's own `data\` folder; name 1-64 bytes of `[A-Za-z0-9_.-]`, not starting with `.`, not a device name; at most 64 files, 16 MiB in total, 8 open handles; at most 65,536 bytes per read or write call; the plugin never sees a path. `flags` is exactly `W2R_FILE_READ` (the file must exist, `W2R_E_NOT_FOUND` otherwise; read from the start) or `W2R_FILE_WRITE` (create or truncate; writes append from there); a handle cannot do the other (`W2R_E_STATE`), anything else is `W2R_E_INVALID`. Names ending in `.` are refused too (Windows drops the dot). A link or non-regular file in the folder is never followed. `data_read` sets `read_out` to 0 at end of file. Without a host root (tests) all of these report `W2R_E_STATE` |
| `subscribe`, `unsubscribe` | per event | see Events; unknown ids return `W2R_E_NOT_FOUND` |
| `symbol_lookup`, `symbol_read` | `game.read` | symbol name at most 64 bytes; the handle is valid for the session; at most 4,096 bytes per read; a read that faults returns `W2R_E_INVALID` and never crashes the game |
| `chat_send` | `chat.write` | at most 200 bytes of printable ASCII; at most 4 per 5 s per plugin; only when the game's chat is available: in a lobby it goes through the lobby's chat box, in a match as a public line to every other player (recorded in replays like a typed line). Not while only watching or in a replay |
| `http_request` | `net.http` | HTTPS to the servers the manifest's `net_hosts` names; the answer arrives as `W2R_EVT_HTTP_RESPONSE` (see HTTP) |
| `units_query` | `game.read` | the units the local player may know about (see Units and orders); at most 256 per call |
| `order_issue` | `game.control` | an order for up to 12 of the local player's own units, through the game's command queue (see Units and orders) |
| `peer_send` | `net.send` | payload 1 to 960 bytes (`W2R_E_INVALID` empty, `W2R_E_LIMIT` over); at most 32 messages/s per plugin and 128/s overall (`W2R_E_RATE`); lobby only, with the session connected (`W2R_E_STATE`); the target slot (0-7, else `W2R_E_INVALID`) must have announced the handshake, or `W2R_SLOT_ALL` for every such peer; no announced peer is `W2R_E_STATE` |
| `symbol_address` | `hooks.raw` (shown as **dangerous**) | the runtime address (image base plus the symbol's RVA) of any symbol, function or global, always inside the game module; `W2R_E_NOT_FOUND` for a bad handle; the handle still comes from `symbol_lookup`, which needs `game.read`, so a plugin that calls game functions or writes game memory declares both. The address makes every later access the plugin's own, outside the host's guard |
| `hook_install`, `hook_remove` | `hooks.raw` (shown as **dangerous**) | `target` is a `symbol_lookup` handle of a function symbol (`W2R_E_NOT_FOUND` for a bad handle, `W2R_E_INVALID` for a global), so a hook is always inside the game module's code; `original_out` receives the trampoline before the hook goes live; no overlap with any host patch or other hook (`W2R_E_CONFLICT`); at most 64 hooks per plugin |

The UI (`W2rUi`) is passed to `on_ui` and is valid only inside that call. It offers `label`,
`button`, `checkbox` and `slider_int`; each call draws one row in the plugin's card in the plugin
manager (opened from the title screen). A plugin's settings belong in its declarations instead (next section), so
they appear in the options screen's categories with everyone else's; `on_ui` remains for what a
declaration cannot express. Strings are at most 128 bytes of UTF-8;
a longer or invalid string, a null value pointer or `min >= max` makes the call a no-op that
reports 0. `button` returns 1 when pressed this frame; `checkbox` and `slider_int` return 1 when
they changed the value this frame (the slider clamps it to the range first). `ctx` is opaque.

Two calls were appended after the first release (`struct_size` grew; a plugin checks
`ui->struct_size >= W2R_UI_TEXT_EDIT_SIZE` before using them, so it still runs on an older host):

- `text_edit(ctx, text, buf, cap)`: a label and an edit box over the plugin's own NUL-terminated
  buffer of `cap` bytes (1 to 256; otherwise a no-op that reports 0). The game's edit widget
  changes the buffer in place while the player types; the call returns 1 on a frame where the text
  changed. The plugin keeps the buffer between frames and passes it again. A buffer with no NUL is
  terminated at its last byte before the game sees it. The text is whatever the player types (the
  game's edit filter applies); the plugin must validate it.
- `set_active(ctx, active)`: 0 greys out and disables the rows that follow, 1 restores them. A
  grey-out left open at the end of `on_ui` is closed by the host.

One more was appended after those (`ui->struct_size >= W2R_UI_COLOR_EDIT_SIZE`; on an older host a
plugin falls back to `text_edit`):

- `color_edit(ctx, text, hex)`: a color row like the options screen's: the caption, a swatch, and
  the value on a `#rrggbb` button that opens the host's color editor (the same one the options
  screen uses: saturation / brightness square, hue strip, the new and the current color, a hex box,
  OK / Cancel; no Reset to Default, since a plugin card has no declared default). `hex` is the
  plugin's own buffer of `W2R_COLOR_HEX_BYTES` (7) bytes, six hex digits and a NUL; anything else
  shows as black. The call returns 1 on the frame after the player pressed OK on a different color,
  with `hex` rewritten as six lowercase digits and a NUL; Cancel changes nothing. The row is known
  by the plugin and its caption, so captions of color rows should differ. While the editor is open
  the plugin manager behind it is read-only; opening the manager again closes an editor left open.

And one after `color_edit` (`ui->struct_size >= W2R_UI_COMBO_SIZE`):

- `combo(ctx, text, choices, count, selected)`: a dropdown row like the options screen's: the
  caption, then a dropdown showing `choices[*selected]`, the game's own dropdown widget. `count` is
  2 to `W2R_MAX_SETTING_CHOICE_COUNT` (16) and each choice at most `W2R_MAX_SETTING_CHOICE` (64)
  bytes; otherwise, or with a null pointer, a no-op that reports 0. Unlike a declared dropdown the
  list is the plugin's to change every frame. A `*selected` out of range shows the last entry. The
  call returns 1 on the frame the player picked a different entry, with `*selected` set to it.
  The open dropdown's rows shrink to fit, as on the options screen.

## Declared settings

Appended after `create_map_folder` (`host->struct_size >= W2R_HOST_SETTINGS_DECLARE_SIZE`; an
older host has a smaller table). Implicit, no capability. The options screen
(settings-screen.md) shows every setting the game, the host and the plugins
have in one place: categories down the left, the selected category's settings on the right grouped
under subcategory headings, a search page, and the description on hover. A plugin puts its own
settings there by declaring them.

`settings_declare(token, def)` declares one setting. It is accepted only while the plugin's
`on_load` runs (`W2R_E_STATE` after), and every string in `def` is copied during the call.

| Field | Rule |
|---|---|
| `struct_size` | at least `W2R_SETTING_DEF_V1_SIZE` (`W2R_E_INVALID` otherwise) |
| `reserved` | 0 (`W2R_E_INVALID` otherwise) |
| `kind` | `W2R_SETTING_BOOL` (a checkbox, stored with `settings_set_i64` as 0 or 1), `_INT` (a slider when `min < max`, a read-only row otherwise), `_CHOICE` (a dropdown, stored as the chosen index), `_STRING` (a text box, `settings_set_str`), `_COLOR` (a color picker, stored with `settings_set_str` as six lowercase hex digits, `rrggbb`) or `_ACTION` (a button: pressing it sets the value to 1 and applies the screen at once, so the plugin gets `W2R_EVT_SETTINGS_CHANGED`; it does its action and sets the key back to 0); anything else `W2R_E_INVALID` |
| `flags` | 0, or any of `W2R_SETTING_HIDDEN` (stored and readable as usual, never shown) and `W2R_SETTING_LIVE` (the screen applies as soon as the setting changes, as for a button; a host without it refuses the flag, so declare again without it); other bits `W2R_E_INVALID` |
| `key` | the settings-key rule above; declared once per plugin (`W2R_E_CONFLICT` the second time); at most 128 declarations, the same cap as stored keys |
| `label` | 1 to 64 bytes. Title Case, like the rest of the screen |
| `description` | at most 256 bytes; may be empty. The hover text. The tooltip's last line names the plugin (its manifest `name`) |
| `category`, `subcategory` | at most 32 bytes each. Several plugins may name the same category, which is the point: related settings end up together. An empty category is the plugin's name; an empty subcategory is "General" |
| `default_int` | 64-bit, like the stored value. 0 or 1 for a bool, a value in `[min, max]` for a slider, an index into `choices` for a choice (`W2R_E_INVALID` otherwise) |
| `min`, `max` | 64-bit; `_INT` only; `min > max` is `W2R_E_INVALID` |
| `default_str` | `_STRING`: at most 1,024 bytes. `_COLOR`: six hex digits without `#` (`W2R_E_INVALID` otherwise) |
| `choices`, `choice_count` | `_CHOICE` only: 2 to 16 labels of 1 to 64 bytes (`W2R_E_LIMIT` above 16, `W2R_E_INVALID` below 2) |
| `active_when` | empty, or the key of a `_BOOL` this plugin declared earlier, or `host:<key>` for one of the host's own bools (for example `host:enhanced_create_screen`). The row is greyed while that checkbox is off. An unknown key is `W2R_E_NOT_FOUND`, a non-bool `W2R_E_INVALID` |

The host reads only the fields the kind uses, so the others may be left uninitialised; the
field order keeps every offset the same under any 32-bit compiler. Every text field is printable
ASCII (the screen draws through the game's font atlas, which has no other glyphs): anything else
is `W2R_E_INVALID`, and an over-long field `W2R_E_LIMIT`. A refused declaration is logged with
its key and status.

Values stay where they always were, in `config\<plugin-id>.json`, and the plugin reads them with
`settings_get_i64` / `settings_get_str` as before. A declared key that was never stored reads as
its declared default (instead of `W2R_E_NOT_FOUND`). The screen writes a plugin's values only
when the player presses Apply or Save and Exit, so a plugin never sees a half-made edit; then it
sends the plugin `W2R_EVT_SETTINGS_CHANGED`, once per apply, after all of that plugin's changed
values are stored. A plugin that subscribed to it re-reads its settings there. Values the plugin
stores itself with `settings_set_*` are what the screen shows the next time it opens.

A button (`W2R_SETTING_ACTION`) applies the screen when pressed, so its handler is the same
`W2R_EVT_SETTINGS_CHANGED`: the plugin finds its key at 1, acts, and sets it back to 0.

`settings_set_choices(token, key, choices, count)` (host `struct_size` at least
`W2R_HOST_SET_CHOICES_SIZE`) replaces the entries of one of the plugin's declared
`W2R_SETTING_CHOICE` settings, any time on the main thread: 2 to 16 labels of 1 to 64 bytes
(`W2R_E_LIMIT` above 16 or 64, `W2R_E_INVALID` for fewer than 2 or an empty label), copied during
the call; `W2R_E_NOT_FOUND` for a key that is not a declared choice. The options screen shows the new
entries as soon as the apply that triggered the call ends. The stored value is still an index, so
a plugin that shortens or reorders the list sets the key again.

## Overlay

Appended after `settings_declare` (`host->struct_size >= W2R_HOST_OVERLAY_SIZE`). Implicit, no
capability: the host draws, the plugin only describes.

`overlay_set(token, items, count)` replaces what the plugin has up on the map view. During a
match, every frame, the host paints each plugin's items over the map (under the minimap pings,
over the units) until the plugin calls again; `count` 0 clears. At most `W2R_MAX_OVERLAY_ITEMS`
(1024) items (`W2R_E_LIMIT` above). Main thread only, like every host call. Nothing is drawn
outside a match, and a plugin should clear its overlay in `on_unload`.

A `W2rOverlayItem` is `kind`, `x0`, `y0`, `x1`, `y1` and `rgba`. Coordinates are world pixels
(tile * 32; a tile's centre is +16); `rgba` is `0xRRGGBBAA`. `W2R_OVERLAY_LINE` is a dotted line
from (x0, y0) to (x1, y1); `W2R_OVERLAY_ARROWS` a solid line from (x0, y0) to (x1, y1) with small
arrows along it pointing at the second point; `W2R_OVERLAY_MARKER` a filled circle with an outline
centred on (x0, y0), radius `x1` world pixels (0: a small solid dot), `rgba` the fill and the
outline the same colour, more opaque; `W2R_OVERLAY_RECT` a rectangle with its top-left corner at
(x0, y0), `x1` by `y1` world pixels, `rgba` the translucent fill and the outline the same colour,
opaque (core 1.149.0; the waypoint assist's footprints of queued buildings). Parts off the view are not drawn. First user: the waypoint assist's shift-queue lines (core 1.61.0).

## Create screen

Appended after the first release (`host->struct_size >= W2R_HOST_CREATE_UI_SIZE`, `W2R_HOST_CREATE_UI_SIZE` in the header; an older host has a smaller table). Implicit, no capability: the plugin
can only change which map the local player is about to host, the same as picking one in the map
browser, and `affects_simulation` stays the plugin's own declaration.

- `create_ui_register(token, callback)` registers the plugin's one create-screen callback (null
  removes it; registering again replaces it). While the enhanced Create Multiplayer Game screen
  draws, the host calls it every frame with a `W2rUi`, under the same guard as `on_ui`. The rows
  appear under the map preview and the Change map button, in a strip reserved for them (room for a
  slider with its caption and one button; rows that do not fit are not drawn). `label`, `button`
  and `slider_int` work; `checkbox` shows its text and never changes; `text_edit` does nothing.
  Nothing is called, and no room is reserved, when no plugin registered one or when the stock
  create screen is in use.
- `create_map_count(token, players, count_out)` and `create_map_select(token, players, nth)` work
  only inside that callback (`W2R_E_STATE` elsewhere). They see the host's map index, the one the
  map browser's search uses: every map under the maps folder, up to the browser's caps. `players`
  is the map's maximum player count, 1 to 8, or 0 for any (`W2R_E_INVALID` above 8). The index is
  read from its cache or built a slice per frame the first time a plugin calls `create_map_count`;
  until it is ready the call returns `W2R_E_BUSY` and the plugin asks again next frame. Maps keep
  their index order while the index is not rebuilt (reload in the browser rebuilds it). `nth` is
  0-based and below the last count (`W2R_E_NOT_FOUND` otherwise, also for a map that vanished since
  indexing). A selection is exactly what Confirm in the map browser does: the game's map list is
  pointed at the map's folder and the map is selected for the game being created.
- `create_map_folder(token, folder)` (appended again; `host->struct_size >=
  W2R_HOST_MAP_FOLDER_SIZE`) limits `create_map_count` and `create_map_select` to the maps in
  `folder` and its subfolders. `folder` is relative to the maps folder (`\` or `/`, case-insensitive,
  whole folder names; empty removes the limit). The limit ends when the callback returns, so call it
  first, every frame. `W2R_E_STATE` outside the callback; `W2R_E_INVALID` for `..`, a drive colon or
  more than 200 bytes. A folder with no maps gives a count of 0.

## Strings and untrusted data

- `W2rStr` is UTF-8 and length-delimited, not NUL-terminated.
- Player names and chat text from other players are reduced to printable ASCII before the plugin
  sees them (anything else becomes `?`), as in wc2r-plus.
- Plugins never receive raw peer buffers except the bounded payload of `W2rEvtPeerMessage`, and
  never receive filesystem paths.
- Lifetime: pointers the host passes in an event or UI call are valid only during that call. The
  host copies plugin-provided data that it keeps.

## Events

Delivered to `on_event` on the main thread, only to plugins that subscribed. Peer messages are
delivered only while the local player is in a lobby, and a net slot that sends more than 128 W2R
messages per second has the excess dropped before any plugin sees them.

| Event | Payload | Capability | Fires |
|---|---|---|---|
| `W2R_EVT_LOBBY_ENTER` 1 | none | none | the local player enters a lobby |
| `W2R_EVT_LOBBY_LEAVE` 2 | none | none | the local player leaves a lobby |
| `W2R_EVT_LOBBY_PLAYER_JOIN` 3 | `W2rEvtPlayer` | none; `name` is empty without `chat.read` | a human appears in a lobby seat. The first look after the lobby opens reports everyone already there, the local player included. Looked at 4 times a second |
| `W2R_EVT_LOBBY_PLAYER_LEAVE` 4 | `W2rEvtPlayer` | same | a human's seat empties or its name changes |
| `W2R_EVT_CHAT_RECEIVED` 5 | `W2rEvtChat` | `chat.read` | a public line a lobby member or a match player said (whispers never reach plugins). Raised after the host's own ignore list; `net_slot` is the sender's net slot and `name` the sender. Lines are printable ASCII, at most 200 bytes |
| `W2R_EVT_PEER_MESSAGE` 6 | `W2rEvtPeerMessage` | `net.send` | a plugin message with this plugin's tag arrives |
| `W2R_EVT_MATCH_START` 7 | none | none | a match begins (the game resets its map-message slots, as wc2r-plus detects it); delivered before the game's own reset runs |
| `W2R_EVT_MATCH_END` 8 | `W2rEvtMatchEnd` | none; player names are empty without `chat.read` | the local player's `PlayerMatchResult` became non-zero (victory, defeat or leaving), once per match; fired from a host hook on `CheckMatchEnd` after the game's own function ran |
| `W2R_EVT_SETTINGS_CHANGED` 9 | none | none | the options screen stored new values for this plugin's declared settings (Apply or Save and Exit), once per apply |
| `W2R_EVT_SHUTDOWN` 10 | none | none | the game is exiting |
| `W2R_EVT_GAME_MODE` 11 | `W2rEvtGameMode` | none | the lobby's game mode changed (host choice, or the host's message arrived) or was reset when a match ended; `flags` bit 0 is Classic Balance, bit 1 says it is for a recording being played (replay, live watch), which is not a networked game. Hosts only turn a mode on when every player has the mod and the plugin that declares it, so a plugin that gets the flag can rely on every machine having it |

`subscribe` for an event whose capability is not approved returns `W2R_E_DENIED`. New events are
only added, with new ids.

## Peer messages

Plugins send through `peer_send`; the host builds the envelope:

```
[0]      0xE7            message type (unused by the base game)
[1..3]   'W' '2' 'R'     magic
[4]      'P'             subsystem: plugin message
[5..8]   tag             first 4 bytes of SHA-256 of the plugin id (UTF-8)
[9..]    payload         at most 960 bytes
```

- The tag is the same on every machine, so peers agree without negotiating. The host drops a
  message whose tag matches no loaded plugin. Two plugins with the same tag cannot both load
  (`E_TAG`).
- A plugin receives only messages carrying its own tag, only from peers that announced the
  handshake, and only while in a lobby. The sender slot is the one the transport reports, never one
  named inside a message.
- Subsystems `'H'`, `'M'` and `'L'` belong to the first-party handshake, map-sharing and
  lobby-settings features, which are compiled into the host rather than shipped as plugins: the
  handshake's roster is the host's own policy input and its fingerprint covers what the host
  loaded, so no plugin could provide it (peer-protocol.md). `'P'` is the only
  subsystem third-party plugins use.
- A whole message is at most 969 bytes, below the game's 989-byte limit for its reliable channel.
  Messages are removed from the stream before the game sees them, and the host never sends
  outside a lobby.

## Crash and unload contract

- Every call into a plugin runs under a host guard that contains panics and access violations. A
  plugin that faults is disabled for the rest of the session, and the reason is logged. The
  mechanism (settled by the Phase 2 spike, `crates/w2r-core/src/guard.rs`): a structured
  exception handling frame (`__try`/`__except` via `microseh`) outermost, with
  `std::panic::catch_unwind` inside it. A caught fault abandons the Rust frames between the
  handler and the faulting instruction without running destructors, so the guard owns no state
  that needs cleanup. Stack overflow is not recoverable.
- **Hook detours are outside this guard.** A detour installed with `hook_install` is called by the
  game, on whatever thread the game uses, not by the host. The host cannot wrap it, and the
  main-thread rule does not apply to it: a detour must be thread-safe and must not call host
  functions from a thread other than the main thread. A fault inside a detour can crash the game.
  This is part of why `hooks.raw` is shown as dangerous.
- When a plugin is disabled after a fault in one of its callbacks, the host also removes all of
  its hooks, cancels its event subscriptions and stops calling `on_ui`.
- A plugin must not unwind across the boundary. The Rust SDK catches panics; other languages must
  do the equivalent.
- `on_unload` is best effort and may be skipped after a fault. The host never relies on it to keep
  its own state consistent.
- The host cannot preempt a slow callback. It logs callbacks over 50 ms, and after 10 in a row the
  plugin manager shows the plugin's status as slow.

## Minimal plugin

```c
#include "w2r_abi.h"

static W2rStatus W2R_CALL on_load(const W2rHost *host) {
    W2rStr msg = {"hello", 5};
    return host->log(host->token, W2R_LOG_INFO, msg);
}
static const W2rPlugin kPlugin = {sizeof(W2rPlugin), on_load, 0, 0, 0};

W2R_EXPORT const W2rPlugin *W2R_CALL w2r_plugin_main(const W2rHost *host) {
    return host->abi_version >= 1 ? &kPlugin : 0;
}
```

This is compiled as part of `crates/w2r-abi/tests/abi_layout.c` by `bash tools/check_abi.sh`, which
also checks struct layout as C11, C++17 and, when a 32-bit toolchain is installed, 32-bit.

## Match end payload

`W2R_EVT_MATCH_END` carries a `W2rEvtMatchEnd` (a host that predates it sends NULL, so check). It
gives the local player's index, the number of game turns, whether the game flagged any turn as out
of sync (`W2R_MATCH_DESYNC`), the lobby's map name, and one `W2rMatchPlayer` per player who played
(humans and computers; step through the array by `player_size`, not `sizeof`). Per player:

- `flags`: `W2R_PLAYER_HUMAN`, `W2R_PLAYER_COMPUTER`, `W2R_PLAYER_LOCAL`.
- `result`: the game's own code, `W2R_RESULT_VICTORY` (3), `W2R_RESULT_DEFEAT` (2),
  `W2R_RESULT_NONE` (0, for a player whose result was never written, such as one who left or
  surrendered) or `W2R_RESULT_DRAW` (5); any other code is passed through.
- the tallies the game's results screen shows: units killed, buildings razed, units lost,
  buildings lost, and gold, lumber and oil mined.
- `name`: printable ASCII, empty without `chat.read`.

Every machine in the match sees the same simulation, so the tallies agree between players, but a
player who closed the game early may never raise the event. The event is raised once per match,
for the local player's own match end, not for a replay being watched.

## Game snapshot

`game_snapshot(token, W2rGameSnapshot *out)` (needs `game.read`; appended to the host table, check
`struct_size >= W2R_HOST_SNAPSHOT_SIZE`) fills in the match as the local player may know it. Set
`out->struct_size = sizeof(W2rGameSnapshot)` first. It returns `W2R_E_STATE` outside a match.

- `flags`: `W2R_SNAPSHOT_IN_MATCH`, `_PAUSED`, `_REPLAY` (a recording is playing), `_OBSERVER` (the
  local player is only watching).
- `game_ms` is the match clock, which stops while the game is paused; `turn` counts network turns.
- `local_player` is the player whose view is on screen (`W2R_NONE` if unknown).
- `players[0..player_count]` lists the humans and computers playing: index, `W2R_PLAYER_*` flags
  (human, computer, local, and `W2R_PLAYER_ALLY` for the local player's team), race (0 human,
  1 orc), lobby team, colour index, the result code (`W2R_RESULT_*`, non-zero once decided) and
  the resources.
- **Hidden values.** `gold`, `lumber`, `oil`, `food_used` and `food_cap` are -1 for a player who is
  not the local player or on the local player's team, unless the local player is only watching
  or a recording is playing. A plugin therefore learns nothing the game itself hides.
- `name` is empty without `chat.read` and is valid until the next `game_snapshot` call.

It costs a handful of memory reads; calling it once per second is fine, once per frame is not
needed. There is no event for changes.

## Hotkeys and commands

Two host functions, appended to the table (check `struct_size >= W2R_HOST_HOTKEY_SIZE` and
`W2R_HOST_COMMAND_SIZE`). Neither needs a capability, because the host decides what a plugin may
claim and the plugin only learns about the one key or command it asked for.

**`hotkey_register(token, combo, id)`** registers a key combination such as `"Ctrl+Alt+K"`,
`"Alt+F5"` or `"Ctrl+Shift+Insert"` (modifiers `Ctrl`, `Shift`, `Alt`; keys A to Z, 0 to 9, F1 to
F12, `Insert`, `Delete`, `Home`, `End`, `PageUp`, `PageDown`, the arrows, `Space`, `Tab`,
`Backtick`, `Minus`, `Equals`, `Comma`, `Period`, `Slash`; any case). When the game window has focus
and exactly those modifiers are down and the key goes down, the plugin gets `W2R_EVT_HOTKEY` with
`W2rEvtHotkey.id`, once per press, without subscribing.

- It must include Ctrl or Alt, or be an F key, so a hotkey never fires while someone types text.
- `W2R_E_INVALID`: not a combination, or no Ctrl/Alt and not an F key.
- `W2R_E_CONFLICT`: the host or the game uses it, or another plugin holds it. Reserved:
  `Alt+F`, `Alt+G`, `Alt+C`, `Ctrl+Shift+N`, `Ctrl+Shift+K`, `Ctrl+A/C/V/X/Z/Y`, `Alt+F4`,
  `Alt+Tab`, `Alt+0` to `Alt+9`, `F1`, `F10`, `F11`, `F12`.
- `W2R_E_LIMIT`: more than 8 hotkeys. A disabled plugin gives its keys up.

**`command_register(token, name, id)`** registers the chat command `/name`: 1 to 16 of `a-z`,
`0-9` and `-`, starting with a letter. When the player types it in the lobby or in a match chat
box, the line is taken from the box and not sent, and the plugin gets `W2R_EVT_COMMAND` with
`W2rEvtCommand.id` and `args`, the text after the name, trimmed, printable ASCII, at most 200
bytes. The name is matched case-insensitively.

- `W2R_E_INVALID`: not a valid name.
- `W2R_E_CONFLICT`: the host or the game uses it (`plugins`, `share`, `ignore`, `unignore`,
  `squelch`, `unsquelch`, `obs`, `g`, `w`, `r`, `reply`, `whisper`, `invite`, `report`, `away`,
  `dnd`, `me`, `mute`, `unmute`, `friend`, `unfriend`, `kick`, `ban`, `unban`, `help`, `motd`,
  `mod`, `gift`, `email`, `all`, `allies`, `team`) or another plugin holds it.
- `W2R_E_LIMIT`: more than 8 commands. Commands do not work in the Community chat tabs.

Registering the same combination or name with the same id twice is not an error.

## HUD text

`hud_text_set(token, key, text, place)` (appended; check `struct_size >= W2R_HOST_HUD_TEXT_SIZE`; no
capability) shows one line of text in the in-match HUD. `key` names the line (1 to 32 bytes) so the
plugin can change or remove it later; `text` is printable ASCII, at most 64 bytes, and an empty
`text` removes the line. `place` is one of:

| `place` | Where |
|---|---|
| `W2R_HUD_BOTTOM_RIGHT` (0) | above the APM counter and the clock, right aligned (the original place) |
| `W2R_HUD_TOP_LEFT` (1) | below the resource bar, left aligned |
| `W2R_HUD_TOP_RIGHT` (2) | below the resource bar, right aligned |
| `W2R_HUD_TOP_CENTER` (3) | below the resource bar, centred |

A plugin has at most 4 lines in all places together (`W2R_E_LIMIT` for a fifth); lines of different
plugins at one place are stacked in the order they were first set. Nothing is drawn outside a match
or while a recording plays, and a disabled plugin's lines are removed. Lua plugins use the same
store through `w2r.hud.text` (see the mod loader's `lua-plugins.md`).

## Units and orders

`units_query` and `order_issue` are appended to the host table (check `struct_size >=
W2R_HOST_UNITS_SIZE` and `W2R_HOST_ORDER_SIZE`). `game.control` is a new capability, shown in the
install prompt as giving orders to your units as if you had clicked, because a plugin that holds it
can automate play.

**`units_query(token, query, out, cap, total)`** (needs `game.read`) fills `out[0..cap]` with units
matching `W2rUnitQuery` and stores in `*total` how many matched (at most 256 are ever returned).
`owners` is a bit mask of players (bit 15 is neutral: gold mines, critters); 0 means any. `type` is
the game's unit type id or `W2R_NONE`. `flags_all` and `flags_none` select by `W2R_UNIT_BUILDING`,
`_WORKER`, `_SELECTED` (in the local player's selection), `_FINISHED` (a completed building) and
`_IDLE` (a unit whose order is stop or stand ground). Each `W2rUnit` has the slot `id`, the creation
`serial`, type, owner, current order, position in tiles, hit points and the mana byte (a building
under construction keeps its progress there).

- **What is listed.** Only units the local player may know about: its own, and those the game does
  not hide from it (the fog), so a plugin sees what the minimap would. When the local player only
  watches, or a recording plays, everything is listed. `W2R_E_STATE` outside a match.
- **Naming a unit.** Slots are reused at once when a unit dies, so `(id, serial)` together name
  one unit for ever; keep both.

**`order_issue(token, request)`** (needs `game.control`) gives an order to up to 12 of the local
player's own living units, exactly as if the player had selected them and clicked: it goes through
the game's command queue, so every machine in a networked game and the replay see it. The player's
selection is put back afterwards. The orders are `W2R_ORDER_STOP` (2), `_MOVE` (3), `_PATROL` (5),
`_FOLLOW` (6), `_ATTACK` (8; with a target unit attacks it, else attack-move to the tile), `_STAND`
(13, stand ground), `_HARVEST` (23), `_RETURN` (24) and `_REPAIR` (27). Follow, harvest and repair
need a target unit. Spells, building and producing are not available.

- `W2R_E_INVALID`: an order not in the list; a unit that is not named correctly (wrong serial), not
  alive, not the local player's, or named twice; a target that is not seen; no units or more than 12.
- `W2R_E_DENIED`: `game.control` was not approved, or the lobby's host switched plugin orders off
  (lobby capability "Plugin-given orders", `plugin_orders`).
- `W2R_E_STATE`: not in a match, or the local player only watches, or a recording plays.
- `W2R_E_RATE`: more than 8 orders in a second from this plugin; the call had no effect.

A plugin that gives orders changes what its player does, so the lobby host may want to forbid it
for fair games. The host's cog popup on the lobby screen switches it off for everyone in the lobby,
like the mod's other player aids. Orders given in a recording playback are refused.

## HTTP

`http_request(token, request)` (appended; check `struct_size >= W2R_HOST_HTTP_SIZE`; needs the
`net.http` capability) lets a plugin talk to a web service. It exists so that plugins can fetch
data, post results or reach their own server, and it is deliberately narrow:

- **Only the servers the manifest names.** `plugin.json` lists them in `net_hosts` (see
  manifest.md). The install prompt shows them ("Connects to"), so the player knows where the plugin
  can send anything, and a plugin that wants another server needs a new version and a new approval.
  The URL's host must be written exactly as one of those names; an IP address, `localhost`, a
  subdomain that is not listed, a user name (`user@host`) or a fragment is refused.
- **https on port 443 only**, no redirects (a redirect could lead to a server the player did not
  approve; a redirected request reports `W2R_HTTP_FAILED`), no cookies, no headers except a POST's
  content type (printable ASCII, at most 64 bytes; empty means `application/json`).
- **Sizes and rates.** URL at most 512 bytes, request body at most 16 KiB, answer at most 256 KiB
  (`W2R_HTTP_TOO_LARGE` past that). At most 4 requests in flight per plugin (`W2R_E_LIMIT`) and
  8 in all, and 30 a minute per plugin (`W2R_E_RATE`).
- **Asynchronous.** The call returns at once. The answer is delivered on the main thread as
  `W2R_EVT_HTTP_RESPONSE` with the `id` the plugin chose, without subscribing: `status` is the HTTP
  status (any status, including 404 and 500, is an answer) or 0 with `error` set when there was
  none. The body is valid only during the callback, so copy it.
- `W2R_E_INVALID` is returned for a bad URL, a host not in `net_hosts`, a bad method, a body on a
  GET, an oversized body, or an `id` that is already in flight.

The host never adds anything about the player to a request. What a plugin sends is up to the
plugin, so install only plugins you trust with the servers they name.
