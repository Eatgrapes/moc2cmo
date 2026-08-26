use serde::Deserialize;
use std::collections::BTreeMap;

/// A model resource manifest.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Model3References {
    /// Relative path to the MOC3 file.
    #[serde(rename = "Moc")]
    pub moc: String,
    /// Relative paths to texture atlas pages.
    #[serde(rename = "Textures", default)]
    pub textures: Vec<String>,
    /// Optional physics3 JSON path.
    #[serde(rename = "Physics", default)]
    pub physics: Option<String>,
    /// Optional pose3 JSON path.
    #[serde(rename = "Pose", default)]
    pub pose: Option<String>,
    /// Optional userdata3 JSON path.
    #[serde(rename = "UserData", default)]
    pub user_data: Option<String>,
    /// Optional display-info JSON path.
    #[serde(rename = "DisplayInfo", default)]
    pub display_info: Option<String>,
    /// Optional expression files.
    #[serde(rename = "Expressions", default)]
    pub expressions: Vec<Model3Expression>,
    /// Motion groups keyed by their caller-facing group name.
    #[serde(rename = "Motions", default)]
    pub motions: BTreeMap<String, Vec<Model3Motion>>,
}

/// An expression file referenced by a model manifest.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Model3Expression {
    /// Expression name.
    #[serde(rename = "Name")]
    pub name: String,
    /// Relative path to the expression JSON file.
    #[serde(rename = "File")]
    pub file: String,
}

/// One motion reference in a model manifest.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Model3Motion {
    /// Relative path to the motion3 JSON file.
    #[serde(rename = "File")]
    pub file: String,
    /// Optional fade-in duration.
    #[serde(rename = "FadeInTime", default)]
    pub fade_in_time: Option<f32>,
    /// Optional fade-out duration.
    #[serde(rename = "FadeOutTime", default)]
    pub fade_out_time: Option<f32>,
}

/// An automatic parameter group such as EyeBlink or LipSync.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Model3Group {
    /// Group target type.
    #[serde(rename = "Target")]
    pub target: String,
    /// Group name.
    #[serde(rename = "Name")]
    pub name: String,
    /// Parameter identifiers in the group.
    #[serde(rename = "Ids", default)]
    pub ids: Vec<String>,
}

/// A named hit-test drawable from a model manifest.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Model3HitArea {
    /// Drawable identifier.
    #[serde(rename = "Id")]
    pub id: String,
    /// Human-readable hit-area name.
    #[serde(rename = "Name")]
    pub name: String,
}
