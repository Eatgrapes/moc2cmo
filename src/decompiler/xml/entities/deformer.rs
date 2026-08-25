use crate::{
    Result,
    moc3::{Deformer, Moc3Model, RotationDeformer, WarpDeformer},
};

use super::{
    super::{
        grid::{float, float_array, reference, start_shared, write_grid},
        plan::{DeformerPlan, ProjectPlan},
        writer::{XmlWriter, attr},
    },
    common::{color, start_parameter_controllable, write_form_header},
};

pub(super) fn write_deformers(
    xml: &mut XmlWriter,
    plan: &ProjectPlan,
    model: &Moc3Model,
) -> Result<()> {
    for (index, (deformer, deformer_plan)) in
        model.deformers().iter().zip(&plan.deformers).enumerate()
    {
        write_grid(
            xml,
            &deformer_plan.grid,
            model,
            deformer.binding_band_index(),
            &plan.parameters,
            &format!("Deformer{index}"),
        )?;
        match deformer {
            Deformer::Warp(warp) => write_warp(xml, plan, deformer_plan, warp, index),
            Deformer::Rotation(rotation) => {
                write_rotation(xml, plan, deformer_plan, rotation, index)
            }
        }
    }
    Ok(())
}

fn write_warp(
    xml: &mut XmlWriter,
    project: &ProjectPlan,
    plan: &DeformerPlan,
    deformer: &WarpDeformer,
    index: usize,
) {
    let name = format!("Warp{index}");
    start_shared(xml, "CWarpDeformerSource", plan.source);
    start_deformer_source(xml, project, plan, deformer.parent_deformer_index(), &name);
    xml.text("i", &[attr("xs.n", "col")], &deformer.columns().to_string());
    xml.text("i", &[attr("xs.n", "row")], &deformer.rows().to_string());
    xml.text("b", &[attr("xs.n", "isQuadTransform")], "false");
    xml.start(
        "carray_list",
        &[
            attr("xs.n", "keyforms"),
            attr("count", plan.grid.forms.len()),
        ],
    );
    for (form_index, form_guid) in plan.grid.forms.iter().copied().enumerate() {
        let keyform = deformer.keyforms().get(form_index);
        xml.start("CWarpDeformerForm", &[]);
        xml.start("ACDeformerForm", &[attr("xs.n", "super")]);
        write_form_header(xml, form_guid, "CWarpDeformerSource", plan.source);
        xml.text(
            "f",
            &[attr("xs.n", "opacity")],
            &float(keyform.map_or(1.0, |value| value.opacity())),
        );
        color(
            xml,
            "multiplyColor",
            keyform.map_or([1.0; 3], |value| value.multiply_color()),
        );
        color(
            xml,
            "screenColor",
            keyform.map_or([0.0; 3], |value| value.screen_color()),
        );
        reference(xml, "CoordType", "coordType", project.coord_type);
        xml.end("ACDeformerForm");
        let positions = keyform
            .map(|value| value.positions())
            .unwrap_or(&[])
            .iter()
            .flat_map(|position| position.iter().copied());
        xml.text(
            "float-array",
            &[
                attr("xs.n", "positions"),
                attr(
                    "count",
                    keyform.map_or(0, |value| value.positions().len() * 2),
                ),
            ],
            &float_array(positions),
        );
        xml.end("CWarpDeformerForm");
    }
    xml.end("carray_list");
    xml.end("CWarpDeformerSource");
}

fn write_rotation(
    xml: &mut XmlWriter,
    project: &ProjectPlan,
    plan: &DeformerPlan,
    deformer: &RotationDeformer,
    index: usize,
) {
    let name = format!("Rotation{index}");
    start_shared(xml, "CRotationDeformerSource", plan.source);
    start_deformer_source(xml, project, plan, deformer.parent_deformer_index(), &name);
    xml.text("b", &[attr("xs.n", "useBoneUi_testImpl")], "true");
    xml.start(
        "carray_list",
        &[
            attr("xs.n", "keyforms"),
            attr("count", plan.grid.forms.len()),
        ],
    );
    for (form_index, form_guid) in plan.grid.forms.iter().copied().enumerate() {
        let keyform = deformer.keyforms().get(form_index);
        let origin = keyform.map_or([0.0; 2], |value| value.origin());
        let reflected = keyform.map_or([false; 2], |value| value.reflected());
        xml.start(
            "CRotationDeformerForm",
            &[
                attr(
                    "angle",
                    float(keyform.map_or(0.0, |value| value.angle_degrees())),
                ),
                attr("originX", float(origin[0])),
                attr("originY", float(origin[1])),
                attr("scale", float(keyform.map_or(1.0, |value| value.scale()))),
                attr("isReflectX", reflected[0]),
                attr("isReflectY", reflected[1]),
            ],
        );
        xml.start("ACDeformerForm", &[attr("xs.n", "super")]);
        write_form_header(xml, form_guid, "CRotationDeformerSource", plan.source);
        xml.text(
            "f",
            &[attr("xs.n", "opacity")],
            &float(keyform.map_or(1.0, |value| value.opacity())),
        );
        color(
            xml,
            "multiplyColor",
            keyform.map_or([1.0; 3], |value| value.multiply_color()),
        );
        color(
            xml,
            "screenColor",
            keyform.map_or([0.0; 3], |value| value.screen_color()),
        );
        reference(xml, "CoordType", "coordType", project.coord_type);
        xml.end("ACDeformerForm");
        xml.end("CRotationDeformerForm");
    }
    xml.end("carray_list");
    xml.text("f", &[attr("xs.n", "handleLengthOnCanvas")], "200.0");
    xml.text("f", &[attr("xs.n", "circleRadiusOnCanvas")], "100.0");
    xml.text(
        "f",
        &[attr("xs.n", "baseAngle")],
        &float(deformer.base_angle_degrees()),
    );
    xml.end("CRotationDeformerSource");
}

fn start_deformer_source(
    xml: &mut XmlWriter,
    project: &ProjectPlan,
    plan: &DeformerPlan,
    parent_deformer: Option<usize>,
    id: &str,
) {
    xml.start("ACDeformerSource", &[attr("xs.n", "super")]);
    start_parameter_controllable(xml, id, Some(project.root_part.guid), plan.grid.grid);
    xml.empty(
        "carray_list",
        &[attr("xs.n", "_extensions"), attr("count", 0)],
    );
    xml.empty("null", &[attr("xs.n", "internalColor_direct_argb")]);
    xml.empty("null", &[attr("xs.n", "internalColor_indirect_argb")]);
    xml.end("ACParameterControllableSource");
    reference(xml, "CDeformerGuid", "guid", plan.guid);
    xml.empty("CDeformerId", &[attr("xs.n", "id"), attr("idstr", id)]);
    let target = parent_deformer
        .and_then(|index| project.deformers.get(index))
        .map_or(project.root_deformer_guid, |parent| parent.guid);
    reference(xml, "CDeformerGuid", "targetDeformerGuid", target);
    xml.end("ACDeformerSource");
}
