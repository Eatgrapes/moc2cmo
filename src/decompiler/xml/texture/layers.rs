use uuid::Uuid;

use crate::{decompiler::Texture, moc3::Moc3Model};

use super::super::{
    grid::{reference, reference_without_name, start_shared},
    plan::{PagePlan, ProjectPlan},
    writer::{XmlWriter, attr},
};

pub(super) fn write_layer(
    xml: &mut XmlWriter,
    project: &ProjectPlan,
    page: &PagePlan,
    texture: &Texture,
    index: usize,
) {
    start_shared(xml, "CLayer", page.layer);
    xml.start("ACImageLayer", &[attr("xs.n", "super")]);
    xml.start("ACLayerEntry", &[attr("xs.n", "super")]);
    xml.text("s", &[attr("xs.n", "name")], &format!("Texture {index}"));
    xml.empty("s", &[attr("xs.n", "memo")]);
    xml.text("b", &[attr("xs.n", "isVisible")], "true");
    xml.text("b", &[attr("xs.n", "isClipping")], "false");
    reference(xml, "CBlend_Normal", "blend", project.blend_normal);
    xml.empty(
        "CLayerGuid",
        &[
            attr("xs.n", "guid"),
            attr("uuid", Uuid::new_v4()),
            attr("note", format!("Texture {index}")),
        ],
    );
    reference(xml, "CLayerGroup", "group", project.layer_group);
    xml.text("i", &[attr("xs.n", "opacity255")], "255");
    xml.empty(
        "hash_map",
        &[
            attr("xs.n", "_optionOfIOption"),
            attr("count", 0),
            attr("keyType", "string"),
        ],
    );
    reference(xml, "CLayeredImage", "_layeredImage", project.layered_image);
    xml.end("ACLayerEntry");
    xml.end("ACImageLayer");
    reference(xml, "CImageResource", "imageResource", page.image_resource);
    xml.start("CRect", &[attr("xs.n", "boundsOnImageDoc")]);
    xml.text("i", &[attr("xs.n", "x")], "0");
    xml.text("i", &[attr("xs.n", "y")], "0");
    xml.text("i", &[attr("xs.n", "width")], &texture.width().to_string());
    xml.text(
        "i",
        &[attr("xs.n", "height")],
        &texture.height().to_string(),
    );
    xml.end("CRect");
    xml.start("CLayerIdentifier", &[attr("xs.n", "layerIdentifier")]);
    xml.text(
        "s",
        &[attr("xs.n", "layerName")],
        &format!("Texture {index}"),
    );
    xml.text(
        "s",
        &[attr("xs.n", "layerId")],
        &format!("00-00-{index:02}-01"),
    );
    xml.text(
        "i",
        &[attr("xs.n", "layerIdValue_testImpl")],
        &(index + 1).to_string(),
    );
    xml.end("CLayerIdentifier");
    xml.empty("null", &[attr("xs.n", "icon16")]);
    xml.empty("null", &[attr("xs.n", "icon64")]);
    xml.empty(
        "linked_map",
        &[
            attr("xs.n", "layerInfo"),
            attr("count", 0),
            attr("keyType", "string"),
        ],
    );
    xml.empty(
        "hash_map",
        &[
            attr("xs.n", "_optionOfIOption"),
            attr("count", 0),
            attr("keyType", "string"),
        ],
    );
    xml.end("CLayer");
}

pub(super) fn write_group(xml: &mut XmlWriter, plan: &ProjectPlan) {
    start_shared(xml, "CLayerGroup", plan.layer_group);
    xml.start("ACLayerGroup", &[attr("xs.n", "super")]);
    xml.start("ACLayerEntry", &[attr("xs.n", "super")]);
    xml.text("s", &[attr("xs.n", "name")], "root");
    xml.empty("s", &[attr("xs.n", "memo")]);
    xml.text("b", &[attr("xs.n", "isVisible")], "true");
    xml.text("b", &[attr("xs.n", "isClipping")], "false");
    reference(xml, "CBlend_Normal", "blend", plan.blend_normal);
    xml.empty(
        "CLayerGuid",
        &[
            attr("xs.n", "guid"),
            attr("uuid", Uuid::new_v4()),
            attr("note", "root"),
        ],
    );
    xml.empty("null", &[attr("xs.n", "group")]);
    xml.text("i", &[attr("xs.n", "opacity255")], "255");
    xml.empty(
        "hash_map",
        &[
            attr("xs.n", "_optionOfIOption"),
            attr("count", 0),
            attr("keyType", "string"),
        ],
    );
    reference(xml, "CLayeredImage", "_layeredImage", plan.layered_image);
    xml.end("ACLayerEntry");
    xml.start(
        "carray_list",
        &[attr("xs.n", "_children"), attr("count", plan.pages.len())],
    );
    for page in &plan.pages {
        reference_without_name(xml, "CLayer", page.layer);
    }
    xml.end("carray_list");
    xml.end("ACLayerGroup");
    xml.empty("null", &[attr("xs.n", "layerIdentifier")]);
    xml.end("CLayerGroup");
}

pub(super) fn write_layered_image(
    xml: &mut XmlWriter,
    plan: &ProjectPlan,
    model: &Moc3Model,
    texture_count: usize,
) {
    let width = pixel_dimension(model.canvas().size()[0]);
    let height = pixel_dimension(model.canvas().size()[1]);
    start_shared(xml, "CLayeredImage", plan.layered_image);
    xml.text("s", &[attr("xs.n", "name")], "moc2cmo_atlas.psd");
    xml.empty("s", &[attr("xs.n", "memo")]);
    xml.text("i", &[attr("xs.n", "width")], &width);
    xml.text("i", &[attr("xs.n", "height")], &height);
    xml.text("file", &[attr("xs.n", "psdFile")], "moc2cmo_atlas.psd");
    xml.empty("s", &[attr("xs.n", "description")]);
    reference(xml, "CLayeredImageGuid", "guid", plan.layered_image_guid);
    xml.empty("null", &[attr("xs.n", "psdBytes")]);
    xml.text("l", &[attr("xs.n", "psdFileLastModified")], "0");
    reference(xml, "CLayerGroup", "_rootLayer", plan.layer_group);
    xml.start("LayerSet", &[attr("xs.n", "layerSet")]);
    reference(xml, "CLayeredImage", "_layeredImage", plan.layered_image);
    xml.start(
        "carray_list",
        &[
            attr("xs.n", "_layerEntryList"),
            attr("count", texture_count + 1),
        ],
    );
    reference_without_name(xml, "CLayerGroup", plan.layer_group);
    for page in &plan.pages {
        reference_without_name(xml, "CLayer", page.layer);
    }
    xml.end("carray_list");
    xml.end("LayerSet");
    xml.empty("null", &[attr("xs.n", "icon16")]);
    xml.empty("null", &[attr("xs.n", "icon64")]);
    xml.end("CLayeredImage");
}

fn pixel_dimension(value: f32) -> String {
    if value.is_finite() {
        value.abs().round().max(1.0).to_string()
    } else {
        "1".into()
    }
}
