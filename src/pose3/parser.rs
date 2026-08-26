use serde::Deserialize;

use super::types::PoseGroup;
use crate::{Error, Result};

const FORMAT: &str = "pose3.json";

/// A parsed Cubism pose document.
#[derive(Debug, Clone, PartialEq)]
pub struct Pose3 {
    fade_in_time: f32,
    groups: Vec<PoseGroup>,
}

impl Pose3 {
    /// Reads and parses a pose document from a UTF-8 JSON file.
    pub fn from_json_file(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let path = path.as_ref();
        let bytes = std::fs::read(path).map_err(|source| Error::Io {
            path: path.to_path_buf(),
            source,
        })?;
        Self::from_json_bytes(&bytes)
    }

    /// Parses a pose document from UTF-8 JSON bytes.
    pub fn from_json_bytes(source: &[u8]) -> Result<Self> {
        let source = std::str::from_utf8(source).map_err(|error| Error::InvalidJson {
            format: FORMAT,
            message: error.to_string(),
        })?;
        Self::from_json_str(source)
    }

    /// Parses a pose document from UTF-8 JSON.
    pub fn from_json_str(source: &str) -> Result<Self> {
        let raw: RawPose3 = serde_json::from_str(source).map_err(|error| Error::InvalidJson {
            format: FORMAT,
            message: error.to_string(),
        })?;
        if raw.pose_type != "Live2D Pose" {
            return Err(Error::InvalidJson {
                format: FORMAT,
                message: format!("unexpected pose type {:?}", raw.pose_type),
            });
        }
        Ok(Self {
            fade_in_time: raw.fade_in_time,
            groups: raw.groups,
        })
    }

    /// Returns the pose fade-in duration in seconds.
    pub fn fade_in_time(&self) -> f32 {
        self.fade_in_time
    }
    /// Returns pose groups and their parts.
    pub fn groups(&self) -> &[PoseGroup] {
        &self.groups
    }
}

#[derive(Deserialize)]
struct RawPose3 {
    #[serde(rename = "Type")]
    pose_type: String,
    #[serde(rename = "FadeInTime", default)]
    fade_in_time: f32,
    #[serde(rename = "Groups", default)]
    groups: Vec<PoseGroup>,
}
