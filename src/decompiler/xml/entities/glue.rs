use crate::{
    Error, Result,
    moc3::{Glue, Moc3Model},
};

use super::{
    super::{
        grid::{float, float_array, reference, start_shared, write_grid},
        plan::{GluePlan, ProjectPlan},
        writer::{XmlWriter, attr},
    },
    common::{start_parameter_controllable, write_form_header},
};

pub(super) fn write_glues(
    xml: &mut XmlWriter,
    plan: &ProjectPlan,
    model: &Moc3Model,
) -> Result<()> {
    for (index, (glue, glue_plan)) in model.glues().iter().zip(&plan.glues).enumerate() {
        write_grid(
            xml,
            &glue_plan.grid,
            model,
            glue.binding_band_index(),
            &plan.parameters,
            &format!("Glue{index}"),
        )?;
        write_glue_source(xml, plan, glue_plan, glue, index)?;
    }
    Ok(())
}

fn write_glue_source(
    xml: &mut XmlWriter,
    project: &ProjectPlan,
    plan: &GluePlan,
    glue: &Glue,
    index: usize,
) -> Result<()> {
    let [mesh_a_index, mesh_b_index] = glue.art_mesh_indices();
    let mesh_a = project.meshes.get(mesh_a_index).ok_or_else(|| {
        Error::InvalidCmo3(format!(
            "Glue{index} references missing ArtMesh {mesh_a_index}"
        ))
    })?;
    let mesh_b = project.meshes.get(mesh_b_index).ok_or_else(|| {
        Error::InvalidCmo3(format!(
            "Glue{index} references missing ArtMesh {mesh_b_index}"
        ))
    })?;
    let mut weights = Vec::with_capacity(glue.vertices_a().len() * 2);
    let mut vertex_uids = Vec::with_capacity(glue.vertices_a().len() * 2);
    for (left, right) in glue.vertices_a().iter().zip(glue.vertices_b()) {
        weights.extend([left.weight(), right.weight()]);
        vertex_uids.extend([left.vertex_index(), right.vertex_index()]);
    }

    start_shared(xml, "CGlueSource", plan.source);
    xml.start("ACAffecterSource", &[attr("xs.n", "super")]);
    start_parameter_controllable(
        xml,
        &format!("Glue{index}"),
        Some(project.root_part.guid),
        plan.grid.grid,
    );
    xml.empty(
        "carray_list",
        &[attr("xs.n", "_extensions"), attr("count", 0)],
    );
    xml.end("ACParameterControllableSource");
    reference(xml, "CAffecterGuid", "guid", plan.guid);
    xml.empty(
        "CAffecterId",
        &[
            attr("xs.n", "id"),
            attr(
                "idstr",
                format!("Glue{index}_{mesh_a_index}_{mesh_b_index}"),
            ),
        ],
    );
    reference(
        xml,
        "CDeformerGuid",
        "targetDeformerGuid",
        project.root_deformer_guid,
    );
    xml.text("i", &[attr("xs.n", "editVersion")], "1");
    xml.end("ACAffecterSource");
    xml.text(
        "float-array",
        &[attr("xs.n", "weights"), attr("count", weights.len())],
        &float_array(weights),
    );
    xml.start(
        "carray_list",
        &[
            attr("xs.n", "keyforms"),
            attr("count", plan.grid.forms.len()),
        ],
    );
    for (form_index, form_guid) in plan.grid.forms.iter().copied().enumerate() {
        xml.start("CGlueForm", &[]);
        xml.start("ACAffecterForm", &[attr("xs.n", "super")]);
        write_form_header(xml, form_guid, "CGlueSource", plan.source);
        xml.end("ACAffecterForm");
        xml.text(
            "f",
            &[attr("xs.n", "intensity")],
            &float(glue.intensities().get(form_index).copied().unwrap_or(1.0)),
        );
        xml.end("CGlueForm");
    }
    xml.end("carray_list");
    reference(xml, "CDrawableGuid", "targetArtMeshA_guid", mesh_a.guid);
    reference(xml, "CDrawableGuid", "targetArtMeshB_guid", mesh_b.guid);
    xml.text(
        "long-array",
        &[
            attr("xs.n", "bindVertexUids"),
            attr("count", vertex_uids.len()),
        ],
        &vertex_uids
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" "),
    );
    xml.start("GVector2", &[attr("xs.n", "tabPosOnCanvas")]);
    xml.text("f", &[attr("xs.n", "x")], "0.0");
    xml.text("f", &[attr("xs.n", "y")], "0.0");
    xml.end("GVector2");
    xml.end("CGlueSource");
    Ok(())
}
