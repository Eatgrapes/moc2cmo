use serde::Deserialize;

use super::types::{MotionCurve, MotionMeta, MotionPoint, MotionSegment};
use crate::{Error, Result};

const FORMAT: &str = "motion3.json";

/// A parsed Cubism `motion3.json` document.
#[derive(Debug, Clone, PartialEq)]
pub struct Motion3 {
    version: u32,
    meta: MotionMeta,
    curves: Vec<MotionCurve>,
}

impl Motion3 {
    /// Parses a motion document from UTF-8 JSON bytes.
    pub fn from_json_bytes(source: &[u8]) -> Result<Self> {
        let source = std::str::from_utf8(source).map_err(|error| Error::InvalidJson {
            format: FORMAT,
            message: error.to_string(),
        })?;
        Self::from_json_str(source)
    }

    /// Parses a motion document from UTF-8 JSON.
    pub fn from_json_str(source: &str) -> Result<Self> {
        let raw: RawMotion3 = serde_json::from_str(source).map_err(|error| Error::InvalidJson {
            format: FORMAT,
            message: error.to_string(),
        })?;
        if raw.version != 3 {
            return Err(Error::UnsupportedVersion {
                format: FORMAT,
                version: raw.version,
            });
        }
        let curves = raw
            .curves
            .into_iter()
            .map(parse_curve)
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            version: raw.version,
            meta: raw.meta,
            curves,
        })
    }
    /// Returns the motion format version.
    pub fn version(&self) -> u32 {
        self.version
    }
    /// Returns motion metadata.
    pub fn meta(&self) -> &MotionMeta {
        &self.meta
    }
    /// Returns all animation curves.
    pub fn curves(&self) -> &[MotionCurve] {
        &self.curves
    }
}

#[derive(Deserialize)]
struct RawMotion3 {
    #[serde(rename = "Version")]
    version: u32,
    #[serde(rename = "Meta")]
    meta: MotionMeta,
    #[serde(rename = "Curves", default)]
    curves: Vec<RawCurve>,
}
#[derive(Deserialize)]
struct RawCurve {
    #[serde(rename = "Target")]
    target: String,
    #[serde(rename = "Id")]
    id: String,
    #[serde(rename = "Segments")]
    segments: Vec<f32>,
    #[serde(rename = "FadeInTime", default)]
    fade_in_time: Option<f32>,
    #[serde(rename = "FadeOutTime", default)]
    fade_out_time: Option<f32>,
}

fn parse_curve(raw: RawCurve) -> Result<MotionCurve> {
    if raw.segments.len() < 2 {
        return Err(invalid("curve has no first point"));
    }
    let first_point = MotionPoint {
        time: raw.segments[0],
        value: raw.segments[1],
    };
    let mut cursor = 2;
    let mut start = first_point;
    let mut segments = Vec::new();
    while cursor < raw.segments.len() {
        let kind = raw.segments[cursor];
        cursor += 1;
        if kind.fract() != 0.0 || !(0.0..=3.0).contains(&kind) {
            return Err(invalid("segment type must be 0, 1, 2, or 3"));
        }
        let segment = match kind as u32 {
            0 => MotionSegment::Linear {
                start,
                end: read_point(&raw.segments, &mut cursor)?,
            },
            1 => MotionSegment::Bezier {
                start,
                control1: read_point(&raw.segments, &mut cursor)?,
                control2: read_point(&raw.segments, &mut cursor)?,
                end: read_point(&raw.segments, &mut cursor)?,
            },
            2 => MotionSegment::Stepped {
                start,
                end: read_point(&raw.segments, &mut cursor)?,
            },
            3 => MotionSegment::InverseStepped {
                start,
                end: read_point(&raw.segments, &mut cursor)?,
            },
            _ => unreachable!(),
        };
        start = segment.end();
        segments.push(segment);
    }
    Ok(MotionCurve {
        target: raw.target,
        id: raw.id,
        first_point,
        segments,
        fade_in_time: raw.fade_in_time,
        fade_out_time: raw.fade_out_time,
    })
}

fn read_point(values: &[f32], cursor: &mut usize) -> Result<MotionPoint> {
    if values.len().saturating_sub(*cursor) < 2 {
        return Err(invalid("segment point is incomplete"));
    }
    let point = MotionPoint {
        time: values[*cursor],
        value: values[*cursor + 1],
    };
    *cursor += 2;
    Ok(point)
}

fn invalid(message: impl Into<String>) -> Error {
    Error::InvalidJson {
        format: FORMAT,
        message: message.into(),
    }
}
