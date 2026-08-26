use serde::Deserialize;
use serde_json::Value;

use crate::{Error, Result};

const FORMAT: &str = "physics3.json";

/// A parsed Cubism physics document.
#[derive(Debug, Clone, PartialEq)]
pub struct Physics3 {
    version: u32,
    meta: Value,
    settings: Vec<Value>,
}

impl Physics3 {
    /// Parses a physics document from UTF-8 JSON bytes.
    pub fn from_json_bytes(source: &[u8]) -> Result<Self> {
        let source = std::str::from_utf8(source).map_err(|error| Error::InvalidJson {
            format: FORMAT,
            message: error.to_string(),
        })?;
        Self::from_json_str(source)
    }

    /// Parses a physics document from UTF-8 JSON.
    pub fn from_json_str(source: &str) -> Result<Self> {
        let raw: RawPhysics3 =
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
            settings: raw.physics_settings,
        })
    }

    /// Returns the physics format version.
    pub fn version(&self) -> u32 {
        self.version
    }
    /// Returns the metadata object from the source document.
    pub fn meta(&self) -> &Value {
        &self.meta
    }
    /// Returns physics settings as JSON values, preserving all source fields.
    pub fn settings(&self) -> &[Value] {
        &self.settings
    }
}

#[derive(Deserialize)]
struct RawPhysics3 {
    #[serde(rename = "Version")]
    version: u32,
    #[serde(rename = "Meta", default)]
    meta: Value,
    #[serde(rename = "PhysicsSettings", default)]
    physics_settings: Vec<Value>,
}
