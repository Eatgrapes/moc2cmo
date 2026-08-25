mod common;
mod deformer;
mod glue;
mod mesh;
mod part;

use crate::{Result, moc3::Moc3Model};

use super::{super::geometry::EvaluatedGeometry, plan::ProjectPlan, writer::XmlWriter};

pub(super) fn write_shared(
    xml: &mut XmlWriter,
    plan: &ProjectPlan,
    model: &Moc3Model,
    geometry: &EvaluatedGeometry,
) -> Result<()> {
    common::write_identifiers(xml, plan, model);
    part::write_parts(xml, plan, model)?;
    deformer::write_deformers(xml, plan, model)?;
    mesh::write_meshes(xml, plan, model, geometry)?;
    glue::write_glues(xml, plan, model)?;
    Ok(())
}
