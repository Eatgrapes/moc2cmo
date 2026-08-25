mod art_mesh;
mod bindings;
mod layout;
mod parts;
mod types;

use serde::{Deserialize, Serialize};

use crate::Result;

use self::{
    art_mesh::parse_art_meshes,
    bindings::parse_bindings,
    layout::{
        parse_canvas, parse_counts, parse_header, parse_ids, parse_offsets, parse_parameters,
    },
    parts::parse_parts,
};
use super::reader::Reader;

pub use types::{
    ArtMesh, ArtMeshKeyform, BindingBand, Canvas, Endianness, Moc3Version, Parameter,
    ParameterBinding, Part, PartKeyform,
};

/// A parsed, caller-readable MOC3 model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Moc3Model {
    version: Moc3Version,
    endianness: Endianness,
    canvas: Canvas,
    parameters: Vec<Parameter>,
    binding_bands: Vec<BindingBand>,
    parts: Vec<Part>,
    art_meshes: Vec<ArtMesh>,
}

impl Moc3Model {
    /// Parses a full MOC3 byte slice.
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let (version, endianness) = parse_header(bytes)?;
        let reader = Reader::new(bytes, endianness);
        let offsets = parse_offsets(&reader)?;
        let counts = parse_counts(&reader, &offsets, version)?;
        let canvas = parse_canvas(&reader, &offsets)?;
        let part_ids = parse_ids(&reader, &offsets, 3, counts.parts)?;
        let art_mesh_ids = parse_ids(&reader, &offsets, 33, counts.art_meshes)?;
        let parameter_ids = parse_ids(&reader, &offsets, 50, counts.parameters)?;

        Ok(Self {
            version,
            endianness,
            canvas,
            parameters: parse_parameters(&reader, &offsets, parameter_ids)?,
            binding_bands: parse_bindings(&reader, &offsets, &counts)?,
            parts: parse_parts(&reader, &offsets, part_ids, &counts)?,
            art_meshes: parse_art_meshes(&reader, &offsets, art_mesh_ids, &counts)?,
        })
    }

    /// Returns the MOC3 format version.
    pub fn version(&self) -> Moc3Version {
        self.version
    }

    /// Returns the numeric byte order.
    pub fn endianness(&self) -> Endianness {
        self.endianness
    }

    /// Returns the model canvas.
    pub fn canvas(&self) -> Canvas {
        self.canvas
    }

    /// Returns the recovered parameters.
    pub fn parameters(&self) -> &[Parameter] {
        &self.parameters
    }

    /// Returns the shared keyform binding bands.
    pub fn binding_bands(&self) -> &[BindingBand] {
        &self.binding_bands
    }

    /// Returns the recovered parts.
    pub fn parts(&self) -> &[Part] {
        &self.parts
    }

    /// Returns the recovered ArtMeshes.
    pub fn art_meshes(&self) -> &[ArtMesh] {
        &self.art_meshes
    }
}
