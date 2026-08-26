use serde::Deserialize;

use crate::{Error, Result};

use super::types::{UserData3Entry, UserData3Meta};

const FORMAT: &str = "userdata3.json";

/// A parsed Cubism userdata document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserData3 {
    version: u32,
    meta: UserData3Meta,
    entries: Vec<UserData3Entry>,
}

impl UserData3 {
    /// Reads and parses a userdata document from a UTF-8 JSON file.
    pub fn from_json_file(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let path = path.as_ref();
        let bytes = std::fs::read(path).map_err(|source| Error::Io {
            path: path.to_path_buf(),
            source,
        })?;
        Self::from_json_bytes(&bytes)
    }

    /// Parses a userdata document from UTF-8 JSON bytes.
    pub fn from_json_bytes(source: &[u8]) -> Result<Self> {
        let source = std::str::from_utf8(source).map_err(|error| Error::InvalidJson {
            format: FORMAT,
            message: error.to_string(),
        })?;
        Self::from_json_str(source)
    }

    /// Parses a userdata document from UTF-8 JSON.
    pub fn from_json_str(source: &str) -> Result<Self> {
        let raw: RawUserData3 =
            serde_json::from_str(source).map_err(|error| Error::InvalidJson {
                format: FORMAT,
                message: error.to_string(),
            })?;
        if raw.version != 3 {
            return Err(Error::UnsupportedVersion {
                format: FORMAT,
                version: raw.version,
            });
        }
        Ok(Self {
            version: raw.version,
            meta: raw.meta,
            entries: raw.entries,
        })
    }

    /// Returns the userdata format version.
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Returns source metadata counters.
    pub fn meta(&self) -> &UserData3Meta {
        &self.meta
    }

    /// Returns all userdata annotations.
    pub fn entries(&self) -> &[UserData3Entry] {
        &self.entries
    }
}

#[derive(Deserialize)]
struct RawUserData3 {
    #[serde(rename = "Version")]
    version: u32,
    #[serde(rename = "Meta", default)]
    meta: UserData3Meta,
    #[serde(rename = "UserData", default)]
    entries: Vec<UserData3Entry>,
}
