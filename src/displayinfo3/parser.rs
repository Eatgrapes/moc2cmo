use serde::Deserialize;

use crate::{Error, Result};

use super::types::{DisplayInfo3Parameter, DisplayInfo3ParameterGroup, DisplayInfo3Part};

const FORMAT: &str = "cdi3.json";

/// A parsed Cubism display-info document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayInfo3 {
    version: u32,
    parameters: Vec<DisplayInfo3Parameter>,
    parameter_groups: Vec<DisplayInfo3ParameterGroup>,
    parts: Vec<DisplayInfo3Part>,
}

impl DisplayInfo3 {
    /// Reads and parses a display-info document from a UTF-8 JSON file.
    pub fn from_json_file(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let path = path.as_ref();
        let bytes = std::fs::read(path).map_err(|source| Error::Io {
            path: path.to_path_buf(),
            source,
        })?;
        Self::from_json_bytes(&bytes)
    }

    /// Parses a display-info document from UTF-8 JSON bytes.
    pub fn from_json_bytes(source: &[u8]) -> Result<Self> {
        let source = std::str::from_utf8(source).map_err(|error| Error::InvalidJson {
            format: FORMAT,
            message: error.to_string(),
        })?;
        Self::from_json_str(source)
    }

    /// Parses a display-info document from UTF-8 JSON.
    pub fn from_json_str(source: &str) -> Result<Self> {
        let raw: RawDisplayInfo3 =
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
            parameters: raw.parameters,
            parameter_groups: raw.parameter_groups,
            parts: raw.parts,
        })
    }

    /// Returns the display-info format version.
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Returns named parameters.
    pub fn parameters(&self) -> &[DisplayInfo3Parameter] {
        &self.parameters
    }

    /// Returns named parameter groups.
    pub fn parameter_groups(&self) -> &[DisplayInfo3ParameterGroup] {
        &self.parameter_groups
    }

    /// Returns named parts.
    pub fn parts(&self) -> &[DisplayInfo3Part] {
        &self.parts
    }
}

#[derive(Deserialize)]
struct RawDisplayInfo3 {
    #[serde(rename = "Version")]
    version: u32,
    #[serde(rename = "Parameters", default)]
    parameters: Vec<DisplayInfo3Parameter>,
    #[serde(rename = "ParameterGroups", default)]
    parameter_groups: Vec<DisplayInfo3ParameterGroup>,
    #[serde(rename = "Parts", default)]
    parts: Vec<DisplayInfo3Part>,
}
