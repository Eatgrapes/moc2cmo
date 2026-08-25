use crate::moc3::{Deformer, Moc3Model};

use super::{
    grid::{float, reference, reference_without_name},
    plan::ProjectPlan,
    writer::{XmlWriter, attr},
};

pub(super) fn write_main(
    xml: &mut XmlWriter,
    plan: &ProjectPlan,
    model: &Moc3Model,
    model_name: &str,
) {
    xml.start("CModelSource", &[attr("isDefaultKeyformLocked", "false")]);
    reference(xml, "CModelGuid", "guid", plan.model_guid);
    xml.text("s", &[attr("xs.n", "name")], model_name);
    xml.start("EditorEdition", &[attr("xs.n", "editorEdition")]);
    xml.text("i", &[attr("xs.n", "edition")], "15");
    xml.end("EditorEdition");
    write_canvas(xml, model);
    write_parameters(xml, plan, model);
    write_texture_manager(xml, plan);

    xml.text(
        "b",
        &[attr("xs.n", "useLegacyDrawOrder__testImpl")],
        "false",
    );
    write_drawable_set(xml, plan);
    write_deformer_set(xml, plan, model);
    write_affecter_set(xml, plan);
    write_part_set(xml, plan);
    reference(xml, "CPartSource", "rootPart", plan.root_part.source);

    xml.start("CParameterGroupSet", &[attr("xs.n", "parameterGroupSet")]);
    xml.empty("carray_list", &[attr("xs.n", "_groups"), attr("count", 0)]);
    xml.end("CParameterGroupSet");
    write_model_info(xml, model);
    xml.text("i", &[attr("xs.n", "targetVersionNo")], "3000");
    xml.text(
        "i",
        &[attr("xs.n", "latestVersionOfLastModelerNo")],
        "5000000",
    );
    xml.end("CModelSource");
}

fn write_canvas(xml: &mut XmlWriter, model: &Moc3Model) {
    let size = model.canvas().size();
    xml.start("CImageCanvas", &[attr("xs.n", "canvas")]);
    xml.text(
        "i",
        &[attr("xs.n", "pixelWidth")],
        &pixel_dimension(size[0]),
    );
    xml.text(
        "i",
        &[attr("xs.n", "pixelHeight")],
        &pixel_dimension(size[1]),
    );
    xml.empty("CColor", &[attr("xs.n", "background")]);
    xml.end("CImageCanvas");
}

fn write_parameters(xml: &mut XmlWriter, plan: &ProjectPlan, model: &Moc3Model) {
    xml.start("CParameterSourceSet", &[attr("xs.n", "parameterSourceSet")]);
    xml.start(
        "carray_list",
        &[
            attr("xs.n", "_sources"),
            attr("count", model.parameters().len()),
        ],
    );
    for (parameter, parameter_plan) in model.parameters().iter().zip(&plan.parameters) {
        xml.start("CParameterSource", &[]);
        xml.text("i", &[attr("xs.n", "decimalPlaces")], "3");
        reference(xml, "CParameterGuid", "guid", parameter_plan.guid);
        xml.text("f", &[attr("xs.n", "snapEpsilon")], "0.001");
        xml.text(
            "f",
            &[attr("xs.n", "minValue")],
            &float(parameter.minimum()),
        );
        xml.text(
            "f",
            &[attr("xs.n", "maxValue")],
            &float(parameter.maximum()),
        );
        xml.text(
            "f",
            &[attr("xs.n", "defaultValue")],
            &float(parameter.default()),
        );
        xml.text("b", &[attr("xs.n", "isRepeat")], "false");
        xml.empty(
            "CParameterId",
            &[attr("xs.n", "id"), attr("idstr", parameter.id())],
        );
        xml.empty("Type", &[attr("xs.n", "paramType"), attr("v", "NORMAL")]);
        xml.text("s", &[attr("xs.n", "name")], parameter.id());
        xml.empty("s", &[attr("xs.n", "description")]);
        xml.text("b", &[attr("xs.n", "combined")], "false");
        reference(
            xml,
            "CParameterGroupGuid",
            "parentGroupGuid",
            plan.parameter_group_guid,
        );
        xml.end("CParameterSource");
    }
    xml.end("carray_list");
    xml.end("CParameterSourceSet");
}

fn write_texture_manager(xml: &mut XmlWriter, plan: &ProjectPlan) {
    xml.start("CTextureManager", &[attr("xs.n", "textureManager")]);
    xml.start("TextureImageGroup", &[attr("xs.n", "textureList")]);
    xml.empty("carray_list", &[attr("xs.n", "children"), attr("count", 0)]);
    xml.end("TextureImageGroup");
    xml.start(
        "carray_list",
        &[attr("xs.n", "_rawImages"), attr("count", 1)],
    );
    xml.start("LayeredImageWrapper", &[]);
    reference(xml, "CLayeredImage", "image", plan.layered_image);
    xml.text("l", &[attr("xs.n", "importedTimeMSec")], "0");
    xml.text("l", &[attr("xs.n", "lastModifiedTimeMSec")], "0");
    xml.text("b", &[attr("xs.n", "isReplaced")], "false");
    xml.end("LayeredImageWrapper");
    xml.end("carray_list");
    xml.start(
        "carray_list",
        &[attr("xs.n", "_modelImageGroups"), attr("count", 1)],
    );
    reference_without_name(xml, "CModelImageGroup", plan.image_group);
    xml.end("carray_list");
    xml.empty(
        "carray_list",
        &[attr("xs.n", "_textureAtlases"), attr("count", 0)],
    );
    xml.text("b", &[attr("xs.n", "isTextureInputModelImageMode")], "true");
    xml.text("i", &[attr("xs.n", "previewReductionRatio")], "1");
    xml.empty(
        "carray_list",
        &[
            attr("xs.n", "artPathBrushUsingLayeredImageIds"),
            attr("count", 0),
        ],
    );
    xml.end("CTextureManager");
}

fn write_drawable_set(xml: &mut XmlWriter, plan: &ProjectPlan) {
    xml.start("CDrawableSourceSet", &[attr("xs.n", "drawableSourceSet")]);
    xml.start(
        "carray_list",
        &[attr("xs.n", "_sources"), attr("count", plan.meshes.len())],
    );
    for mesh in &plan.meshes {
        reference_without_name(xml, "CArtMeshSource", mesh.source);
    }
    xml.end("carray_list");
    xml.end("CDrawableSourceSet");
}

fn write_deformer_set(xml: &mut XmlWriter, plan: &ProjectPlan, model: &Moc3Model) {
    xml.start("CDeformerSourceSet", &[attr("xs.n", "deformerSourceSet")]);
    xml.start(
        "carray_list",
        &[
            attr("xs.n", "_sources"),
            attr("count", plan.deformers.len()),
        ],
    );
    for (deformer, deformer_plan) in model.deformers().iter().zip(&plan.deformers) {
        let tag = match deformer {
            Deformer::Warp(_) => "CWarpDeformerSource",
            Deformer::Rotation(_) => "CRotationDeformerSource",
        };
        reference_without_name(xml, tag, deformer_plan.source);
    }
    xml.end("carray_list");
    xml.end("CDeformerSourceSet");
}

fn write_affecter_set(xml: &mut XmlWriter, plan: &ProjectPlan) {
    xml.start("CAffecterSourceSet", &[attr("xs.n", "affecterSourceSet")]);
    xml.start(
        "carray_list",
        &[attr("xs.n", "_sources"), attr("count", plan.glues.len())],
    );
    for glue in &plan.glues {
        reference_without_name(xml, "CGlueSource", glue.source);
    }
    xml.end("carray_list");
    xml.end("CAffecterSourceSet");
}

fn write_part_set(xml: &mut XmlWriter, plan: &ProjectPlan) {
    xml.start("CPartSourceSet", &[attr("xs.n", "partSourceSet")]);
    xml.start(
        "carray_list",
        &[
            attr("xs.n", "_sources"),
            attr("count", plan.parts.len() + 1),
        ],
    );
    reference_without_name(xml, "CPartSource", plan.root_part.source);
    for part in &plan.parts {
        reference_without_name(xml, "CPartSource", part.source);
    }
    xml.end("carray_list");
    xml.end("CPartSourceSet");
}

fn write_model_info(xml: &mut XmlWriter, model: &Moc3Model) {
    let origin = model.canvas().origin();
    xml.start("CModelInfo", &[attr("xs.n", "modelInfo")]);
    xml.text(
        "f",
        &[attr("xs.n", "pixelsPerUnit")],
        &float(model.canvas().pixels_per_unit()),
    );
    xml.start("CPoint", &[attr("xs.n", "originInPixels")]);
    xml.text("i", &[attr("xs.n", "x")], &pixel_coordinate(origin[0]));
    xml.text("i", &[attr("xs.n", "y")], &pixel_coordinate(origin[1]));
    xml.end("CPoint");
    xml.end("CModelInfo");
}

fn pixel_dimension(value: f32) -> String {
    if value.is_finite() {
        value.abs().round().max(1.0).to_string()
    } else {
        "1".into()
    }
}

fn pixel_coordinate(value: f32) -> String {
    if value.is_finite() {
        value.round().to_string()
    } else {
        "0".into()
    }
}
