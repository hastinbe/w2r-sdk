# Readout (example script plugin)

A small Lua plugin that shows your gold, lumber, oil, food and the match clock at the top of the
screen. It is here to show how a script uses the plugin API: reading the game state, drawing HUD
text at a place, registering a hotkey and a chat command, reacting to the end of a match, and
declaring a setting.

To try it, zip `plugin.json` and `main.lua` together (with `plugin.json` at the top of the archive)
and install the zip from the W2R Mod Loader with **Install from file...**
(see ../../docs/sharing-plugins.md). Lua plugins are always allowed to install from a file.
