use super::types::{Model3Group, Model3References};
use crate::{Error, Result};
use serde::Deserialize;

const FORMAT: &str = "model3.json";

/// A parsed Cubism `model3.json` manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct Model3 {
    version: u32,
    file_references: Model3References,
    groups: Vec<Model3Group>,
}

impl Model3 {
    /// Parses a model manifest from UTF-8 JSON bytes.
    pub fn from_json_bytes(source: &[u8]) -> Result<Self> {
        let source = std::str::from_utf8(source).map_err(|error| Error::InvalidJson {
            format: FORMAT,
            message: error.to_string(),
        })?;
        Self::from_json_str(source)
    }

    /// Parses a model manifest from UTF-8 JSON.
    pub fn from_json_str(source: &str) -> Result<Self> {
        let raw: RawModel3 = serde_json::from_str(source).map_err(|error| Error::InvalidJson {
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
            file_references: raw.file_references,
            groups: raw.groups,
        })
    }
    /// Returns the model manifest version.
    pub fn version(&self) -> u32 {
        self.version
    }
    /// Returns referenced MOC3, textures, and motions.
    pub fn file_references(&self) -> &Model3References {
        &self.file_references
    }
    /// Returns automatic parameter groups.
    pub fn groups(&self) -> &[Model3Group] {
        &self.groups
    }
}

#[derive(Deserialize)]
struct RawModel3 {
    #[serde(rename = "Version")]
    version: u32,
    #[serde(rename = "FileReferences")]
    file_references: Model3References,
    #[serde(rename = "Groups", default)]
    groups: Vec<Model3Group>,
}
