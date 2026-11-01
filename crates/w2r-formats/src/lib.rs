//! The parsers the host and the manager must agree on, byte for byte: strict JSON, SemVer,
//! `plugin.json`, `state.json` and `index.json`. Specs: `docs/manifest.md`, `docs/index.md`, `docs/layout.md`.

pub mod index;
pub mod json;
pub mod manifest;
pub mod semver;
pub mod state;

/// A stable reason code from the specs (`rollback`, `bad_hash`, ...).
pub type Reason = &'static str;
