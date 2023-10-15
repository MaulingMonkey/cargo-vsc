#![allow(dead_code)]

use serde::*;

use std::collections::HashSet;
use std::io;
use std::path::PathBuf;
use std::process::{Command, Stdio};



pub(crate) type Url = String;
pub(crate) type PackageId = String;

/// cargo metadata<br>
/// `{ ... }`
#[derive(Deserialize, Debug)]
pub(crate) struct Root {
    pub workspace_root: PathBuf,
    pub packages: Vec<PackageRef>,
    pub workspace_members: HashSet<PackageId>,
    pub metadata: Option<Metadata>,
    // ...
}

/// cargo metadata<br>
/// `{ "packages": [ ... ] }`
#[derive(Deserialize, Debug)]
pub(crate) struct PackageRef {
    pub manifest_path:  PathBuf,
    pub id:             PackageId,
    pub name:           String,
    pub version:        String,
    pub repository:     Option<Url>, // present in cargo +1.47.0 metadata
    pub documentation:  Option<Url>, // MISSING in cargo +1.47.0 metadata, might lead to fewer tasks.json links in older cargo
    pub homepage:       Option<Url>, // MISSING in cargo +1.47.0 metadata, might lead to fewer tasks.json links in older cargo
    pub targets:        Vec<PackageTarget>,
    pub metadata:       Option<Metadata>,
    // ...
}

/// cargo metadata<br>
/// `{ "packages": [ { "manifest_path": "..." } ] }`
#[derive(Deserialize, Debug)]
pub(crate) struct PackageTarget {
    pub kind:           Vec<String>, // "lib", "example" (, "bin"?)
    pub crate_types:    Vec<String>, // "lib", "bin"
    pub name:           String,
    //pub src_path:       PathBuf,
    //pub edition:        String,
    //pub doctest:        bool,
    //pub test:           bool,
}

/// cargo metadata<br>
/// `{ "packages": [ { "metadata": {...} } ] }` or<br>
/// `{ "metadata": {...} }` (workspace)
#[derive(Deserialize, Debug)]
pub(crate) struct Metadata {
    pub local_install: Option<de::IgnoredAny>,
    #[serde(rename="cargo-vsc")] #[serde(default)] pub cargo_vsc: MetadataCargoVsc,
}

/// Cargo.toml<br>
///
/// ```toml
/// [package.metadata.cargo-vsc] # or:
///
/// [workspace.metadata.cargo-vsc]
/// simple = true
/// ```
#[derive(Deserialize, Debug, Default)]
pub(crate) struct MetadataCargoVsc {
    pub simple: Option<bool>,
}



impl Root {
    pub fn get() -> io::Result<Self> {
        let o = Command::new("cargo").args(&["metadata", "--all-features", "--format-version", "1"]).stderr(Stdio::inherit()).output()?;
        match o.status.code() {
            Some(0) => {},
            Some(n) => return Err(io::Error::new(io::ErrorKind::BrokenPipe, format!("`cargo metadata` failed (exit code {})", n))),
            None    => return Err(io::Error::new(io::ErrorKind::BrokenPipe, "`cargo metadata` failed (signal)")),
        }
        let stdout = String::from_utf8(o.stdout).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
        serde_json::from_str(&stdout).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
    }
}
