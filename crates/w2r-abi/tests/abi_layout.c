/* Compiled as C11, C++17 and (if available) 32-bit by tools/check_abi.sh. Never run. */
#include <stddef.h>
#include "w2r_abi.h"

#ifdef __cplusplus
#define CHECK(c) static_assert(c, #c)
#else
#define CHECK(c) _Static_assert(c, #c)
#endif

CHECK(W2R_ABI_VERSION == 1);
CHECK(offsetof(W2rHost, struct_size) == 0);
CHECK(offsetof(W2rHost, abi_version) == 4);
CHECK(offsetof(W2rHost, token) == 8);
CHECK(offsetof(W2rPlugin, struct_size) == 0);
CHECK(offsetof(W2rUi, struct_size) == 0);
CHECK(W2R_UI_COMBO_SIZE == sizeof(W2rUi));
CHECK(W2R_UI_COMBO_SIZE > W2R_UI_COLOR_EDIT_SIZE);
CHECK(W2R_UI_COLOR_EDIT_SIZE > W2R_UI_TEXT_EDIT_SIZE);
CHECK(W2R_UI_TEXT_EDIT_SIZE == (sizeof(void *) == 4 ? 32 : 64));
CHECK(offsetof(W2rEvent, struct_size) == 0);
CHECK(offsetof(W2rEvtGameMode, flags) == 4);
CHECK(W2R_EVT_GAME_MODE == 11);
CHECK(W2R_EVT_HOTKEY == 12 && W2R_EVT_COMMAND == 13);
CHECK(W2R_EVT_HTTP_RESPONSE == 14);
CHECK(W2R_EVT_HOTKEY_ACTION == 15 && W2R_EVT_KEY_CAPTURED == 16);
CHECK(W2R_HOST_HOTKEY_GET_SIZE == sizeof(W2rHost));
CHECK(W2R_HOST_HOTKEY_SERVICE_SIZE == offsetof(W2rHost, hotkey_get));
CHECK(W2R_HOST_HOTKEY_SERVICE_SIZE > W2R_HOST_HTTP_SIZE);
CHECK(sizeof(W2rHotkeyDef) == (sizeof(void *) == 4 ? 40 : 64));
CHECK(sizeof(W2rRemap) == 16);
CHECK(sizeof(W2rScreen) == (sizeof(void *) == 4 ? 76 : 136));
CHECK(W2R_SCREEN_V1_SIZE == (sizeof(void *) == 4 ? 72 : 128));
CHECK(offsetof(W2rEvtMatchEnd, players) == 16 + sizeof(W2rStr) + 8);
CHECK(sizeof(W2rMatchPlayer) == 40 + sizeof(W2rStr));
CHECK(W2R_RESULT_VICTORY == 3);
CHECK(sizeof(W2rUnit) == 40);
CHECK(sizeof(W2rUnitRef) == 8);
CHECK(offsetof(W2rOrderRequest, units) == 28);
CHECK(sizeof(W2rOrderRequest) == 28 + 12 * 8);
CHECK(offsetof(W2rGameSnapshot, players) == 24);
CHECK(W2R_MAX_PLAYERS == 8);
CHECK(offsetof(W2rStr, len) == sizeof(void *));
CHECK(sizeof(W2rStatus) == 4);
CHECK(W2R_OK == 0);
CHECK(W2R_E_DENIED < 0 && W2R_E_BUSY < 0);
CHECK(W2R_HOST_CREATE_UI_SIZE > offsetof(W2rHost, symbol_address));
CHECK(W2R_HOST_MAP_FOLDER_SIZE > offsetof(W2rHost, create_map_select));
CHECK(W2R_HOST_HTTP_SIZE == offsetof(W2rHost, http_request) + sizeof(void *));
CHECK(W2R_HOST_HTTP_SIZE > W2R_HOST_ORDER_SIZE);
CHECK(W2R_HOST_UNITS_SIZE > W2R_HOST_HUD_TEXT_SIZE);
CHECK(W2R_HOST_ORDER_SIZE > W2R_HOST_UNITS_SIZE);
CHECK(W2R_HOST_HUD_TEXT_SIZE > W2R_HOST_COMMAND_SIZE);
CHECK(W2R_HOST_HOTKEY_SIZE > W2R_HOST_SNAPSHOT_SIZE);
CHECK(W2R_HOST_COMMAND_SIZE > W2R_HOST_HOTKEY_SIZE);
CHECK(W2R_HOST_SET_CHOICES_SIZE > W2R_HOST_OVERLAY_SIZE);
CHECK(W2R_HOST_SNAPSHOT_SIZE > W2R_HOST_SET_CHOICES_SIZE);
CHECK(W2R_HOST_SETTINGS_DECLARE_SIZE > W2R_HOST_MAP_FOLDER_SIZE);
CHECK(W2R_HOST_OVERLAY_SIZE > W2R_HOST_SETTINGS_DECLARE_SIZE);
CHECK(sizeof(W2rOverlayItem) == 24);
CHECK(offsetof(W2rSettingDef, struct_size) == 0);
CHECK(offsetof(W2rSettingDef, default_int) == 8);
CHECK(offsetof(W2rSettingDef, flags) == 32);
CHECK(offsetof(W2rSettingDef, choices) == 40);
CHECK(offsetof(W2rSettingDef, key) == (sizeof(void *) == 4 ? 48 : 56));
CHECK(offsetof(W2rSettingDef, active_when) == (sizeof(void *) == 4 ? 96 : 152));
CHECK(sizeof(W2rSettingDef) == (sizeof(void *) == 4 ? 104 : 168));
CHECK(W2R_SETTING_DEF_V1_SIZE == sizeof(W2rSettingDef));
CHECK(W2R_MAX_PEER_PAYLOAD + W2R_PEER_HEADER_BYTES <= 989);

/* The smallest possible plugin must compile against the header. */
static W2rStatus W2R_CALL on_load(const W2rHost *host) {
    W2rStr msg = {"hello", 5};
    return host->log(host->token, W2R_LOG_INFO, msg);
}
static const W2rPlugin kPlugin = {sizeof(W2rPlugin), on_load, 0, 0, 0};

W2R_EXPORT const W2rPlugin *W2R_CALL w2r_plugin_main(const W2rHost *host) {
    return host->abi_version >= 1 ? &kPlugin : 0;
}
