# W2R SDK

What you need to write a plugin for the W2R Mod Loader (Warcraft II Remastered), MIT licensed.

- `crates/w2r-abi`: the plugin C ABI. `include/w2r_abi.h` is the source of truth; the crate is its
  Rust binding.
- `crates/w2r-formats`: strict parsers for `plugin.json`, `state.json`, `index.json`, JSON and SemVer.
  The mod loader and the plugin publisher use the same code, so what passes here passes there.
- `crates/w2r-preview`: PUD parsing and map preview rendering.
- `docs/`: [abi.md](docs/abi.md), [manifest.md](docs/manifest.md), [index.md](docs/index.md),
  [plugin-dev-setup.md](docs/plugin-dev-setup.md), and `docs/vectors/` (test vectors both the Rust
  and the Python reference implementations must pass).
- `tools/`: the Python reference implementation of the manifest and index rules (`w2r_spec`), the
  vector generators and the ABI header check.

The mod loader itself (the game host and the manager) is not part of this repository.

## Checks

```
bash tools/run_all.sh
```

Needs Python 3, a C compiler (gcc or MSVC) and Rust 1.85+.

## License

MIT, see [LICENSE](LICENSE). Each crate also carries a `LICENSE-MIT`.
