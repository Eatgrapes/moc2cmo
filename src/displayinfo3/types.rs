use serde::Deserialize;

/// A named parameter shown by the Cubism editor.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct DisplayInfo3Parameter {
    /// Parameter identifier.
    #[serde(rename = "Id")]
    pub id: String,
    /// Optional parameter group identifier.
    #[serde(rename = "GroupId", default)]
    pub group_id: String,
    /// Human-readable parameter name.
    #[serde(rename = "Name", default)]
    pub name: String,
}

/// A named parameter group shown by the Cubism editor.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct DisplayInfo3ParameterGroup {
    /// Group identifier.
    #[serde(rename = "Id")]
    pub id: String,
    /// Parent group identifier, when present.
    #[serde(rename = "GroupId", default)]
    pub group_id: String,
    /// Human-readable group name.
    #[serde(rename = "Name", default)]
    pub name: String,
}

/// A named part shown by the Cubism editor.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct DisplayInfo3Part {
    /// Part identifier.
    #[serde(rename = "Id")]
    pub id: String,
    /// Human-readable part name.
    #[serde(rename = "Name", default)]
    pub name: String,
}
