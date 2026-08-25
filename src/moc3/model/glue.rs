use serde::{Deserialize, Serialize};

use crate::Result;

use super::layout::{
    Counts, OFFSET_COUNT, checked_range, invalid, nonnegative, optional_unbounded_index, read_f32,
    read_i32, read_u16,
};
use crate::moc3::reader::Reader;

/// A paired vertex and its glue weight.
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlueVertex {
    vertex_index: usize,
    weight: f32,
}

impl GlueVertex {
    /// Returns the vertex index in its ArtMesh.
    pub fn vertex_index(&self) -> usize {
        self.vertex_index
    }

    /// Returns the glue influence weight.
    pub fn weight(&self) -> f32 {
        self.weight
    }
}

/// A glue constraint joining vertices from two ArtMeshes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Glue {
    art_mesh_indices: [usize; 2],
    binding_band_index: Option<usize>,
    vertices_a: Vec<GlueVertex>,
    vertices_b: Vec<GlueVertex>,
    intensities: Vec<f32>,
}

impl Glue {
    /// Returns the two joined ArtMesh indices.
    pub fn art_mesh_indices(&self) -> [usize; 2] {
        self.art_mesh_indices
    }

    /// Returns the parameter binding band used by the intensity keyforms.
    pub fn binding_band_index(&self) -> Option<usize> {
        self.binding_band_index
    }

    /// Returns the vertices on the first ArtMesh.
    pub fn vertices_a(&self) -> &[GlueVertex] {
        &self.vertices_a
    }

    /// Returns the paired vertices on the second ArtMesh.
    pub fn vertices_b(&self) -> &[GlueVertex] {
        &self.vertices_b
    }

    /// Returns keyed glue intensities in grid order.
    pub fn intensities(&self) -> &[f32] {
        &self.intensities
    }
}

pub(super) fn parse_glues(
    reader: &Reader<'_>,
    offsets: &[usize; OFFSET_COUNT],
    counts: &Counts,
) -> Result<Vec<Glue>> {
    let bands = read_i32(reader, offsets, 91, counts.glues)?;
    let keyform_begins = read_i32(reader, offsets, 92, counts.glues)?;
    let keyform_counts = read_i32(reader, offsets, 93, counts.glues)?;
    let mesh_a = read_i32(reader, offsets, 94, counts.glues)?;
    let mesh_b = read_i32(reader, offsets, 95, counts.glues)?;
    let vertex_begins = read_i32(reader, offsets, 96, counts.glues)?;
    let vertex_counts = read_i32(reader, offsets, 97, counts.glues)?;
    let weights = read_f32(reader, offsets, 98, counts.glue_vertices)?;
    let vertex_indices = read_u16(reader, offsets, 99, counts.glue_vertices)?;
    let intensities = read_f32(reader, offsets, 100, counts.glue_keyforms)?;

    (0..counts.glues)
        .map(|index| {
            let vertex_range = checked_range(
                vertex_begins[index],
                vertex_counts[index],
                counts.glue_vertices,
                "glue vertices",
            )?;
            if vertex_range.len() % 2 != 0 {
                return Err(invalid("glue vertex table contains an unpaired vertex"));
            }
            let mut vertices_a = Vec::with_capacity(vertex_range.len() / 2);
            let mut vertices_b = Vec::with_capacity(vertex_range.len() / 2);
            for pair in (vertex_range.start..vertex_range.end).step_by(2) {
                vertices_a.push(GlueVertex {
                    vertex_index: usize::from(vertex_indices[pair]),
                    weight: weights[pair],
                });
                vertices_b.push(GlueVertex {
                    vertex_index: usize::from(vertex_indices[pair + 1]),
                    weight: weights[pair + 1],
                });
            }
            let keyform_range = checked_range(
                keyform_begins[index],
                keyform_counts[index],
                counts.glue_keyforms,
                "glue keyforms",
            )?;
            Ok(Glue {
                art_mesh_indices: [
                    nonnegative(mesh_a[index], "glue ArtMesh index")?,
                    nonnegative(mesh_b[index], "glue ArtMesh index")?,
                ],
                binding_band_index: optional_unbounded_index(bands[index])?,
                vertices_a,
                vertices_b,
                intensities: intensities[keyform_range].to_vec(),
            })
        })
        .collect()
}
