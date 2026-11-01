# Sharing a plugin without publishing it

A plugin does not have to be in the plugin index to be used. Anyone can hand a friend a package file
(`<id>-<version>.zip`, the same file the index would serve) and the friend installs it from the W2R
Mod Loader:

- **App:** Installed tab, **Install from file...**, or drop the zip on the window.
- **Command line:** `w2r-loader install-file <file.zip>`.

The manager shows the same consent window as for a published plugin (what the plugin may do, which
game builds it targets), says plainly that the package is not from the index, and records the SHA-256
of the plugin file in `state.json`, so the game still refuses it if the file changes afterwards.

## What is different from a published plugin

| | Published | From a file |
|---|---|---|
| Reviewed by the index maintainers | yes | no |
| Updated from the index | yes | never |
| Shown in the Installed tab | plain | with a "From a file" tag |
| May use the id of a published plugin | yes (it is that plugin) | no, pick your own id |
| May list other plugins in `replaces` | yes | no |
| Native DLL | yes | only if the user turned on **Allow native plugins (DLLs) from files** in Settings (`w2r-loader allow-native-files on`) |
| Lua script (`.lua` entry) | yes | yes, always |

A plugin from a file is marked by a `LOCAL.txt` in its folder. A package cannot contain that file, so
it cannot claim to be local or index-installed by itself.

## Why native DLLs are off by default

A native plugin runs inside the game with the player's permissions and no sandbox. A script plugin
runs in the Lua sandbox with a small, fixed API (docs/lua-plugins.md in the mod loader), so sending
someone a script is low risk. Sending a DLL is the same as sending an executable, so the receiver has
to opt in first, and the consent window repeats the warning.

## Rules for a shared package

- It is a normal package: `plugin.json`, the entry file, optionally `README.md` and `LICENSE`, nothing
  else (see [manifest.md](manifest.md)).
- The `id` must not be the id of a plugin in the index, and not the id of a plugin the receiver
  installed from the index. Receivers who already have it from the index must remove it first.
- Installing the same id again from a file replaces the files (a reinstall or an update), so a friend
  can send you a new build and you install over the old one.
- Multiplayer: a plugin with `affects_simulation` must be installed, at the same version, by every
  player in the match, exactly as for published plugins (see [load-policy.md](load-policy.md)).
- Removing it (Installed tab, or `w2r-loader remove <id>`) forgets it completely.

## Making the package

For a plugin written in this repository's style, `w2r-publish package` writes it (see
[plugin-dev-setup.md](plugin-dev-setup.md)). A Lua plugin only needs a folder with `plugin.json` and
the `.lua` file: zip those two files together, with `plugin.json` at the top of the archive.
