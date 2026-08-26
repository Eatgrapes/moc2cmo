use serde::Deserialize;

/// Metadata counters from a userdata document.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
pub struct UserData3Meta {
    /// Number of userdata entries reported by the source.
    #[serde(rename = "UserDataCount", default)]
    pub user_data_count: u32,
    /// Total UTF-8 payload size reported by the source.
    #[serde(rename = "TotalUserDataSize", default)]
    pub total_user_data_size: u32,
}

/// One userdata annotation attached to a model object.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct UserData3Entry {
    /// Target object kind, normally `ArtMesh`.
    #[serde(rename = "Target")]
    pub target: String,
    /// Target object identifier.
    #[serde(rename = "Id")]
    pub id: String,
    /// User-defined annotation text.
    #[serde(rename = "Value")]
    pub value: String,
}
