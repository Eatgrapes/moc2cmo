mod affine;
mod pose;
mod warp;

use crate::{
    Error, Result,
    moc3::{Deformer, Moc3Model},
};

use super::Texture;

pub(super) use affine::Affine2;

pub(super) struct EvaluatedGeometry {
    meshes: Vec<MeshGeometry>,
}

impl EvaluatedGeometry {
    pub(super) fn evaluate(model: &Moc3Model, textures: &[Texture]) -> Result<Self> {
        let pose = pose::DefaultPose::evaluate(model)?;
        let meshes = model
            .art_meshes()
            .iter()
            .enumerate()
            .map(|(index, mesh)| {
                let positions = pose.mesh_positions(index)?.to_vec();
                let texture = textures.get(mesh.texture_index()).ok_or_else(|| {
                    Error::InvalidCmo3(format!(
                        "ArtMesh {:?} references missing texture {}",
                        mesh.id(),
                        mesh.texture_index()
                    ))
                })?;
                let atlas_to_canvas = affine::fit_page_to_canvas(
                    mesh.uvs(),
                    &positions,
                    texture.width(),
                    texture.height(),
                );
                Ok(MeshGeometry {
                    positions,
                    atlas_to_canvas,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { meshes })
    }

    pub(super) fn mesh(&self, index: usize) -> Result<&MeshGeometry> {
        self.meshes.get(index).ok_or_else(|| {
            Error::InvalidCmo3(format!("evaluated ArtMesh geometry {index} is missing"))
        })
    }
}

pub(super) struct MeshGeometry {
    positions: Vec<[f32; 2]>,
    atlas_to_canvas: Affine2,
}

impl MeshGeometry {
    pub(super) fn positions(&self) -> &[[f32; 2]] {
        &self.positions
    }

    pub(super) fn atlas_to_canvas(&self) -> Affine2 {
        self.atlas_to_canvas
    }
}

pub(super) fn rotation_ancestor_flags(model: &Moc3Model) -> Result<Vec<bool>> {
    (0..model.deformers().len())
        .map(|index| {
            let mut parent = model.deformers()[index].parent_deformer_index();
            for _ in 0..=model.deformers().len() {
                let Some(parent_index) = parent else {
                    return Ok(false);
                };
                let deformer = model.deformers().get(parent_index).ok_or_else(|| {
                    Error::InvalidCmo3(format!(
                        "deformer {index} references missing parent {parent_index}"
                    ))
                })?;
                if matches!(deformer, Deformer::Rotation(_)) {
                    return Ok(true);
                }
                parent = deformer.parent_deformer_index();
            }
            Err(Error::InvalidCmo3(format!(
                "deformer {index} has a cyclic parent chain"
            )))
        })
        .collect()
}
