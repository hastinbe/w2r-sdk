/*
 * w2rpm plugin ABI, version 1. ADD-ONLY: fields are only ever appended to the end of a struct,
 * enum values are never renumbered, nothing is removed. See docs/abi.md for limits and rules.
 *
 * Threading: every host function and every plugin callback runs on the game's main thread.
 * Capabilities are a consent prompt and a guardrail, not a sandbox: a native plugin runs in the
 * game process and can do anything that process can.
 */
#ifndef W2R_ABI_H
#define W2R_ABI_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define W2R_ABI_VERSION 1u

#if defined(_WIN32)
#define W2R_CALL __cdecl
#define W2R_EXPORT __declspec(dllexport)
#else
#define W2R_CALL
#define W2R_EXPORT __attribute__((visibility("default")))
#endif

/* ---- status codes ---------------------------------------------------------------------- */
typedef int32_t W2rStatus;
enum {
    W2R_OK = 0,
    W2R_E_DENIED = -1,      /* capability not approved */
    W2R_E_INVALID = -2,     /* bad argument, or access to game memory failed */
    W2R_E_LIMIT = -3,       /* a size or count limit was exceeded */
    W2R_E_STATE = -4,       /* not allowed right now (wrong thread, not in a lobby, ...) */
    W2R_E_UNSUPPORTED = -5, /* host does not provide this (older host) */
    W2R_E_NOT_FOUND = -6,
    W2R_E_RATE = -7,        /* rate limit hit; the call had no effect */
    W2R_E_CONFLICT = -8,    /* hook overlaps an existing patch */
    W2R_E_BUSY = -9         /* not ready yet, ask again on a later frame */
};

/* ---- value types ----------------------------------------------------------------------- */
/* UTF-8, NOT NUL-terminated. */
typedef struct W2rStr {
    const char *ptr;
    uint32_t len;
} W2rStr;

typedef struct W2rBuf {
    const uint8_t *ptr;
    uint32_t len;
} W2rBuf;

typedef uint32_t W2rHandle; /* host-issued, 0 is never valid */

enum { W2R_LOG_DEBUG = 0, W2R_LOG_INFO = 1, W2R_LOG_WARN = 2, W2R_LOG_ERROR = 3 };
enum { W2R_FILE_READ = 1, W2R_FILE_WRITE = 2 /* create or truncate */ };
#define W2R_SLOT_ALL 0xFFFFFFFFu /* peer_send: every peer that announced the handshake */

/* ---- limits (also in docs/abi.md) ------------------------------------------------------ */
#define W2R_MAX_LOG_BYTES 512u
#define W2R_MAX_SETTING_KEY 64u
#define W2R_MAX_SETTING_STR 1024u
#define W2R_MAX_FILE_NAME 64u
#define W2R_MAX_IO_BYTES 65536u
#define W2R_MAX_SYMBOL_NAME 64u
#define W2R_MAX_READ_BYTES 4096u
#define W2R_MAX_CHAT_BYTES 200u
#define W2R_PEER_HEADER_BYTES 9u   /* 0xE7 'W' '2' 'R' 'P' tag[4] */
#define W2R_MAX_PEER_PAYLOAD 960u
#define W2R_MAX_SETTING_LABEL 64u
#define W2R_MAX_SETTING_DESCRIPTION 256u
#define W2R_MAX_SETTING_CATEGORY 32u
#define W2R_MAX_SETTING_CHOICE 64u      /* chars in one dropdown choice */
#define W2R_MAX_SETTING_CHOICE_COUNT 16u     /* number of choices in one dropdown */

/* ---- events ---------------------------------------------------------------------------- */
typedef uint32_t W2rEventId;
enum {
    W2R_EVT_LOBBY_ENTER = 1,
    W2R_EVT_LOBBY_LEAVE = 2,
    W2R_EVT_LOBBY_PLAYER_JOIN = 3,  /* payload W2rEvtPlayer; name empty without chat.read */
    W2R_EVT_LOBBY_PLAYER_LEAVE = 4, /* payload W2rEvtPlayer */
    W2R_EVT_CHAT_RECEIVED = 5,      /* payload W2rEvtChat; needs chat.read */
    W2R_EVT_PEER_MESSAGE = 6,       /* payload W2rEvtPeerMessage; needs net.send */
    W2R_EVT_MATCH_START = 7,
    W2R_EVT_MATCH_END = 8,          /* payload W2rEvtMatchEnd (added later; NULL from an older host) */
    W2R_EVT_SETTINGS_CHANGED = 9,
    W2R_EVT_SHUTDOWN = 10,
    W2R_EVT_GAME_MODE = 11,         /* payload W2rEvtGameMode; the lobby's game mode for the next match */
    W2R_EVT_HOTKEY = 12,            /* payload W2rEvtHotkey; sent to the plugin that registered the key */
    W2R_EVT_COMMAND = 13            /* payload W2rEvtCommand; sent to the plugin that registered the command */
};

typedef struct W2rEvent {
    uint32_t struct_size;
    W2rEventId id;
    const void *payload; /* NULL for events without one; valid only during the callback */
} W2rEvent;

typedef struct W2rEvtPlayer {
    uint32_t struct_size;
    uint32_t net_slot;
    W2rStr name; /* sanitised: printable ASCII only */
} W2rEvtPlayer;

typedef struct W2rEvtChat {
    uint32_t struct_size;
    uint32_t net_slot;
    W2rStr name;
    W2rStr text; /* sanitised: printable ASCII only */
} W2rEvtChat;

typedef struct W2rEvtPeerMessage {
    uint32_t struct_size;
    uint32_t net_slot; /* the slot the transport reported, never one named inside the message */
    W2rBuf data;       /* the payload after the 9-byte header */
} W2rEvtPeerMessage;

typedef struct W2rEvtHotkey {
    uint32_t struct_size;
    uint32_t id; /* the id given to hotkey_register */
} W2rEvtHotkey;

typedef struct W2rEvtCommand {
    uint32_t struct_size;
    uint32_t id;    /* the id given to command_register */
    W2rStr args;    /* the text after "/name ", trimmed; printable ASCII, at most 200 bytes */
} W2rEvtCommand;

enum {
    W2R_GAME_MODE_CLASSIC_BALANCE = 1u << 0,
    W2R_GAME_MODE_REPLAY = 1u << 1 /* the mode is for a recording being played (not a networked game) */
};
typedef struct W2rEvtGameMode {
    uint32_t struct_size;
    uint32_t flags; /* W2R_GAME_MODE_*: the modes in force for the next match, on every machine */
} W2rEvtGameMode;

/* W2R_EVT_MATCH_END: how the match ended for the local player, and the final tallies of everyone
   who played. A host that predates the payload sends NULL, so check it. Player names are empty
   without chat.read, like the lobby events. */
enum {
    W2R_PLAYER_HUMAN = 1u << 0,
    W2R_PLAYER_COMPUTER = 1u << 1,
    W2R_PLAYER_LOCAL = 1u << 2
};
enum {
    W2R_RESULT_NONE = 0,    /* no result was written: surrendered, left, or never decided */
    W2R_RESULT_DEFEAT = 2,
    W2R_RESULT_VICTORY = 3,
    W2R_RESULT_DRAW = 5     /* any other code is passed through unchanged */
};
enum { W2R_MATCH_DESYNC = 1u << 0 /* the game flagged at least one turn as out of sync */ };
typedef struct W2rMatchPlayer {
    uint32_t player;          /* the player's index in the match, 0 to 7 */
    uint32_t flags;           /* W2R_PLAYER_* */
    uint32_t result;          /* W2R_RESULT_* */
    uint32_t units_killed;
    uint32_t buildings_razed;
    uint32_t units_lost;
    uint32_t buildings_lost;
    uint32_t gold_mined;
    uint32_t lumber_mined;
    uint32_t oil_mined;
    W2rStr name;              /* sanitised: printable ASCII only; empty without chat.read */
} W2rMatchPlayer;
typedef struct W2rEvtMatchEnd {
    uint32_t struct_size;
    uint32_t local_player;    /* index of the local player */
    uint32_t flags;           /* W2R_MATCH_* */
    uint32_t turns;           /* game turns that ran */
    W2rStr map;               /* the lobby's map name; sanitised */
    uint32_t player_count;
    uint32_t player_size;     /* sizeof(W2rMatchPlayer) in the host: step by this, not sizeof */
    const W2rMatchPlayer *players;
} W2rEvtMatchEnd;

/* ---- overlay (overlay_set; docs/abi.md, "Overlay") ------------------------------------------ */
enum {
    W2R_OVERLAY_LINE = 0,    /* a dotted line from (x0, y0) to (x1, y1) */
    W2R_OVERLAY_MARKER = 1,  /* a filled circle with outline at (x0, y0), radius x1 (0: small dot) */
    W2R_OVERLAY_ARROWS = 2,  /* a solid line (x0, y0) to (x1, y1) with small arrows along it */
    W2R_OVERLAY_RECT = 4,    /* a rectangle, top-left (x0, y0), size x1 by y1; translucent fill, opaque outline */
    W2R_MAX_OVERLAY_ITEMS = 1024
};
/* Coordinates are world pixels (tile * 32; add 16 for a tile's centre); rgba is 0xRRGGBBAA. */
typedef struct W2rOverlayItem {
    uint32_t kind;
    int32_t x0, y0, x1, y1;
    uint32_t rgba;
} W2rOverlayItem;

/* ---- declared settings (settings_declare; docs/abi.md, "Declared settings") ------------- */
enum {
    W2R_SETTING_BOOL = 0,   /* a checkbox; settings_set_i64 with 0 or 1 */
    W2R_SETTING_INT = 1,    /* settings_set_i64; a slider when min < max, read-only otherwise */
    W2R_SETTING_CHOICE = 2, /* a dropdown over choices; settings_set_i64 with the index */
    W2R_SETTING_STRING = 3, /* a text box; settings_set_str */
    W2R_SETTING_COLOR = 4   /* a color picker; settings_set_str with six hex digits, "rrggbb" */
};
#define W2R_SETTING_HIDDEN 1u /* flags: stored and readable as usual, never shown */
#define W2R_SETTING_LIVE 2u   /* flags: the screen applies as soon as this setting changes */

/* Every string is copied during the call. Fields a kind does not use are ignored. The order keeps
   every field at the same offset under any 32-bit compiler (each 64-bit field 8-aligned, the size a
   multiple of 8). */
typedef struct W2rSettingDef {
    uint32_t struct_size;  /* W2R_SETTING_DEF_V1_SIZE */
    uint32_t kind;         /* W2R_SETTING_* */
    int64_t default_int;   /* bool 0/1, int value, choice index */
    int64_t min, max;      /* W2R_SETTING_INT only, inclusive; min == max draws no slider */
    uint32_t flags;        /* W2R_SETTING_HIDDEN, W2R_SETTING_LIVE or 0 */
    uint32_t choice_count;
    const W2rStr *choices; /* W2R_SETTING_CHOICE only: 2..W2R_MAX_SETTING_CHOICE_COUNT labels */
    uint32_t reserved;     /* must be 0 */
    W2rStr key;            /* the settings key, same rule as settings_set_* */
    W2rStr label;          /* the row's text, Title Case */
    W2rStr description;    /* hover text; may be empty */
    W2rStr category;       /* left-hand list entry, shared across plugins; empty = plugin name */
    W2rStr subcategory;    /* heading inside the category; empty = "General" */
    W2rStr default_str;    /* W2R_SETTING_STRING, or W2R_SETTING_COLOR as "rrggbb" */
    W2rStr active_when;    /* greyed while this BOOL is off: an earlier key of this plugin, or
                              "host:<key>" for one of the host's; empty = always active */
} W2rSettingDef;

#define W2R_SETTING_DEF_V1_SIZE ((uint32_t)sizeof(W2rSettingDef))

/* ---- UI (only valid inside on_ui) ------------------------------------------------------ */
typedef struct W2rUi {
    uint32_t struct_size;
    void *ctx;
    void (W2R_CALL *label)(void *ctx, W2rStr text);
    int32_t (W2R_CALL *button)(void *ctx, W2rStr text);           /* 1 when pressed */
    int32_t (W2R_CALL *checkbox)(void *ctx, W2rStr text, int32_t *value); /* 1 when changed */
    int32_t (W2R_CALL *slider_int)(void *ctx, W2rStr text, int32_t *value, int32_t min, int32_t max);
    /* Appended after the first release: check ui->struct_size covers these before using them. */
    int32_t (W2R_CALL *text_edit)(void *ctx, W2rStr text, char *buf, uint32_t cap); /* 1 when changed */
    void (W2R_CALL *set_active)(void *ctx, int32_t active); /* 0 greys out the rows after it */
    /* Appended after set_active: check ui->struct_size >= W2R_UI_COLOR_EDIT_SIZE. A color row:
       caption, swatch and a #RRGGBB button that opens the host's color editor. hex is the
       plugin's own 7-byte buffer, "rrggbb" and a NUL; 1 when the player pressed OK on a
       different color, with hex rewritten. */
    int32_t (W2R_CALL *color_edit)(void *ctx, W2rStr text, char *hex);
    /* Appended after color_edit: check ui->struct_size >= W2R_UI_COMBO_SIZE. A dropdown row:
       caption and `count` choices (2..W2R_MAX_SETTING_CHOICE_COUNT, each at most
       W2R_MAX_SETTING_CHOICE bytes) showing *selected; 1 when the player picked a different
       entry, with *selected set to it. */
    int32_t (W2R_CALL *combo)(void *ctx, W2rStr text, const W2rStr *choices, uint32_t count,
                              int32_t *selected);
} W2rUi;

#define W2R_UI_TEXT_EDIT_SIZE ((uint32_t)(offsetof(W2rUi, set_active) + sizeof(void *)))
#define W2R_UI_COLOR_EDIT_SIZE ((uint32_t)(offsetof(W2rUi, color_edit) + sizeof(void *)))
#define W2R_UI_COMBO_SIZE ((uint32_t)(offsetof(W2rUi, combo) + sizeof(void *)))
#define W2R_COLOR_HEX_BYTES 7u

/* ---- host table: provided by the host, one instance per plugin ------------------------- */
/* ---- game snapshot (game_snapshot; docs/abi.md, "Game snapshot") ----------------------------- */
enum {
    W2R_SNAPSHOT_IN_MATCH = 1u << 0,
    W2R_SNAPSHOT_PAUSED = 1u << 1,
    W2R_SNAPSHOT_REPLAY = 1u << 2,   /* a recording is being played back */
    W2R_SNAPSHOT_OBSERVER = 1u << 3  /* the local player is watching, not playing */
};
enum { W2R_PLAYER_ALLY = 1u << 3 /* on the local player's team (W2rPlayerInfo only) */ };
#define W2R_NONE 0xFFFFFFFFu /* "no value" in the unsigned fields of the snapshot */
#define W2R_MAX_PLAYERS 8u
typedef struct W2rPlayerInfo {
    uint32_t player;   /* index in the match, 0 to 7 */
    uint32_t flags;    /* W2R_PLAYER_HUMAN | COMPUTER | LOCAL | ALLY */
    uint32_t race;     /* the game's race byte: 0 human, 1 orc; W2R_NONE if unknown */
    uint32_t team;     /* the lobby team as the game stores it; W2R_NONE if unknown */
    uint32_t colour;   /* colour index 0 to 7; W2R_NONE before the game assigns one */
    uint32_t result;   /* W2R_RESULT_*: non-zero once the player's match is decided */
    int32_t gold;      /* -1 where the local player may not know: not yourself or an ally */
    int32_t lumber;
    int32_t oil;
    int32_t food_used; /* -1 likewise */
    int32_t food_cap;
    W2rStr name;       /* sanitised: printable ASCII only; empty without chat.read. Valid until the
                          next game_snapshot call. */
} W2rPlayerInfo;
typedef struct W2rGameSnapshot {
    uint32_t struct_size;    /* IN: sizeof(W2rGameSnapshot) as the plugin was built */
    uint32_t flags;          /* W2R_SNAPSHOT_* */
    uint32_t game_ms;        /* the match clock, pause-aware */
    uint32_t turn;           /* network turns run so far */
    uint32_t local_player;   /* the player whose view the screen shows; W2R_NONE if unknown */
    uint32_t player_count;   /* entries used in players[] */
    W2rPlayerInfo players[W2R_MAX_PLAYERS];
} W2rGameSnapshot;

/* ---- HUD text (hud_text_set; docs/abi.md, "HUD text") ----------------------------------------- */
enum {
    W2R_HUD_BOTTOM_RIGHT = 0, /* above the APM counter and the clock (the original place) */
    W2R_HUD_TOP_LEFT = 1,     /* below the resource bar */
    W2R_HUD_TOP_RIGHT = 2,
    W2R_HUD_TOP_CENTER = 3
};
#define W2R_MAX_HUD_LINES 4u   /* per plugin, in all places together */
#define W2R_MAX_HUD_KEY 32u
#define W2R_MAX_HUD_TEXT 64u

typedef struct W2rHost {
    uint32_t struct_size; /* check before using any field past the version you built against */
    uint32_t abi_version;
    void *token;          /* opaque; pass back as the first argument of every call */

    /* log (implicit) */
    W2rStatus (W2R_CALL *log)(void *token, uint32_t level, W2rStr msg);

    /* settings (implicit): the plugin's own settings only */
    W2rStatus (W2R_CALL *settings_get_i64)(void *token, W2rStr key, int64_t *out);
    W2rStatus (W2R_CALL *settings_set_i64)(void *token, W2rStr key, int64_t value);
    W2rStatus (W2R_CALL *settings_get_str)(void *token, W2rStr key, char *buf, uint32_t cap,
                                           uint32_t *len_out);
    W2rStatus (W2R_CALL *settings_set_str)(void *token, W2rStr key, W2rStr value);

    /* fs.data (implicit): flat files in the plugin's own data folder, by host handle */
    W2rStatus (W2R_CALL *data_open)(void *token, W2rStr name, uint32_t flags, W2rHandle *out);
    W2rStatus (W2R_CALL *data_read)(void *token, W2rHandle file, void *dst, uint32_t cap,
                                    uint32_t *read_out);
    W2rStatus (W2R_CALL *data_write)(void *token, W2rHandle file, const void *src, uint32_t len);
    W2rStatus (W2R_CALL *data_close)(void *token, W2rHandle file);

    /* events */
    W2rStatus (W2R_CALL *subscribe)(void *token, W2rEventId id);
    W2rStatus (W2R_CALL *unsubscribe)(void *token, W2rEventId id);

    /* game.read */
    W2rStatus (W2R_CALL *symbol_lookup)(void *token, W2rStr name, W2rHandle *out);
    W2rStatus (W2R_CALL *symbol_read)(void *token, W2rHandle symbol, uint32_t offset, void *dst,
                                      uint32_t len);

    /* chat.write */
    W2rStatus (W2R_CALL *chat_send)(void *token, W2rStr text);

    /* net.send */
    W2rStatus (W2R_CALL *peer_send)(void *token, uint32_t net_slot_or_all, W2rBuf payload);

    /* hooks.raw (dangerous). target: a symbol_lookup handle of a function symbol;
       original_out receives the trampoline before the hook goes live. */
    W2rStatus (W2R_CALL *hook_install)(void *token, uint32_t target, void *detour, void **original_out,
                                       W2rHandle *out);
    W2rStatus (W2R_CALL *hook_remove)(void *token, W2rHandle hook);

    /* hooks.raw (dangerous): the runtime address of any symbol (function or global), always inside
       the game module. For plugins that call game functions or write game memory themselves. */
    W2rStatus (W2R_CALL *symbol_address)(void *token, W2rHandle symbol, uintptr_t *address_out);

    /* Create-screen extension (implicit), appended after the first release: check
       host->struct_size >= W2R_HOST_CREATE_UI_SIZE before using these. */
    /* The callback is called every frame the enhanced Create Multiplayer Game screen draws, with a
       W2rUi whose rows appear under the map preview. NULL removes it. */
    W2rStatus (W2R_CALL *create_ui_register)(void *token, void (W2R_CALL *callback)(const W2rUi *ui));
    /* Only inside that callback. Counts the indexed maps with `players` maximum players (0 = any);
       W2R_E_BUSY while the host is still building its map index. */
    W2rStatus (W2R_CALL *create_map_count)(void *token, uint32_t players, uint32_t *count_out);
    /* Only inside that callback. Selects the nth (0-based) such map for the game being created. */
    W2rStatus (W2R_CALL *create_map_select)(void *token, uint32_t players, uint32_t nth);

    /* Appended after those: check host->struct_size >= W2R_HOST_MAP_FOLDER_SIZE. Only inside the
       create-screen callback. Limits create_map_count/create_map_select of this callback to the
       maps in `folder` (under the maps folder, subfolders included); empty removes the limit. The
       limit lasts until the callback returns, so call it first, every frame. */
    W2rStatus (W2R_CALL *create_map_folder)(void *token, W2rStr folder);

    /* Appended after create_map_folder: check host->struct_size >= W2R_HOST_SETTINGS_DECLARE_SIZE.
       Only inside on_load. Declares one of the plugin's own settings for the host's options
       screen, which edits it for the plugin; W2R_EVT_SETTINGS_CHANGED says when it did. A declared
       key that was never stored reads as its default. */
    W2rStatus (W2R_CALL *settings_declare)(void *token, const W2rSettingDef *def);

    /* Appended after settings_declare: check host->struct_size >= W2R_HOST_OVERLAY_SIZE.
       Replaces the plugin's overlay: `count` items (at most W2R_MAX_OVERLAY_ITEMS) the host draws
       on the map view during a match, in world pixels (tile * 32), until the next call. 0 clears. */
    W2rStatus (W2R_CALL *overlay_set)(void *token, const W2rOverlayItem *items, uint32_t count);

    /* Appended after overlay_set: check host->struct_size >= W2R_HOST_SET_CHOICES_SIZE.
       Replaces the entries of one of the plugin's declared choice settings: `count` labels (2 to
       16, 1 to 64 bytes each), copied during the call. W2R_E_NOT_FOUND for another key. */
    W2rStatus (W2R_CALL *settings_set_choices)(void *token, W2rStr key, const W2rStr *choices,
                                               uint32_t count);

    /* Appended after settings_set_choices: check host->struct_size >= W2R_HOST_SNAPSHOT_SIZE.
       Fills `*out` with the match as the local player may know it: the clock, the pause state and the
       players with their race, team, colour, result and resources. Requires `game.read`. Another
       player's resources and food are -1 unless that player is on the local player's team, the
       local player is only watching (observer), or a recording is being played, so a plugin cannot
       see what the game itself hides. W2R_E_STATE outside a match.
       */
    W2rStatus (W2R_CALL *game_snapshot)(void *token, W2rGameSnapshot *out);

    /* Appended after game_snapshot: check host->struct_size >= W2R_HOST_HOTKEY_SIZE.
       Asks for a key combination such as "Ctrl+Alt+K", "Alt+F5" or "Ctrl+Shift+Insert". When it is
       pressed (exactly those modifiers, the game window focused) the plugin gets W2R_EVT_HOTKEY with
       `id`, without subscribing. The combination must include Ctrl or Alt or be an F key.
       W2R_E_INVALID: not a combination or not allowed; W2R_E_CONFLICT: the host, the game or
       another plugin holds it; W2R_E_LIMIT: more than 8. No capability is needed.
       */
    W2rStatus (W2R_CALL *hotkey_register)(void *token, W2rStr combo, uint32_t id);

    /* Appended after hotkey_register: check host->struct_size >= W2R_HOST_COMMAND_SIZE.
       Asks for the chat command `/name` (1 to 16 of a-z, 0-9 and '-', starting with a letter). When
       the player types it in the lobby or in a match, the line is taken from the chat box (not
       sent) and the plugin gets W2R_EVT_COMMAND with `id` and the text after the name, without
       subscribing. W2R_E_INVALID: not a valid name; W2R_E_CONFLICT: the host, the game or another
       plugin uses it; W2R_E_LIMIT: more than 8. No capability is needed.
       */
    W2rStatus (W2R_CALL *command_register)(void *token, W2rStr name, uint32_t id);

    /* Appended after command_register: check host->struct_size >= W2R_HOST_HUD_TEXT_SIZE.
       Sets (or, with empty `text`, removes) the plugin's HUD line `key` (1 to 32 bytes) in the
       in-match HUD: one line of printable ASCII, at most 64 bytes, at `place` (W2R_HUD_*). A plugin
       has at most 4 lines in all places together. Nothing is drawn outside a match or while a
       recording plays. W2R_E_INVALID: bad key or text or place; W2R_E_LIMIT: a fifth line.
       No capability is needed.
       */
    W2rStatus (W2R_CALL *hud_text_set)(void *token, W2rStr key, W2rStr text, uint32_t place);
} W2rHost;

#define W2R_HOST_CREATE_UI_SIZE (offsetof(W2rHost, create_map_select) + sizeof(void *))
#define W2R_HOST_MAP_FOLDER_SIZE (offsetof(W2rHost, create_map_folder) + sizeof(void *))
#define W2R_HOST_SETTINGS_DECLARE_SIZE (offsetof(W2rHost, settings_declare) + sizeof(void *))
#define W2R_HOST_OVERLAY_SIZE (offsetof(W2rHost, overlay_set) + sizeof(void *))
#define W2R_HOST_SET_CHOICES_SIZE (offsetof(W2rHost, settings_set_choices) + sizeof(void *))
#define W2R_HOST_SNAPSHOT_SIZE (offsetof(W2rHost, game_snapshot) + sizeof(void *))
#define W2R_HOST_HOTKEY_SIZE (offsetof(W2rHost, hotkey_register) + sizeof(void *))
#define W2R_HOST_COMMAND_SIZE (offsetof(W2rHost, command_register) + sizeof(void *))
#define W2R_HOST_HUD_TEXT_SIZE (offsetof(W2rHost, hud_text_set) + sizeof(void *))

/* ---- plugin table: returned by the plugin ---------------------------------------------- */
typedef struct W2rPlugin {
    uint32_t struct_size;
    W2rStatus (W2R_CALL *on_load)(const W2rHost *host);   /* non-zero fails the load (E_LOAD) */
    void (W2R_CALL *on_event)(const W2rEvent *event);     /* may be NULL */
    void (W2R_CALL *on_ui)(const W2rUi *ui);              /* may be NULL */
    void (W2R_CALL *on_unload)(void);                     /* may be NULL; best effort */
} W2rPlugin;

/* The single export every plugin DLL provides. */
typedef const W2rPlugin *(W2R_CALL *W2rPluginMain)(const W2rHost *host);
W2R_EXPORT const W2rPlugin *W2R_CALL w2r_plugin_main(const W2rHost *host);

#ifdef __cplusplus
}
#endif

#endif /* W2R_ABI_H */
