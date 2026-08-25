use serde::{Deserialize, Serialize};

use crate::Result;

use super::layout::{
    Counts, OFFSET_COUNT, checked_range, checked_scaled_range, invalid, nonnegative,
    optional_unbounded_index, read_colors, read_f32, read_f32_or, read_i32, read_i32_or,
};
use crate::moc3::reader::Reader;

/// A keyed warp-deformer state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WarpDeformerKeyform {
    positions: Vec<[f32; 2]>,
    opacity: f32,
    multiply_color: [f32; 3],
    screen_color: [f32; 3],
}

impl WarpDeformerKeyform {
    /// Returns the keyed lattice positions.
    pub fn positions(&self) -> &[[f32; 2]] {
        &self.positions
    }

    /// Returns the keyed opacity.
    pub fn opacity(&self) -> f32 {
        self.opacity
    }

    /// Returns the keyed multiply color.
    pub fn multiply_color(&self) -> [f32; 3] {
        self.multiply_color
    }

    /// Returns the keyed screen color.
    pub fn screen_color(&self) -> [f32; 3] {
        self.screen_color
    }
}

/// A warp deformer recovered from the model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WarpDeformer {
    parent_deformer_index: Option<usize>,
    binding_band_index: Option<usize>,
    rows: usize,
    columns: usize,
    keyforms: Vec<WarpDeformerKeyform>,
}

impl WarpDeformer {
    /// Returns the parent deformer index.
    pub fn parent_deformer_index(&self) -> Option<usize> {
        self.parent_deformer_index
    }

    /// Returns the parameter binding band used by the keyforms.
    pub fn binding_band_index(&self) -> Option<usize> {
        self.binding_band_index
    }

    /// Returns the number of lattice rows.
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// Returns the number of lattice columns.
    pub fn columns(&self) -> usize {
        self.columns
    }

    /// Returns the keyed warp states in grid order.
    pub fn keyforms(&self) -> &[WarpDeformerKeyform] {
        &self.keyforms
    }
}

/// A keyed rotation-deformer state.
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct RotationDeformerKeyform {
    angle_degrees: f32,
    origin: [f32; 2],
    scale: f32,
    reflected: [bool; 2],
    opacity: f32,
    multiply_color: [f32; 3],
    screen_color: [f32; 3],
}

impl RotationDeformerKeyform {
    /// Returns the keyed rotation angle in degrees.
    pub fn angle_degrees(&self) -> f32 {
        self.angle_degrees
    }

    /// Returns the keyed origin in the parent deformer coordinate system.
    pub fn origin(&self) -> [f32; 2] {
        self.origin
    }

    /// Returns the keyed scale.
    pub fn scale(&self) -> f32 {
        self.scale
    }

    /// Returns horizontal and vertical reflection flags.
    pub fn reflected(&self) -> [bool; 2] {
        self.reflected
    }

    /// Returns the keyed opacity.
    pub fn opacity(&self) -> f32 {
        self.opacity
    }

    /// Returns the keyed multiply color.
    pub fn multiply_color(&self) -> [f32; 3] {
        self.multiply_color
    }

    /// Returns the keyed screen color.
    pub fn screen_color(&self) -> [f32; 3] {
        self.screen_color
    }
}

/// A rotation deformer recovered from the model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RotationDeformer {
    parent_deformer_index: Option<usize>,
    binding_band_index: Option<usize>,
    base_angle_degrees: f32,
    keyforms: Vec<RotationDeformerKeyform>,
}

impl RotationDeformer {
    /// Returns the parent deformer index.
    pub fn parent_deformer_index(&self) -> Option<usize> {
        self.parent_deformer_index
    }

    /// Returns the parameter binding band used by the keyforms.
    pub fn binding_band_index(&self) -> Option<usize> {
        self.binding_band_index
    }

    /// Returns the unkeyed base angle in degrees.
    pub fn base_angle_degrees(&self) -> f32 {
        self.base_angle_degrees
    }

    /// Returns the keyed rotation states in grid order.
    pub fn keyforms(&self) -> &[RotationDeformerKeyform] {
        &self.keyforms
    }
}

/// A recovered warp or rotation deformer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Deformer {
    /// A lattice-based warp deformer.
    Warp(WarpDeformer),
    /// A rotation and scale deformer.
    Rotation(RotationDeformer),
}

impl Deformer {
    /// Returns the parent deformer index.
    pub fn parent_deformer_index(&self) -> Option<usize> {
        match self {
            Self::Warp(deformer) => deformer.parent_deformer_index(),
            Self::Rotation(deformer) => deformer.parent_deformer_index(),
        }
    }

    /// Returns the parameter binding band used by this deformer.
    pub fn binding_band_index(&self) -> Option<usize> {
        match self {
            Self::Warp(deformer) => deformer.binding_band_index(),
            Self::Rotation(deformer) => deformer.binding_band_index(),
        }
    }
}

pub(super) fn parse_deformers(
    reader: &Reader<'_>,
    offsets: &[usize; OFFSET_COUNT],
    counts: &Counts,
) -> Result<Vec<Deformer>> {
    let parents = read_i32(reader, offsets, 16, counts.deformers)?;
    let kinds = read_i32(reader, offsets, 17, counts.deformers)?;
    let specific_indices = read_i32(reader, offsets, 18, counts.deformers)?;
    let warp_bands = read_i32(reader, offsets, 19, counts.warp_deformers)?;
    let warp_begins = read_i32(reader, offsets, 20, counts.warp_deformers)?;
    let warp_counts = read_i32(reader, offsets, 21, counts.warp_deformers)?;
    let warp_vertices = read_i32(reader, offsets, 22, counts.warp_deformers)?;
    let warp_rows = read_i32(reader, offsets, 23, counts.warp_deformers)?;
    let warp_columns = read_i32(reader, offsets, 24, counts.warp_deformers)?;
    let rotation_bands = read_i32(reader, offsets, 25, counts.rotation_deformers)?;
    let rotation_begins = read_i32(reader, offsets, 26, counts.rotation_deformers)?;
    let rotation_counts = read_i32(reader, offsets, 27, counts.rotation_deformers)?;
    let rotation_base_angles = read_f32(reader, offsets, 28, counts.rotation_deformers)?;
    let warp_opacities = read_f32_or(reader, offsets, 59, counts.warp_deformer_keyforms, 1.0)?;
    let warp_position_begins = read_i32(reader, offsets, 60, counts.warp_deformer_keyforms)?;
    let rotation_opacities =
        read_f32_or(reader, offsets, 61, counts.rotation_deformer_keyforms, 1.0)?;
    let rotation_angles = read_f32(reader, offsets, 62, counts.rotation_deformer_keyforms)?;
    let rotation_x = read_f32(reader, offsets, 63, counts.rotation_deformer_keyforms)?;
    let rotation_y = read_f32(reader, offsets, 64, counts.rotation_deformer_keyforms)?;
    let rotation_scales = read_f32(reader, offsets, 65, counts.rotation_deformer_keyforms)?;
    let rotation_reflect_x = read_i32(reader, offsets, 66, counts.rotation_deformer_keyforms)?;
    let rotation_reflect_y = read_i32(reader, offsets, 67, counts.rotation_deformer_keyforms)?;
    let positions = read_f32(reader, offsets, 71, counts.keyform_positions)?;
    let warp_color_begins = read_i32_or(reader, offsets, 105, counts.warp_deformers, -1)?;
    let rotation_color_begins = read_i32_or(reader, offsets, 106, counts.rotation_deformers, -1)?;
    let multiply = read_colors(
        reader,
        offsets,
        [108, 109, 110],
        counts.keyform_multiply_colors,
        [1.0; 3],
    )?;
    let screen = read_colors(
        reader,
        offsets,
        [111, 112, 113],
        counts.keyform_screen_colors,
        [0.0; 3],
    )?;

    (0..counts.deformers)
        .map(|index| {
            let parent = optional_unbounded_index(parents[index])?;
            let specific = nonnegative(specific_indices[index], "deformer specific index")?;
            match kinds[index] {
                0 => parse_warp(
                    specific,
                    parent,
                    &warp_bands,
                    &warp_begins,
                    &warp_counts,
                    &warp_vertices,
                    &warp_rows,
                    &warp_columns,
                    &warp_opacities,
                    &warp_position_begins,
                    &positions,
                    &warp_color_begins,
                    &multiply,
                    &screen,
                )
                .map(Deformer::Warp),
                1 => parse_rotation(
                    specific,
                    parent,
                    &rotation_bands,
                    &rotation_begins,
                    &rotation_counts,
                    &rotation_base_angles,
                    &rotation_opacities,
                    &rotation_angles,
                    &rotation_x,
                    &rotation_y,
                    &rotation_scales,
                    &rotation_reflect_x,
                    &rotation_reflect_y,
                    &rotation_color_begins,
                    &multiply,
                    &screen,
                )
                .map(Deformer::Rotation),
                kind => Err(invalid(format!("unsupported deformer kind {kind}"))),
            }
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn parse_warp(
    index: usize,
    parent: Option<usize>,
    bands: &[i32],
    begins: &[i32],
    counts: &[i32],
    vertex_counts: &[i32],
    rows: &[i32],
    columns: &[i32],
    opacities: &[f32],
    position_begins: &[i32],
    positions: &[f32],
    color_begins: &[i32],
    multiply: &[[f32; 3]],
    screen: &[[f32; 3]],
) -> Result<WarpDeformer> {
    let keyform_range = checked_range(
        *begins
            .get(index)
            .ok_or_else(|| invalid("warp index is outside its table"))?,
        *counts
            .get(index)
            .ok_or_else(|| invalid("warp index is outside its table"))?,
        opacities.len(),
        "warp keyforms",
    )?;
    let vertex_count = nonnegative(vertex_counts[index], "warp vertex count")?;
    let color_begin = color_begins[index];
    let mut keyforms = Vec::with_capacity(keyform_range.len());
    for (local_index, keyform_index) in keyform_range.enumerate() {
        let position_range = checked_scaled_range(
            position_begins[keyform_index],
            vertex_count,
            2,
            positions.len(),
            "warp positions",
        )?;
        let color_index = color_index(color_begin, local_index)?;
        keyforms.push(WarpDeformerKeyform {
            positions: positions[position_range]
                .chunks_exact(2)
                .map(|xy| [xy[0], xy[1]])
                .collect(),
            opacity: opacities[keyform_index],
            multiply_color: color_index
                .and_then(|index| multiply.get(index).copied())
                .unwrap_or([1.0; 3]),
            screen_color: color_index
                .and_then(|index| screen.get(index).copied())
                .unwrap_or([0.0; 3]),
        });
    }
    Ok(WarpDeformer {
        parent_deformer_index: parent,
        binding_band_index: optional_unbounded_index(bands[index])?,
        rows: nonnegative(rows[index], "warp rows")?,
        columns: nonnegative(columns[index], "warp columns")?,
        keyforms,
    })
}

#[allow(clippy::too_many_arguments)]
fn parse_rotation(
    index: usize,
    parent: Option<usize>,
    bands: &[i32],
    begins: &[i32],
    counts: &[i32],
    base_angles: &[f32],
    opacities: &[f32],
    angles: &[f32],
    x: &[f32],
    y: &[f32],
    scales: &[f32],
    reflect_x: &[i32],
    reflect_y: &[i32],
    color_begins: &[i32],
    multiply: &[[f32; 3]],
    screen: &[[f32; 3]],
) -> Result<RotationDeformer> {
    let keyform_range = checked_range(
        *begins
            .get(index)
            .ok_or_else(|| invalid("rotation index is outside its table"))?,
        *counts
            .get(index)
            .ok_or_else(|| invalid("rotation index is outside its table"))?,
        angles.len(),
        "rotation keyforms",
    )?;
    let color_begin = color_begins[index];
    let mut keyforms = Vec::with_capacity(keyform_range.len());
    for (local_index, keyform_index) in keyform_range.enumerate() {
        let color_index = color_index(color_begin, local_index)?;
        keyforms.push(RotationDeformerKeyform {
            angle_degrees: angles[keyform_index],
            origin: [x[keyform_index], y[keyform_index]],
            scale: scales[keyform_index],
            reflected: [reflect_x[keyform_index] == 1, reflect_y[keyform_index] == 1],
            opacity: opacities[keyform_index],
            multiply_color: color_index
                .and_then(|index| multiply.get(index).copied())
                .unwrap_or([1.0; 3]),
            screen_color: color_index
                .and_then(|index| screen.get(index).copied())
                .unwrap_or([0.0; 3]),
        });
    }
    Ok(RotationDeformer {
        parent_deformer_index: parent,
        binding_band_index: optional_unbounded_index(bands[index])?,
        base_angle_degrees: base_angles[index],
        keyforms,
    })
}

fn color_index(begin: i32, local_index: usize) -> Result<Option<usize>> {
    if begin < 0 {
        return Ok(None);
    }
    usize::try_from(begin)
        .map_err(|_| invalid("color index is too large"))?
        .checked_add(local_index)
        .map(Some)
        .ok_or_else(|| invalid("color index overflows"))
}
