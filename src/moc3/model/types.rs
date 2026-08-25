use serde::{Deserialize, Serialize};

use crate::Result;

use super::layout::invalid;

/// The byte order used by numeric sections in a MOC3 file.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Endianness {
    /// Least-significant byte first.
    Little,
    /// Most-significant byte first.
    Big,
}

/// A supported MOC3 binary format version.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Moc3Version {
    /// Cubism 3.0 format.
    V3_0,
    /// Cubism 3.3 format.
    V3_3,
    /// Cubism 4.0 format.
    V4_0,
    /// Cubism 4.2 format.
    V4_2,
    /// Cubism 5.0 format.
    V5_0,
    /// Cubism 5.3 format.
    V5_3,
}

impl Moc3Version {
    pub(super) fn from_raw(raw: u8) -> Result<Self> {
        match raw {
            1 => Ok(Self::V3_0),
            2 => Ok(Self::V3_3),
            3 => Ok(Self::V4_0),
            4 => Ok(Self::V4_2),
            5 => Ok(Self::V5_0),
            6 => Ok(Self::V5_3),
            _ => Err(invalid(format!("unsupported format version {raw}"))),
        }
    }

    pub(super) fn count_words(self) -> usize {
        match self {
            Self::V3_0 | Self::V3_3 | Self::V4_0 => 23,
            Self::V4_2 => 32,
            Self::V5_0 | Self::V5_3 => 35,
        }
    }
}

/// The model coordinate system stored in the MOC3 file.
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Canvas {
    pub(super) pixels_per_unit: f32,
    pub(super) origin: [f32; 2],
    pub(super) size: [f32; 2],
    pub(super) reverse_y: bool,
}

impl Canvas {
    /// Returns the scale between model units and pixels.
    pub fn pixels_per_unit(&self) -> f32 {
        self.pixels_per_unit
    }

    /// Returns the canvas origin in pixels.
    pub fn origin(&self) -> [f32; 2] {
        self.origin
    }

    /// Returns the canvas width and height in pixels.
    pub fn size(&self) -> [f32; 2] {
        self.size
    }

    /// Returns whether the Y axis is reversed.
    pub fn reverse_y(&self) -> bool {
        self.reverse_y
    }
}

/// A parameter recovered from a MOC3 model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    pub(super) id: String,
    pub(super) minimum: f32,
    pub(super) maximum: f32,
    pub(super) default: f32,
}

impl Parameter {
    /// Returns the original runtime identifier.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the minimum parameter value.
    pub fn minimum(&self) -> f32 {
        self.minimum
    }

    /// Returns the maximum parameter value.
    pub fn maximum(&self) -> f32 {
        self.maximum
    }

    /// Returns the default parameter value.
    pub fn default(&self) -> f32 {
        self.default
    }
}

/// One parameter axis used by a keyform grid.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterBinding {
    pub(super) parameter_index: usize,
    pub(super) keys: Vec<f32>,
}

impl ParameterBinding {
    /// Returns the index into [`super::Moc3Model::parameters`].
    pub fn parameter_index(&self) -> usize {
        self.parameter_index
    }

    /// Returns the ordered key values on this parameter axis.
    pub fn keys(&self) -> &[f32] {
        &self.keys
    }
}

/// The parameter axes that define an entity's keyform grid.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BindingBand {
    pub(super) bindings: Vec<ParameterBinding>,
}

impl BindingBand {
    /// Returns the parameter axes in grid-stride order.
    pub fn bindings(&self) -> &[ParameterBinding] {
        &self.bindings
    }
}

/// A keyed part state.
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct PartKeyform {
    pub(super) opacity: f32,
    pub(super) draw_order: f32,
}

impl PartKeyform {
    /// Returns the keyed opacity.
    pub fn opacity(&self) -> f32 {
        self.opacity
    }

    /// Returns the keyed draw order.
    pub fn draw_order(&self) -> f32 {
        self.draw_order
    }
}

/// A part in the recovered hierarchy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Part {
    pub(super) id: String,
    pub(super) parent_part_index: Option<usize>,
    pub(super) binding_band_index: Option<usize>,
    pub(super) keyforms: Vec<PartKeyform>,
}

impl Part {
    /// Returns the original runtime identifier.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the parent part index, or `None` for a root part.
    pub fn parent_part_index(&self) -> Option<usize> {
        self.parent_part_index
    }

    /// Returns the parameter binding band used by the keyforms.
    pub fn binding_band_index(&self) -> Option<usize> {
        self.binding_band_index
    }

    /// Returns the part keyforms in grid order.
    pub fn keyforms(&self) -> &[PartKeyform] {
        &self.keyforms
    }
}

/// A keyed drawable state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArtMeshKeyform {
    pub(super) positions: Vec<[f32; 2]>,
    pub(super) opacity: f32,
    pub(super) draw_order: f32,
    pub(super) multiply_color: [f32; 3],
    pub(super) screen_color: [f32; 3],
}

impl ArtMeshKeyform {
    /// Returns keyed positions in the parent deformer coordinate system.
    pub fn positions(&self) -> &[[f32; 2]] {
        &self.positions
    }

    /// Returns the keyed opacity.
    pub fn opacity(&self) -> f32 {
        self.opacity
    }

    /// Returns the keyed draw order.
    pub fn draw_order(&self) -> f32 {
        self.draw_order
    }

    /// Returns the keyed multiply color as RGB values.
    pub fn multiply_color(&self) -> [f32; 3] {
        self.multiply_color
    }

    /// Returns the keyed screen color as RGB values.
    pub fn screen_color(&self) -> [f32; 3] {
        self.screen_color
    }
}

/// An ArtMesh recovered from the runtime model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArtMesh {
    pub(super) id: String,
    pub(super) texture_index: usize,
    pub(super) drawable_flags: u8,
    pub(super) parent_part_index: Option<usize>,
    pub(super) parent_deformer_index: Option<usize>,
    pub(super) binding_band_index: Option<usize>,
    pub(super) uvs: Vec<[f32; 2]>,
    pub(super) triangle_indices: Vec<u16>,
    pub(super) mask_drawable_indices: Vec<usize>,
    pub(super) keyforms: Vec<ArtMeshKeyform>,
}

impl ArtMesh {
    /// Returns the original runtime identifier.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the zero-based texture atlas index.
    pub fn texture_index(&self) -> usize {
        self.texture_index
    }

    /// Returns the original packed drawable flags.
    pub fn drawable_flags(&self) -> u8 {
        self.drawable_flags
    }

    /// Returns the parent part index.
    pub fn parent_part_index(&self) -> Option<usize> {
        self.parent_part_index
    }

    /// Returns the parent deformer index.
    pub fn parent_deformer_index(&self) -> Option<usize> {
        self.parent_deformer_index
    }

    /// Returns the parameter binding band used by the keyforms.
    pub fn binding_band_index(&self) -> Option<usize> {
        self.binding_band_index
    }

    /// Returns the texture UV coordinates.
    pub fn uvs(&self) -> &[[f32; 2]] {
        &self.uvs
    }

    /// Returns triangle vertex indices.
    pub fn triangle_indices(&self) -> &[u16] {
        &self.triangle_indices
    }

    /// Returns the ArtMesh indices used as clipping masks.
    pub fn mask_drawable_indices(&self) -> &[usize] {
        &self.mask_drawable_indices
    }

    /// Returns drawable keyforms in grid order.
    pub fn keyforms(&self) -> &[ArtMeshKeyform] {
        &self.keyforms
    }
}
