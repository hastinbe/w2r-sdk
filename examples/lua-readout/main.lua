-- Example script plugin: a readout of your resources and the match clock.
--
-- It shows what the new plugin API offers a script: the game state (w2r.game.state), text in the
-- HUD at a chosen place (w2r.hud.text), a hotkey and a chat command, the match-end event with
-- everyone's tallies, and a setting on the options screen.
--
--   Ctrl+Alt+R     switch the readout on or off
--   /readout off   the same by typing, "/readout on" brings it back

w2r.setting.define{ key = "show", label = "Show the readout", type = "bool", default = true }

local visible = true

local function clock(seconds)
  local s = math.floor(seconds)
  return string.format("%d:%02d", s // 60, s % 60)
end

local function hide()
  w2r.hud.text("resources", nil)
  w2r.hud.text("clock", nil)
end

local function refresh()
  if not visible then hide() return end
  local state = w2r.game.state()
  if not state then return end
  for _, p in ipairs(state.players) do
    if p.is_local and p.gold then
      -- gold, lumber and oil are only present for you and your allies
      w2r.hud.text("resources",
        string.format("Gold %d   Lumber %d   Oil %d   Food %d/%d",
          p.gold, p.lumber, p.oil, p.food_used or 0, p.food_cap or 0),
        "top_left")
    end
  end
  w2r.hud.text("clock", (state.paused and "Paused " or "") .. clock(state.time), "top_center")
end

-- the setting, or on when it cannot be read
local function wanted()
  local ok, value = pcall(w2r.setting.get, "show")
  return not ok or value ~= false
end

visible = wanted()
w2r.every(1.0, function()
  if w2r.in_match() then refresh() end
end)

w2r.on("settings_changed", function()
  visible = wanted()
  refresh()
end)

w2r.on("match_end", function(e)
  hide()
  for _, p in ipairs(e.players) do
    if p.is_local then
      w2r.log(string.format("match over on %s: %s, %d units killed, %d gold mined, %d turns",
        e.map, p.outcome, p.units_killed, p.gold_mined, e.turns))
    end
  end
end)

local ok, why = w2r.hotkey("Ctrl+Alt+R", function()
  visible = not visible
  refresh()
end)
if not ok then w2r.log("hotkey not available: " .. tostring(why)) end

w2r.command("readout", function(e)
  visible = e.args ~= "off"
  refresh()
end)
