use serde::Deserialize;

/// Metadata from a Cubism motion file.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct MotionMeta {
    #[serde(rename = "Duration")]
    pub(crate) duration: f32,
    #[serde(rename = "Fps")]
    pub(crate) fps: f32,
    #[serde(rename = "Loop")]
    pub(crate) loop_motion: bool,
    #[serde(rename = "AreBeziersRestricted", default)]
    pub(crate) are_beziers_restricted: bool,
    #[serde(rename = "CurveCount", default)]
    pub(crate) curve_count: u32,
    #[serde(rename = "TotalSegmentCount", default)]
    pub(crate) total_segment_count: u32,
    #[serde(rename = "TotalPointCount", default)]
    pub(crate) total_point_count: u32,
    #[serde(rename = "UserDataCount", default)]
    pub(crate) user_data_count: u32,
    #[serde(rename = "TotalUserDataSize", default)]
    pub(crate) total_user_data_size: u32,
}

impl MotionMeta {
    /// Returns the motion duration in seconds.
    pub fn duration(&self) -> f32 {
        self.duration
    }
    /// Returns the motion frame rate.
    pub fn fps(&self) -> f32 {
        self.fps
    }
    /// Returns whether the motion loops.
    pub fn is_looping(&self) -> bool {
        self.loop_motion
    }
    /// Returns whether Bezier time coordinates use restricted semantics.
    pub fn are_beziers_restricted(&self) -> bool {
        self.are_beziers_restricted
    }
    /// Returns the curve count reported by the file.
    pub fn curve_count(&self) -> u32 {
        self.curve_count
    }
    /// Returns the total segment count reported by the file.
    pub fn total_segment_count(&self) -> u32 {
        self.total_segment_count
    }
    /// Returns the total point count reported by the file.
    pub fn total_point_count(&self) -> u32 {
        self.total_point_count
    }
    /// Returns the user-data event count reported by the file.
    pub fn user_data_count(&self) -> u32 {
        self.user_data_count
    }
    /// Returns the user-data byte size reported by the file.
    pub fn total_user_data_size(&self) -> u32 {
        self.total_user_data_size
    }
}

/// A point on a motion curve.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MotionPoint {
    /// Time in seconds.
    pub time: f32,
    /// Value at the point.
    pub value: f32,
}

/// A timed user-data event embedded in a motion.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct MotionUserData {
    /// Event time in seconds.
    #[serde(rename = "Time")]
    pub time: f32,
    /// Event payload.
    #[serde(rename = "Value")]
    pub value: String,
}

/// One animated target curve.
#[derive(Debug, Clone, PartialEq)]
pub struct MotionCurve {
    pub(crate) target: String,
    pub(crate) id: String,
    pub(crate) first_point: MotionPoint,
    pub(crate) segments: Vec<MotionSegment>,
    pub(crate) fade_in_time: Option<f32>,
    pub(crate) fade_out_time: Option<f32>,
}

impl MotionCurve {
    /// Returns the curve target, such as `Parameter` or `PartOpacity`.
    pub fn target(&self) -> &str {
        &self.target
    }
    /// Returns the target parameter or part identifier.
    pub fn id(&self) -> &str {
        &self.id
    }
    /// Returns the first point.
    pub fn first_point(&self) -> MotionPoint {
        self.first_point
    }
    /// Returns all interpolation segments.
    pub fn segments(&self) -> &[MotionSegment] {
        &self.segments
    }
    /// Returns the optional curve fade-in override.
    pub fn fade_in_time(&self) -> Option<f32> {
        self.fade_in_time
    }
    /// Returns the optional curve fade-out override.
    pub fn fade_out_time(&self) -> Option<f32> {
        self.fade_out_time
    }
}

/// Interpolation mode between two motion points.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MotionSegment {
    /// Linear interpolation.
    Linear {
        /// Start point.
        start: MotionPoint,
        /// End point.
        end: MotionPoint,
    },
    /// Cubic Bezier interpolation.
    Bezier {
        /// Start point.
        start: MotionPoint,
        /// First control point.
        control1: MotionPoint,
        /// Second control point.
        control2: MotionPoint,
        /// End point.
        end: MotionPoint,
    },
    /// Holds the start value until the end point.
    Stepped {
        /// Start point.
        start: MotionPoint,
        /// End point.
        end: MotionPoint,
    },
    /// Holds the end value for the segment.
    InverseStepped {
        /// Start point.
        start: MotionPoint,
        /// End point.
        end: MotionPoint,
    },
}

impl MotionSegment {
    /// Returns the segment's end point.
    pub fn end(&self) -> MotionPoint {
        match *self {
            Self::Linear { end, .. }
            | Self::Bezier { end, .. }
            | Self::Stepped { end, .. }
            | Self::InverseStepped { end, .. } => end,
        }
    }
}
