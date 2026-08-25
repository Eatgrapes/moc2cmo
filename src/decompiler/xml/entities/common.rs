use uuid::Uuid;

use crate::moc3::Moc3Model;

use super::super::{
    constants::{DEFORMER_ROOT_UUID, PARAMETER_GROUP_ROOT_UUID},
    grid::{empty_shared, reference, start_shared},
    plan::{GridPlan, ProjectPlan, RefId},
    writer::{XmlWriter, attr},
};

pub(super) fn write_identifiers(xml: &mut XmlWriter, plan: &ProjectPlan, model: &Moc3Model) {
    guid(
        xml,
        "CParameterGroupGuid",
        plan.parameter_group_guid,
        PARAMETER_GROUP_ROOT_UUID,
        "Parameter root",
    );
    guid(
        xml,
        "CModelGuid",
        plan.model_guid,
        &Uuid::new_v4().to_string(),
        "Decompiled model",
    );
    guid(
        xml,
        "CDeformerGuid",
        plan.root_deformer_guid,
        DEFORMER_ROOT_UUID,
        "ROOT",
    );

    start_shared(xml, "CoordType", plan.coord_type);
    xml.text("s", &[attr("xs.n", "coordName")], "DeformerLocal");
    xml.end("CoordType");

    start_shared(xml, "CBlend_Normal", plan.blend_normal);
    xml.start("ACBlend", &[attr("xs.n", "super")]);
    xml.text("s", &[attr("xs.n", "displayName")], "Normal");
    xml.end("ACBlend");
    xml.end("CBlend_Normal");

    for (parameter, parameter_plan) in model.parameters().iter().zip(&plan.parameters) {
        random_guid(xml, "CParameterGuid", parameter_plan.guid, parameter.id());
    }

    random_guid(xml, "CPartGuid", plan.root_part.guid, "__RootPart__");
    write_form_guids(xml, &plan.root_part.grid, "Root Part");
    for (part, part_plan) in model.parts().iter().zip(&plan.parts) {
        random_guid(xml, "CPartGuid", part_plan.guid, part.id());
        write_form_guids(xml, &part_plan.grid, part.id());
    }
    for (index, deformer_plan) in plan.deformers.iter().enumerate() {
        random_guid(
            xml,
            "CDeformerGuid",
            deformer_plan.guid,
            &format!("Deformer{index}"),
        );
        write_form_guids(xml, &deformer_plan.grid, &format!("Deformer{index}"));
    }
    for (mesh, mesh_plan) in model.art_meshes().iter().zip(&plan.meshes) {
        random_guid(xml, "CDrawableGuid", mesh_plan.guid, mesh.id());
        random_guid(
            xml,
            "GEditableMeshGuid",
            mesh_plan.editable_mesh_guid,
            mesh.id(),
        );
        random_guid(
            xml,
            "CExtensionGuid",
            mesh_plan.editable_extension_guid,
            &format!("{} editable mesh", mesh.id()),
        );
        random_guid(
            xml,
            "CExtensionGuid",
            mesh_plan.generator_extension_guid,
            &format!("{} mesh generator", mesh.id()),
        );
        random_guid(
            xml,
            "CExtensionGuid",
            mesh_plan.texture_extension_guid,
            &format!("{} texture input", mesh.id()),
        );
        write_form_guids(xml, &mesh_plan.grid, mesh.id());
    }
    for (index, glue_plan) in plan.glues.iter().enumerate() {
        random_guid(
            xml,
            "CAffecterGuid",
            glue_plan.guid,
            &format!("Glue{index}"),
        );
        write_form_guids(xml, &glue_plan.grid, &format!("Glue{index}"));
    }
}

pub(super) fn start_parameter_controllable(
    xml: &mut XmlWriter,
    local_name: &str,
    parent_part: Option<RefId>,
    grid: RefId,
) {
    xml.start("ACParameterControllableSource", &[attr("xs.n", "super")]);
    xml.text("s", &[attr("xs.n", "localName")], local_name);
    xml.text("b", &[attr("xs.n", "isVisible")], "true");
    xml.text("b", &[attr("xs.n", "isLocked")], "false");
    if let Some(parent_part) = parent_part {
        reference(xml, "CPartGuid", "parentGuid", parent_part);
    } else {
        xml.empty("null", &[attr("xs.n", "parentGuid")]);
    }
    reference(xml, "KeyformGridSource", "keyformGridSource", grid);
    xml.start(
        "KeyFormMorphTargetSet",
        &[attr("xs.n", "keyformMorphTargetSet")],
    );
    xml.empty(
        "carray_list",
        &[attr("xs.n", "_morphTargets"), attr("count", 0)],
    );
    xml.start(
        "MorphTargetBlendWeightConstraintSet",
        &[attr("xs.n", "blendWeightConstraintSet")],
    );
    xml.empty(
        "carray_list",
        &[attr("xs.n", "_constraints"), attr("count", 0)],
    );
    xml.end("MorphTargetBlendWeightConstraintSet");
    xml.end("KeyFormMorphTargetSet");
}

pub(super) fn write_form_header(
    xml: &mut XmlWriter,
    form_guid: RefId,
    source_tag: &str,
    source: RefId,
) {
    xml.start("ACForm", &[attr("xs.n", "super")]);
    reference(xml, "CFormGuid", "guid", form_guid);
    xml.text("b", &[attr("xs.n", "isAnimatedForm")], "false");
    xml.text("b", &[attr("xs.n", "isLocalAnimatedForm")], "false");
    reference(xml, source_tag, "_source", source);
    xml.empty("null", &[attr("xs.n", "name")]);
    xml.empty("s", &[attr("xs.n", "notes")]);
    xml.end("ACForm");
}

pub(super) fn color(xml: &mut XmlWriter, name: &str, rgb: [f32; 3]) {
    xml.empty(
        "CFloatColor",
        &[
            attr("xs.n", name),
            attr("red", super::super::grid::float(rgb[0])),
            attr("green", super::super::grid::float(rgb[1])),
            attr("blue", super::super::grid::float(rgb[2])),
            attr("alpha", "1.0"),
        ],
    );
}

fn write_form_guids(xml: &mut XmlWriter, grid: &GridPlan, owner: &str) {
    for (index, form) in grid.forms.iter().copied().enumerate() {
        random_guid(xml, "CFormGuid", form, &format!("{owner} keyform {index}"));
    }
}

fn random_guid(xml: &mut XmlWriter, tag: &str, id: RefId, note: &str) {
    guid(xml, tag, id, &Uuid::new_v4().to_string(), note);
}

fn guid(xml: &mut XmlWriter, tag: &str, id: RefId, uuid: &str, note: &str) {
    empty_shared(xml, tag, id, vec![attr("uuid", uuid), attr("note", note)]);
}
