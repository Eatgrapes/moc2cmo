use serde::Deserialize;

use super::types::ExpressionParameter;
use crate::{Error, Result};

const FORMAT: &str = "exp3.json";

/// A parsed Cubism expression file.
#[derive(Debug, Clone, PartialEq)]
pub struct Expression3 {
    parameters: Vec<ExpressionParameter>,
}

impl Expression3 {
    /// Parses an expression from UTF-8 JSON bytes.
    pub fn from_json_bytes(source: &[u8]) -> Result<Self> {
        let source = std::str::from_utf8(source).map_err(|error| Error::InvalidJson {
            format: FORMAT,
            message: error.to_string(),
        })?;
        Self::from_json_str(source)
    }

    /// Parses an expression from UTF-8 JSON.
    pub fn from_json_str(source: &str) -> Result<Self> {
        let raw: RawExpression =
            serde_json::from_str(source).map_err(|error| Error::InvalidJson {
                format: FORMAT,
                message: error.to_string(),
            })?;
        if raw.expression_type != "Live2D Expression" {
            return Err(Error::InvalidJson {
                format: FORMAT,
                message: format!("unexpected expression type {:?}", raw.expression_type),
            });
        }
        Ok(Self {
            parameters: raw.parameters,
        })
    }

    /// Returns all parameter overrides.
    pub fn parameters(&self) -> &[ExpressionParameter] {
        &self.parameters
    }
}

#[derive(Deserialize)]
struct RawExpression {
    #[serde(rename = "Type")]
    expression_type: String,
    #[serde(rename = "Parameters", default)]
    parameters: Vec<ExpressionParameter>,
}
