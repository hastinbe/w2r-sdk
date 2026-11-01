//! `plugin.json` validation (docs/manifest.md). Mirrors `tools/w2r_spec/manifest.py` check for
//! check and in the same order; `docs/vectors/manifest.json` is the contract and the test below
//! runs every vector.

use crate::json::{self, Value};
use crate::semver::{Version, VersionReq};

pub const MAX_BYTES: usize = 64 * 1024;
pub const CAPABILITIES: [&str; 7] = [
    "game.read",
    "chat.read",
    "chat.write",
    "net.send",
    "hooks.raw",
    "game.control",
    "net.http",
];
const FIELDS: [&str; 12] = [
    "schema",
    "id",
    "name",
    "version",
    "author",
    "description",
    "entry",
    "abi",
    "game_builds",
    "capabilities",
    "affects_simulation",
    "dependencies",
];
/// Allowed but not required (docs/manifest.md).
const OPTIONAL_FIELDS: [&str; 2] = ["replaces", "net_hosts"];

/// A reason code from docs/manifest.md, or `"ok"`.
pub type Reason = &'static str;

/// `a.b.c.d`, each part 1 to 5 digits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Build(pub [u32; 4]);

impl Build {
    pub fn parse(s: &str) -> Option<Build> {
        let mut out = [0u32; 4];
        let mut parts = s.split('.');
        for slot in &mut out {
            let p = parts.next()?;
            if p.is_empty() || p.len() > 5 || !p.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            *slot = p.parse().ok()?;
        }
        if parts.next().is_some() {
            return None;
        }
        Some(Build(out))
    }
}

impl core::fmt::Display for Build {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let [a, b, c, d] = self.0;
        write!(f, "{a}.{b}.{c}.{d}")
    }
}

/// One `game_builds` entry: a single build or an inclusive range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuildRange {
    pub min: Build,
    pub max: Build,
}

impl core::fmt::Display for BuildRange {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.min == self.max {
            write!(f, "{}", self.min)
        } else {
            write!(f, "{}-{}", self.min, self.max)
        }
    }
}

impl BuildRange {
    pub fn parse(s: &str) -> Option<BuildRange> {
        let (min, max) = match s.split_once('-') {
            Some((a, b)) => (Build::parse(a)?, Build::parse(b)?),
            None => {
                let b = Build::parse(s)?;
                (b, b)
            }
        };
        if min > max {
            return None;
        }
        Some(BuildRange { min, max })
    }

    pub fn contains(&self, b: Build) -> bool {
        self.min <= b && b <= self.max
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dependency {
    pub id: String,
    pub version_req: VersionReq,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Manifest {
    pub id: String,
    pub name: String,
    pub version: Version,
    pub version_text: String,
    pub author: String,
    pub description: String,
    pub entry: String,
    pub abi: u32,
    pub game_builds: Vec<BuildRange>,
    pub capabilities: Vec<String>,
    pub affects_simulation: bool,
    pub dependencies: Vec<Dependency>,
    /// Plugins this one supersedes (`replaces`; empty when absent). Same shape as a dependency:
    /// an id and the versions of it that are replaced.
    pub replaces: Vec<Dependency>,
    /// The servers a plugin with `net.http` may talk to (`net_hosts`; empty without `net.http`).
    pub net_hosts: Vec<String>,
}

impl Manifest {
    /// Whether this manifest replaces the plugin `id` at `version`.
    pub fn replaces_plugin(&self, id: &str, version: &Version) -> bool {
        self.replaces
            .iter()
            .any(|r| r.id == id && r.version_req.matches(version))
    }

    pub fn supports_build(&self, b: Build) -> bool {
        self.game_builds.iter().any(|r| r.contains(b))
    }

    /// Whether the entry is a Lua script (docs/lua-plugins.md) rather than a DLL.
    pub fn is_script(&self) -> bool {
        self.entry.ends_with(SCRIPT_SUFFIX)
    }
}

/// `CON`, `PRN`, `AUX`, `NUL`, `COM1`-`COM9`, `LPT1`-`LPT9`, case-insensitively.
pub fn is_reserved_device(stem: &str) -> bool {
    let up = stem.to_ascii_uppercase();
    matches!(up.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (up.len() == 4
            && (up.starts_with("COM") || up.starts_with("LPT"))
            && matches!(up.as_bytes()[3], b'1'..=b'9'))
}

/// `[a-z0-9][a-z0-9-]{1,62}[a-z0-9]` and not a device name.
pub fn is_valid_id(s: &str) -> bool {
    let b = s.as_bytes();
    let edge = |c: u8| c.is_ascii_lowercase() || c.is_ascii_digit();
    (3..=64).contains(&b.len())
        && edge(b[0])
        && edge(b[b.len() - 1])
        && b[1..b.len() - 1].iter().all(|&c| edge(c) || c == b'-')
        && !is_reserved_device(s)
}

/// The entry suffix of a Lua script plugin (docs/lua-plugins.md); anything else is a DLL.
pub const SCRIPT_SUFFIX: &str = ".lua";
/// The capabilities a script plugin may ask for: it has no way to use the others.
pub const SCRIPT_CAPABILITIES: &[&str] =
    &["chat.read", "chat.write", "game.read", "game.control", "net.http"];

/// `[A-Za-z0-9_][A-Za-z0-9_.-]{0,59}\.(dll|lua)`, no `..`, stem not a device name.
fn is_valid_entry(s: &str) -> bool {
    let Some(stem) = s
        .strip_suffix(".dll")
        .or_else(|| s.strip_suffix(SCRIPT_SUFFIX))
    else {
        return false;
    };
    let b = stem.as_bytes();
    let ok_rest = |c: u8| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'.' | b'-');
    (1..=60).contains(&b.len())
        && (b[0].is_ascii_alphanumeric() || b[0] == b'_')
        && b[1..].iter().all(|&c| ok_rest(c))
        && !s.contains("..")
        && !is_reserved_device(s.split('.').next().unwrap_or(""))
}

fn text(v: &Value, max_bytes: usize) -> Result<String, Reason> {
    let s = v.as_str().ok_or("bad_type")?;
    if s.is_empty() || s.chars().any(|c| (c as u32) < 0x20 || c as u32 == 0x7F) {
        return Err("bad_string");
    }
    if s.len() > max_bytes {
        return Err("too_long");
    }
    Ok(s.to_owned())
}

fn builds(v: &Value) -> Result<Vec<BuildRange>, Reason> {
    let items = v.as_array().ok_or("bad_type")?;
    if !(1..=16).contains(&items.len()) {
        return Err("bad_build");
    }
    items
        .iter()
        .map(|b| b.as_str().and_then(BuildRange::parse).ok_or("bad_build"))
        .collect()
}

fn capabilities(v: &Value) -> Result<Vec<String>, Reason> {
    let items = v.as_array().ok_or("bad_type")?;
    let mut out: Vec<String> = Vec::with_capacity(items.len());
    for c in items {
        let c = c.as_str().ok_or("bad_capability")?;
        if !CAPABILITIES.contains(&c) || out.iter().any(|seen| seen == c) {
            return Err("bad_capability");
        }
        out.push(c.to_owned());
    }
    Ok(out)
}

fn dependencies(v: &Value, own_id: &str) -> Result<Vec<Dependency>, Reason> {
    let items = v.as_array().ok_or("bad_type")?;
    if items.len() > 16 {
        return Err("bad_dependency");
    }
    let mut out: Vec<Dependency> = Vec::with_capacity(items.len());
    for d in items {
        let pairs = d.as_object().ok_or("bad_dependency")?;
        let mut keys: Vec<&str> = pairs.iter().map(|(k, _)| k.as_str()).collect();
        keys.sort_unstable();
        if keys != ["id", "version_req"] {
            return Err("bad_dependency");
        }
        let id = d
            .get("id")
            .and_then(Value::as_str)
            .ok_or("bad_dependency")?;
        let req = d
            .get("version_req")
            .and_then(Value::as_str)
            .ok_or("bad_dependency")?;
        if !is_valid_id(id) || id == own_id || out.iter().any(|seen| seen.id == id) {
            return Err("bad_dependency");
        }
        let version_req = VersionReq::parse(req).ok_or("bad_dependency")?;
        out.push(Dependency {
            id: id.to_owned(),
            version_req,
        });
    }
    Ok(out)
}

/// The optional `replaces` list: the same entries as `dependencies`, with `bad_replaces` for any
/// problem, and no id may also be a dependency (a plugin cannot need what it replaces).
fn replaces(v: &Value, own_id: &str, deps: &[Dependency]) -> Result<Vec<Dependency>, Reason> {
    let list = dependencies(v, own_id).map_err(|r| match r {
        "bad_type" => "bad_type",
        _ => "bad_replaces",
    })?;
    if list.iter().any(|r| deps.iter().any(|d| d.id == r.id)) {
        return Err("bad_replaces");
    }
    Ok(list)
}

/// A host name a plugin may be given leave to contact: lowercase DNS labels of `a-z`, `0-9` and
/// `-`, at least two of them, each 1 to 63 bytes and neither starting nor ending with `-`, at most
/// 253 bytes in all, and a last label with a letter in it (so an IP address never qualifies).
pub fn is_valid_net_host(s: &str) -> bool {
    if s.is_empty() || s.len() > 253 {
        return false;
    }
    let labels: Vec<&str> = s.split('.').collect();
    labels.len() >= 2
        && labels.iter().all(|l| {
            (1..=63).contains(&l.len())
                && !l.starts_with('-')
                && !l.ends_with('-')
                && l.bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        })
        && labels
            .last()
            .is_some_and(|l| l.bytes().any(|b| b.is_ascii_lowercase()))
}

/// The optional `net_hosts` list: 1 to 4 distinct valid host names, and only with `net.http`
/// (which in turn needs it): `bad_net_hosts` for any mismatch, `bad_type` when it is not a list.
fn net_hosts(doc: &Value, capabilities: &[String]) -> Result<Vec<String>, Reason> {
    let wants = capabilities.iter().any(|c| c == "net.http");
    let Some(v) = doc.get("net_hosts") else {
        return if wants { Err("bad_net_hosts") } else { Ok(Vec::new()) };
    };
    let items = v.as_array().ok_or("bad_type")?;
    if !wants || items.is_empty() || items.len() > 4 {
        return Err("bad_net_hosts");
    }
    let mut out: Vec<String> = Vec::new();
    for item in items {
        let host = item.as_str().filter(|h| is_valid_net_host(h)).ok_or("bad_net_hosts")?;
        if out.iter().any(|h| h == host) {
            return Err("bad_net_hosts");
        }
        out.push(host.to_owned());
    }
    Ok(out)
}

/// Validates raw `plugin.json` bytes. `Err` carries the reason code.
pub fn validate(raw: &[u8]) -> Result<Manifest, Reason> {
    if raw.len() > MAX_BYTES {
        return Err("too_large");
    }
    let doc = json::parse(raw).map_err(|r| r.code())?;
    let pairs = doc.as_object().ok_or("not_object")?;
    if pairs
        .iter()
        .any(|(k, _)| !FIELDS.contains(&k.as_str()) && !OPTIONAL_FIELDS.contains(&k.as_str()))
    {
        return Err("unknown_field");
    }
    if FIELDS.iter().any(|f| doc.get(f).is_none()) {
        return Err("missing_field");
    }
    let field = |name: &str| doc.get(name).expect("checked above");

    if field("schema").as_number().and_then(|n| n.as_u64()) != Some(1) {
        return Err("bad_schema");
    }
    let id = field("id")
        .as_str()
        .filter(|s| is_valid_id(s))
        .ok_or("bad_id")?
        .to_owned();
    let name = text(field("name"), 64)?;
    let author = text(field("author"), 64)?;
    let description = text(field("description"), 256)?;
    let version_text = field("version").as_str().ok_or("bad_version")?.to_owned();
    let version = Version::parse(&version_text).ok_or("bad_version")?;
    let entry = field("entry")
        .as_str()
        .filter(|s| is_valid_entry(s))
        .ok_or("bad_entry")?
        .to_owned();
    let abi = field("abi")
        .as_number()
        .and_then(|n| n.as_u64())
        .filter(|&n| (1..=u32::MAX as u64).contains(&n))
        .ok_or("bad_abi")? as u32;
    let game_builds = builds(field("game_builds"))?;
    let capabilities = capabilities(field("capabilities"))?;
    let affects_simulation = field("affects_simulation").as_bool().ok_or("bad_type")?;
    // A script cannot read game memory, hook the game, send peer messages or change the
    // simulation; a manifest that claims otherwise is a mistake, refused as `bad_script`.
    if entry.ends_with(SCRIPT_SUFFIX)
        && (affects_simulation
            || capabilities
                .iter()
                .any(|c| !SCRIPT_CAPABILITIES.contains(&c.as_str())))
    {
        return Err("bad_script");
    }
    let dependencies = dependencies(field("dependencies"), &id)?;
    let replaces = match doc.get("replaces") {
        Some(v) => replaces(v, &id, &dependencies)?,
        None => Vec::new(),
    };
    let net_hosts = net_hosts(&doc, &capabilities)?;

    Ok(Manifest {
        id,
        name,
        version,
        version_text,
        author,
        description,
        entry,
        abi,
        game_builds,
        capabilities,
        affects_simulation,
        dependencies,
        replaces,
        net_hosts,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const VECTORS: &str = include_str!("../../../docs/vectors/manifest.json");

    fn raw_of(case: &Value) -> Vec<u8> {
        match case.get("raw") {
            Some(raw) => {
                let mut bytes = raw.as_str().unwrap().as_bytes().to_vec();
                let pad = case
                    .get("pad")
                    .and_then(Value::as_number)
                    .and_then(|n| n.as_u64())
                    .unwrap_or(0);
                bytes.extend(std::iter::repeat_n(b' ', pad as usize));
                bytes
            }
            None => case.get("manifest").unwrap().to_json().into_bytes(),
        }
    }

    #[test]
    fn every_vector_agrees_with_the_reference() {
        let cases = json::parse(VECTORS.as_bytes()).expect("vectors parse");
        let cases = cases.as_array().unwrap();
        assert!(cases.len() >= 80);
        let mut names = std::collections::BTreeSet::new();
        for case in cases {
            let name = case.get("name").unwrap().as_str().unwrap();
            assert!(names.insert(name), "duplicate vector {name}");
            let expect = case.get("expect").unwrap().as_str().unwrap();
            let got = validate(&raw_of(case)).map(|_| "ok").unwrap_or_else(|e| e);
            assert_eq!(got, expect, "vector {name}");
        }
    }

    #[test]
    fn the_example_manifest_is_fully_decoded() {
        let cases = json::parse(VECTORS.as_bytes()).unwrap();
        let full = cases
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c.get("name").unwrap().as_str() == Some("valid_full"))
            .unwrap();
        let m = validate(&raw_of(full)).unwrap();
        assert_eq!(m.id, "example-plugin");
        assert_eq!(m.version_text, "2.0.0-beta.1");
        assert_eq!(m.abi, 1);
        assert_eq!(m.entry, "example_plugin.dll");
        assert!(m.affects_simulation);
        assert_eq!(m.capabilities, CAPABILITIES);
        assert_eq!(m.net_hosts, vec!["api.example.com".to_owned()]);
        assert_eq!(m.game_builds.len(), 2);
        assert!(m.supports_build(Build::parse("1.0.2.2818").unwrap()));
        assert!(m.supports_build(Build::parse("1.0.3.0").unwrap()));
        assert!(!m.supports_build(Build::parse("1.0.3.101").unwrap()));
        assert_eq!(m.dependencies.len(), 1);
        assert_eq!(m.dependencies[0].id, "other-plugin");
        assert!(
            m.dependencies[0]
                .version_req
                .matches(&Version::parse("1.5.0").unwrap())
        );
    }

    #[test]
    fn helpers_cover_the_edges() {
        assert!(is_valid_id("ab0"));
        assert!(is_valid_id(&format!("a{}b", "-".repeat(62))));
        assert!(!is_valid_id("ab"));
        assert!(!is_valid_id("-ab"));
        assert!(!is_valid_id("ab-"));
        assert!(!is_valid_id("Ab0"));
        assert!(!is_valid_id("lpt9"));
        assert!(is_valid_id("com0"));
        assert!(is_valid_entry("a.dll"));
        assert!(is_valid_entry("_x-1.2.dll"));
        assert!(!is_valid_entry("a..dll"));
        assert!(!is_valid_entry(".a.dll"));
        assert!(!is_valid_entry("a.DLL"));
        assert!(!is_valid_entry("CON.dll"));
        assert!(!is_valid_entry("nul.x.dll"));
        assert!(!is_valid_entry("a\\b.dll"));
        assert!(!is_valid_entry(&format!("{}.dll", "a".repeat(61))));
        assert!(is_valid_entry(&format!("{}.dll", "a".repeat(60))));
        assert_eq!(
            Build::parse("1.0.2.2818").unwrap().to_string(),
            "1.0.2.2818"
        );
        assert!(Build::parse("1.0.2").is_none());
        assert!(Build::parse("1.0.2.123456").is_none());
        assert!(BuildRange::parse("1.0.3.0-1.0.2.0").is_none());
    }
}
