use crate::{Result, moc3::Moc3Model};

use super::{
    super::{
        grid::{reference, reference_without_name, start_shared, write_grid},
        plan::{PartPlan, ProjectPlan, RefId},
        writer::{XmlWriter, attr},
    },
    common::{start_parameter_controllable, write_form_header},
};

pub(super) fn write_parts(
    xml: &mut XmlWriter,
    plan: &ProjectPlan,
    model: &Moc3Model,
) -> Result<()> {
    write_grid(
        xml,
        &plan.root_part.grid,
        model,
        None,
        &plan.parameters,
        "__RootPart__",
    )?;
    write_part_source(
        xml,
        plan,
        model,
        &plan.root_part,
        "Root Part",
        "__RootPart__",
        None,
        root_children(plan, model),
        &[],
    );

    for (index, (part, part_plan)) in model.parts().iter().zip(&plan.parts).enumerate() {
        write_grid(
            xml,
            &part_plan.grid,
            model,
            part.binding_band_index(),
            &plan.parameters,
            part.id(),
        )?;
        let parent = part
            .parent_part_index()
            .and_then(|parent| plan.parts.get(parent))
            .map_or(plan.root_part.guid, |parent| parent.guid);
        write_part_source(
            xml,
            plan,
            model,
            part_plan,
            part.id(),
            part.id(),
            Some(parent),
            part_children(plan, model, index),
            part.keyforms(),
        );
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn write_part_source(
    xml: &mut XmlWriter,
    plan: &ProjectPlan,
    _model: &Moc3Model,
    part_plan: &PartPlan,
    local_name: &str,
    id: &str,
    parent: Option<RefId>,
    children: Vec<(&'static str, RefId)>,
    keyforms: &[crate::moc3::PartKeyform],
) {
    start_shared(xml, "CPartSource", part_plan.source);
    start_parameter_controllable(xml, local_name, parent, part_plan.grid.grid);
    xml.empty(
        "carray_list",
        &[attr("xs.n", "_extensions"), attr("count", 0)],
    );
    xml.empty("null", &[attr("xs.n", "internalColor_direct_argb")]);
    xml.end("ACParameterControllableSource");
    reference(xml, "CPartGuid", "guid", part_plan.guid);
    xml.empty("CPartId", &[attr("xs.n", "id"), attr("idstr", id)]);
    xml.text("b", &[attr("xs.n", "enableDrawOrderGroup")], "false");
    xml.text("i", &[attr("xs.n", "defaultOrder_forEditor")], "500");
    xml.text("b", &[attr("xs.n", "isSketch")], "false");
    xml.empty("CColor", &[attr("xs.n", "partsEditColor")]);
    xml.start(
        "carray_list",
        &[attr("xs.n", "_childGuids"), attr("count", children.len())],
    );
    for (tag, child) in children {
        reference_without_name(xml, tag, child);
    }
    xml.end("carray_list");
    reference(
        xml,
        "CDeformerGuid",
        "targetDeformerGuid",
        plan.root_deformer_guid,
    );
    xml.start(
        "carray_list",
        &[
            attr("xs.n", "keyforms"),
            attr("count", part_plan.grid.forms.len()),
        ],
    );
    for (index, form) in part_plan.grid.forms.iter().copied().enumerate() {
        let draw_order = keyforms
            .get(index)
            .map_or(500.0, crate::moc3::PartKeyform::draw_order);
        xml.start("CPartForm", &[]);
        write_form_header(xml, form, "CPartSource", part_plan.source);
        xml.text(
            "i",
            &[attr("xs.n", "drawOrder")],
            &format_integer(draw_order),
        );
        xml.end("CPartForm");
    }
    xml.end("carray_list");
    xml.end("CPartSource");
}

fn root_children(plan: &ProjectPlan, model: &Moc3Model) -> Vec<(&'static str, RefId)> {
    let mut children = Vec::new();
    for (part, part_plan) in model.parts().iter().zip(&plan.parts) {
        if part.parent_part_index().is_none() {
            children.push(("CPartGuid", part_plan.guid));
        }
    }
    for (mesh, mesh_plan) in model.art_meshes().iter().zip(&plan.meshes) {
        if mesh.parent_part_index().is_none() {
            children.push(("CDrawableGuid", mesh_plan.guid));
        }
    }
    children.extend(
        plan.deformers
            .iter()
            .map(|deformer| ("CDeformerGuid", deformer.guid)),
    );
    children.extend(plan.glues.iter().map(|glue| ("CAffecterGuid", glue.guid)));
    children
}

fn part_children(
    plan: &ProjectPlan,
    model: &Moc3Model,
    part_index: usize,
) -> Vec<(&'static str, RefId)> {
    let mut children = Vec::new();
    for (part, part_plan) in model.parts().iter().zip(&plan.parts) {
        if part.parent_part_index() == Some(part_index) {
            children.push(("CPartGuid", part_plan.guid));
        }
    }
    for (mesh, mesh_plan) in model.art_meshes().iter().zip(&plan.meshes) {
        if mesh.parent_part_index() == Some(part_index) {
            children.push(("CDrawableGuid", mesh_plan.guid));
        }
    }
    children
}

fn format_integer(value: f32) -> String {
    if value.is_finite() {
        value.round().to_string()
    } else {
        "500".into()
    }
}
