# Overlay drawing (example)

A native plugin that draws on the map view every frame with the `W2rOverlay` table
(`docs/abi.md`, "Overlay", "Drawing every frame"). It needs no capability and core 2.7.0 or later.

In a match or a replay it draws:

- a small card in the top-left corner of the map view. It uses every call once: `fill`, `outline`,
  `line`, `circle` (a ring and a filled one), `label` (left, right and the HUD's own colour) and
  `measure` (a box exactly as wide as its text). It follows the screen, because it is placed from
  `view`.
- a gold ring with the caption "tile 8, 8" around that tile. It stays on the map and scrolls
  with it.

## Build and install

```
cargo build --release --target i686-pc-windows-msvc
```

Copy `plugin.json` and `target\i686-pc-windows-msvc\release\example_overlay.dll` into one folder
and use **Install from file** in the manager on that folder.

## What to check in the game

- The card sits in the corner and does not move when the map scrolls; the ring does.
- The card's parts at the edge of the map view are cut off there, not drawn over the HUD.
- The resource bar's text stays on top of anything drawn under it.
- The box behind "measured text" is exactly as wide as the text.
