use serde::Deserialize;

/// A pose group containing mutually exclusive parts.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(transparent)]
pub struct PoseGroup(
    /// Parts in the group.
    pub Vec<PosePart>,
);

/// One part in a pose group.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PosePart {
    /// Part identifier.
    #[serde(rename = "Id")]
    pub id: String,
    /// Linked part identifiers.
    #[serde(rename = "Link", default)]
    pub link: Vec<String>,
}
