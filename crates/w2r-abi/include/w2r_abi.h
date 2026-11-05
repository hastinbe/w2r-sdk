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
    W2R_EVT_COMMAND = 13,           /* payload W2rEvtCommand; sent to the plugin that registered the command */
    W2R_EVT_HTTP_RESPONSE = 14,      /* payload W2rEvtHttp; sent to the plugin that made the request */
    W2R_EVT_HOTKEY_ACTION = 15, /* payload W2rEvtHotkeyAction; only while a hotkey manager is active */
    W2R_EVT_KEY_CAPTURED = 16   /* payload W2rEvtKey; the key the manager asked for (key_capture) */
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

/* ---- hotkeys (docs/abi.md, "Hotkey service") --------------------------------------------------- */
/* A key: the SDL scancode in the low 16 bits and the modifiers that must be held, exactly. */
typedef uint32_t W2rKey;
#define W2R_KEY_SHIFT (1u << 16)
#define W2R_KEY_CTRL (1u << 17)
#define W2R_KEY_ALT (1u << 18)
#define W2R_KEY_CODE_MASK 0xFFFFu
#define W2R_KEY_A 4u
#define W2R_KEY_P 19u
#define W2R_KEY_GRAVE 53u
#define W2R_KEY_F1 58u

#define W2R_HOTKEY_PASS 1u      /* the game sees the key too */
#define W2R_HOTKEY_SAFE_CAST 2u /* not while a command waits for its target, nor just after one */
#define W2R_HOTKEY_POLL 4u      /* the plugin polls the key itself; the service only keeps the binding */
#define W2R_HOTKEY_FIXED 8u     /* listed, not rebindable: set elsewhere (a setting) or fixed by the game; hotkey_bind refuses it */

typedef struct W2rHotkeyDef {
    uint32_t struct_size; /* W2R_HOTKEY_DEF_V1_SIZE */
    uint32_t flags;
    W2rKey default_key;   /* 0 for none */
    W2rKey default_alt;
    W2rStr id;            /* unique within the plugin, [a-z0-9_.], 1-64 bytes */
    W2rStr label;         /* 1-64 bytes, Title Case */
    W2rStr category;      /* empty: the plugin's name */
} W2rHotkeyDef;
#define W2R_HOTKEY_DEF_V1_SIZE ((uint32_t)sizeof(W2rHotkeyDef))

typedef struct W2rEvtHotkeyAction {
    uint32_t struct_size;
    W2rHandle action; /* from hotkey_offer */
    uint32_t down;    /* 1 down, 0 up; key repeat is not delivered */
} W2rEvtHotkeyAction;

typedef struct W2rEvtKey {
    uint32_t struct_size;
    W2rKey key;
} W2rEvtKey;

/* Strings point into the host's copy, valid until the next hotkey_info call. */
typedef struct W2rHotkeyInfo {
    uint32_t struct_size;
    uint32_t flags;
    W2rKey key, alt, default_key, default_alt;
    W2rStr plugin, plugin_name, id, label, category;
} W2rHotkeyInfo;
#define W2R_HOTKEY_INFO_V1_SIZE ((uint32_t)sizeof(W2rHotkeyInfo))

#define W2R_REMAP_SAFE_CAST 1u /* kept from the game while a command waits for its target */
#define W2R_MAX_REMAPS 512u
#define W2R_SCREEN_CLOSE 2     /* screen_open: close the Options screen */
#define W2R_SCREEN_QUERY 3     /* screen_open: W2R_OK while Options is up */
#define W2R_SCREEN_GRAPHICS 4  /* screen_open: switch Classic / HD (as F5) */
#define W2R_SCREEN_ART_CACHE_ON 5  /* screen_open: keep the page art on disk for faster launches */
#define W2R_SCREEN_ART_CACHE_OFF 6 /* screen_open: stop, and remove the cached art */
#define W2R_SCREEN_OPTIONS 0x100 /* screen_open: Options on its first category */
#define W2R_SCREEN_SOUND 0x101
#define W2R_SCREEN_SPEED 0x102
#define W2R_SCREEN_PREFERENCES 0x103
#define W2R_SCREEN_GAMEPLAY 0x104
#define W2R_SCREEN_MULTIPLAYER 0x105
#define W2R_MAX_HOTKEYS 64u

typedef struct W2rRemap {
    W2rKey from;
    W2rKey to; /* 0 keeps `from` from the game */
    uint32_t flags;
    uint32_t card; /* only while this command card (button set) is up; W2R_ANY_CARD always */
} W2rRemap;
#define W2R_ANY_CARD 0xFFFFFFFFu

/* ---- plugin screen (screen_set; only valid inside its callback) ------------------------- */
typedef struct W2rRect {
    float x, y, w, h;
} W2rRect;

#define W2R_FRAME_WINDOW 0u
#define W2R_FRAME_PANEL 1u
#define W2R_FRAME_BANNER 2u /* the game's title scroll at the top (rect ignored) */
#define W2R_FRAME_CHECK 3u    /* the game's checkbox, off (with W2R_FRAME_ORC for the Orc art) */
#define W2R_FRAME_CHECK_ON 4u /* the same, ticked */
#define W2R_FRAME_SCROLL 5u       /* the game's scroll bar track (with W2R_FRAME_ORC for the Orc art) */
#define W2R_FRAME_SCROLL_THUMB 6u /* its thumb; W2R_FRAME_ACTIVE while dragged */
#define W2R_FRAME_TITLE 7u        /* a title band: dark strip, gold rule along the bottom (none in Classic) */
#define W2R_FRAME_ACTIVE 0x200u
#define W2R_SCREEN_CLASSIC 2u /* W2rScreen.icons bit: Classic graphics */
#define W2R_FRAME_ORC 0x100u
#define W2R_BUTTON_GREYED 1u
#define W2R_BUTTON_ORC 2u /* button flag: the Orc Game Menu button */
#define W2R_ICON_SELECTED 1u
#define W2R_ICON_ORC 2u /* icon flag: the icon is on the Orc side (the menus use one team colour for both) */
#define W2R_ALIGN_LEFT 0x11u
#define W2R_ALIGN_CENTRE 0x12u
#define W2R_ALIGN_RIGHT 0x14u
#define W2R_TEXT_TITLE 0x100u /* label flag: the menus' own font, for headings */
#define W2R_MOUSE_LEFT_DOWN 1u
#define W2R_MOUSE_LEFT_CLICK 2u
#define W2R_MOUSE_WHEEL_UP 4u   /* the wheel turned this frame, up */
#define W2R_MOUSE_WHEEL_DOWN 8u /* the wheel turned this frame, down */

typedef struct W2rScreen {
    uint32_t struct_size;
    void *ctx;
    float width, height;
    float mouse_x, mouse_y;
    uint32_t mouse;
    void (W2R_CALL *fill)(void *ctx, const W2rRect *r, uint32_t colour, float round);
    void (W2R_CALL *frame)(void *ctx, const W2rRect *r, uint32_t style);
    void (W2R_CALL *label)(void *ctx, const W2rRect *r, W2rStr text, uint32_t align, uint32_t colour);
    void (W2R_CALL *wrapped)(void *ctx, const W2rRect *r, W2rStr text, uint32_t colour);
    int32_t (W2R_CALL *button)(void *ctx, const W2rRect *r, W2rStr text, uint32_t flags);
    int32_t (W2R_CALL *icon)(void *ctx, const W2rRect *r, uint32_t icon, uint32_t flags);
    int32_t (W2R_CALL *hit)(void *ctx, const W2rRect *r);
    int32_t (W2R_CALL *hover)(void *ctx, const W2rRect *r);
    int32_t (W2R_CALL *list_begin)(void *ctx, const W2rRect *r, W2rStr name, float content_height);
    void (W2R_CALL *list_end)(void *ctx);
    uint32_t icons; /* 1 when command icon art is loaded (in a match) */
    /* V2: the width text takes on the screen, drawn as label with flags (W2R_TEXT_TITLE: headings). */
    float (W2R_CALL *measure)(void *ctx, W2rStr text, uint32_t flags);
} W2rScreen;
#define W2R_SCREEN_V1_SIZE ((uint32_t)offsetof(W2rScreen, measure))
#define W2R_SCREEN_V2_SIZE ((uint32_t)sizeof(W2rScreen))

/* ---- declared settings (settings_declare; docs/abi.md, "Declared settings") ------------- */
enum {
    W2R_SETTING_BOOL = 0,   /* a checkbox; settings_set_i64 with 0 or 1 */
    W2R_SETTING_INT = 1,    /* settings_set_i64; a slider when min < max, read-only otherwise */
    W2R_SETTING_CHOICE = 2, /* a dropdown over choices; settings_set_i64 with the index */
    W2R_SETTING_STRING = 3, /* a text box; settings_set_str */
    W2R_SETTING_COLOR = 4   /* a color picker; settings_set_str with six hex digits, "rrggbb" */
};
#define W2R_SETTING_HOTKEY 4u /* flags: not shown while a hotkey manager is active */
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

/* ---- units and orders (units_query, order_issue; docs/abi.md, "Units and orders") ----------- */
enum {
    W2R_UNIT_BUILDING = 1u << 0,
    W2R_UNIT_WORKER = 1u << 1,
    W2R_UNIT_SELECTED = 1u << 2,  /* in the local player's current selection */
    W2R_UNIT_FINISHED = 1u << 3,  /* a building that is completed (not under construction) */
    W2R_UNIT_IDLE = 1u << 4       /* its order is stop or stand and it is not carrying one out */
};
typedef struct W2rUnit {
    uint32_t id;       /* index in the game's unit array; slots are reused, so keep the serial too */
    uint32_t serial;   /* creation serial: (id, serial) names one unit for ever */
    uint32_t type;     /* the game's unit type id */
    uint32_t owner;    /* player index 0 to 7; 15 is neutral (mines, critters) */
    uint32_t order;    /* current order id (W2R_ORDER_*, others as the game numbers them) */
    uint32_t flags;    /* W2R_UNIT_* */
    int32_t tile_x;
    int32_t tile_y;
    uint32_t hp;       /* hit points */
    uint32_t mana;     /* the game's mana byte; for a building under construction its progress, 0 to 255 */
} W2rUnit;
typedef struct W2rUnitQuery {
    uint32_t struct_size;  /* sizeof(W2rUnitQuery) */
    uint32_t owners;       /* bit n selects player n, bit 15 neutral; 0 means any owner */
    uint32_t type;         /* a unit type id, or W2R_NONE for any */
    uint32_t flags_all;    /* the unit must have all of these W2R_UNIT_* flags */
    uint32_t flags_none;   /* and none of these */
} W2rUnitQuery;
enum {
    W2R_ORDER_STOP = 2,
    W2R_ORDER_MOVE = 3,
    W2R_ORDER_PATROL = 5,
    W2R_ORDER_FOLLOW = 6,
    W2R_ORDER_ATTACK = 8,       /* with a target unit: attack it; else attack-move to the tile */
    W2R_ORDER_STAND = 13,       /* stand ground */
    W2R_ORDER_HARVEST = 23,
    W2R_ORDER_RETURN = 24,
    W2R_ORDER_REPAIR = 27
};
#define W2R_MAX_ORDER_UNITS 12u  /* the game's selection limit */
typedef struct W2rUnitRef {
    uint32_t id;
    uint32_t serial;
} W2rUnitRef;
typedef struct W2rOrderRequest {
    uint32_t struct_size;    /* sizeof(W2rOrderRequest) */
    uint32_t order;          /* W2R_ORDER_* */
    int32_t tile_x;          /* where, when there is no target unit */
    int32_t tile_y;
    W2rUnitRef target;       /* id == W2R_NONE: no target unit */
    uint32_t unit_count;     /* 1 to W2R_MAX_ORDER_UNITS */
    W2rUnitRef units[W2R_MAX_ORDER_UNITS];
} W2rOrderRequest;

/* ---- HTTP (http_request; docs/abi.md, "HTTP") ------------------------------------------------ */
enum { W2R_HTTP_GET = 0, W2R_HTTP_POST = 1 };
enum {
    W2R_HTTP_OK = 0,           /* W2rEvtHttp.error: an answer came back; see `status` */
    W2R_HTTP_FAILED = 1,       /* no answer: could not connect, timed out, redirected or aborted */
    W2R_HTTP_TOO_LARGE = 2     /* the answer was longer than W2R_MAX_HTTP_RESPONSE */
};
#define W2R_MAX_HTTP_URL 512u
#define W2R_MAX_HTTP_BODY 16384u
#define W2R_MAX_HTTP_RESPONSE 262144u
#define W2R_MAX_HTTP_IN_FLIGHT 4u
typedef struct W2rHttpRequest {
    uint32_t struct_size;   /* sizeof(W2rHttpRequest) */
    uint32_t id;            /* chosen by the plugin; comes back in the answer */
    uint32_t method;        /* W2R_HTTP_GET or W2R_HTTP_POST */
    W2rStr url;             /* https://host/path?query, host from the manifest's net_hosts */
    W2rStr content_type;    /* POST only; printable ASCII, at most 64 bytes; empty means application/json */
    W2rBuf body;            /* POST only; at most W2R_MAX_HTTP_BODY bytes */
} W2rHttpRequest;
typedef struct W2rEvtHttp {
    uint32_t struct_size;
    uint32_t id;       /* the id of the request */
    uint32_t status;   /* the HTTP status, or 0 when error is not W2R_HTTP_OK */
    uint32_t error;    /* W2R_HTTP_* */
    W2rBuf body;       /* the answer's body; valid only during the callback */
} W2rEvtHttp;

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

    /* Appended after hud_text_set: check host->struct_size >= W2R_HOST_UNITS_SIZE.
       Lists units into `out[0..cap]` and stores the number that matched in `*total` (which may
       be more than `cap`, at most 256 are ever returned). Requires `game.read`. Only units the local
       player may know about are listed: its own, and those the game shows it (not hidden by the fog),
       or everything when the local player only watches or a recording plays. W2R_E_STATE outside a
       match.
       */
    W2rStatus (W2R_CALL *units_query)(void *token, const W2rUnitQuery *query, W2rUnit *out,
                                       uint32_t cap, uint32_t *total);

    /* Appended after units_query: check host->struct_size >= W2R_HOST_ORDER_SIZE.
       Gives an order to up to 12 of the local player's own units, exactly as if the player had
       selected them and clicked: it goes through the game's command queue, so every machine and the
       replay see it. The player's selection is put back afterwards. Requires `game.control`.
       W2R_E_INVALID: a unit that is not named correctly, not yours or not alive, or an order not in
       W2R_ORDER_*; W2R_E_DENIED: no permission, or the lobby's host switched plugin orders off;
       W2R_E_STATE: not in a match, only watching, or a recording is playing; W2R_E_RATE: more than
       8 orders in a second from this plugin.
       */
    W2rStatus (W2R_CALL *order_issue)(void *token, const W2rOrderRequest *request);

    /* Appended after order_issue: check host->struct_size >= W2R_HOST_HTTP_SIZE.
       Sends an HTTPS request to one of the servers the plugin's manifest names in `net_hosts`
       (and the user approved when installing it). The answer comes later as W2R_EVT_HTTP_RESPONSE
       with the same `id`, without subscribing. Requires `net.http`. Only https on port 443, no
       redirects, no cookies, no other headers. W2R_E_INVALID: a bad URL, a host not in net_hosts,
       an IP address or a bad size or method; W2R_E_LIMIT: 4 requests already in flight;
       W2R_E_RATE: more than 30 requests in a minute; W2R_E_STATE: not allowed in on_load.
       */
    W2rStatus (W2R_CALL *http_request)(void *token, const W2rHttpRequest *request);

    /* Hotkeys, appended after http_request: check host->struct_size >= W2R_HOST_HOTKEY_SERVICE_SIZE.
       docs/abi.md, "Hotkey service". */
    /* Implicit, only inside on_load: offers one action to the hotkey manager. */
    W2rStatus (W2R_CALL *hotkey_offer)(void *token, const W2rHotkeyDef *def, W2rHandle *out);
    /* Implicit: 1 while a hotkey manager plugin is active, else 0. */
    int32_t (W2R_CALL *hotkeys_managed)(void *token);
    /* hooks.raw, only inside on_load: become the hotkey manager (one per session). */
    W2rStatus (W2R_CALL *hotkey_manage)(void *token);
    /* The manager only. */
    W2rStatus (W2R_CALL *hotkey_count)(void *token, uint32_t *count_out);
    W2rStatus (W2R_CALL *hotkey_info)(void *token, uint32_t index, W2rHotkeyInfo *out);
    W2rStatus (W2R_CALL *hotkey_bind)(void *token, uint32_t index, W2rKey key, W2rKey alt);
    W2rStatus (W2R_CALL *key_remap_set)(void *token, const W2rRemap *entries, uint32_t count);
    W2rStatus (W2R_CALL *key_capture)(void *token, int32_t on);
    W2rStatus (W2R_CALL *screen_set)(void *token, void (W2R_CALL *callback)(const W2rScreen *screen));
    /* 1: open Options on the manager's page (a match only); 0: leave the page. */
    W2rStatus (W2R_CALL *screen_open)(void *token, int32_t open);
    /* Appended after screen_open (struct_size >= W2R_HOST_HOTKEY_GET_SIZE): the key and alternate one of the
       caller's own actions is bound to now; for W2R_HOTKEY_POLL actions the plugin polls itself. */
    W2rStatus (W2R_CALL *hotkey_get)(void *token, W2rHandle action, W2rKey *key_out, W2rKey *alt_out);
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
#define W2R_HOST_UNITS_SIZE (offsetof(W2rHost, units_query) + sizeof(void *))
#define W2R_HOST_ORDER_SIZE (offsetof(W2rHost, order_issue) + sizeof(void *))
#define W2R_HOST_HTTP_SIZE (offsetof(W2rHost, http_request) + sizeof(void *))
#define W2R_HOST_HOTKEY_SERVICE_SIZE (offsetof(W2rHost, screen_open) + sizeof(void *))
#define W2R_HOST_HOTKEY_GET_SIZE ((uint32_t)sizeof(W2rHost))

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
