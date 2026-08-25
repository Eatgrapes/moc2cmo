//! Readable model data recovered from a `.moc3` file.

mod model;
mod reader;

pub use model::{
    ArtMesh, ArtMeshKeyform, BindingBand, Canvas, Endianness, Moc3Model, Moc3Version, Parameter,
    ParameterBinding, Part, PartKeyform,
};
