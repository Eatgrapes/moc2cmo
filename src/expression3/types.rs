use serde::Deserialize;

/// How an expression value is combined with the current parameter value.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum ExpressionParameterBlend {
    /// Adds the expression value.
    #[serde(rename = "Add")]
    Add,
    /// Multiplies the current value.
    #[serde(rename = "Multiply")]
    Multiply,
    /// Replaces the current value.
    #[serde(rename = "Overwrite")]
    Overwrite,
}

/// One parameter override in an expression.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ExpressionParameter {
    /// Target parameter identifier.
    #[serde(rename = "Id")]
    pub id: String,
    /// Expression value.
    #[serde(rename = "Value")]
    pub value: f32,
    /// Value blend mode.
    #[serde(rename = "Blend")]
    pub blend: ExpressionParameterBlend,
}
