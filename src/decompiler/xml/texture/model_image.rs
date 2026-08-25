use crate::decompiler::Texture;

use super::{
    super::{
        grid::{reference, reference_without_name, start_shared},
        plan::{PagePlan, ProjectPlan},
        writer::{XmlWriter, attr},
    },
    affine, cache,
};

pub(super) fn write_group(xml: &mut XmlWriter, plan: &ProjectPlan, textures: &[Texture]) {
    start_shared(xml, "CModelImageGroup", plan.image_group);
    xml.empty("s", &[attr("xs.n", "memo")]);
    xml.text("s", &[attr("xs.n", "groupName")], "moc2cmo_atlas");
    xml.start(
        "carray_list",
        &[attr("xs.n", "_linkedRawImageGuids"), attr("count", 1)],
    );
    reference_without_name(xml, "CLayeredImageGuid", plan.layered_image_guid);
    xml.end("carray_list");
    xml.start(
        "carray_list",
        &[
            attr("xs.n", "_modelImages"),
            attr("count", plan.pages.len()),
        ],
    );
    for (index, (page, texture)) in plan.pages.iter().zip(textures).enumerate() {
        write_model_image(xml, plan, page, texture, index);
    }
    xml.end("carray_list");
    xml.end("CModelImageGroup");
}

fn write_model_image(
    xml: &mut XmlWriter,
    plan: &ProjectPlan,
    page: &PagePlan,
    texture: &Texture,
    index: usize,
) {
    let ids = &plan.filters.value_ids;
    xml.start("CModelImage", &[attr("modelImageVersion", 0)]);
    reference(xml, "CModelImageGuid", "guid", page.model_image_guid);
    xml.text("s", &[attr("xs.n", "name")], &format!("Texture {index}"));
    reference(xml, "ModelImageFilterSet", "inputFilter", page.filter_set);
    xml.start("ModelImageFilterEnv", &[attr("xs.n", "inputFilterEnv")]);
    xml.start("FilterEnv", &[attr("xs.n", "super")]);
    xml.empty("null", &[attr("xs.n", "parentEnv")]);
    xml.start("hash_map", &[attr("xs.n", "envValues"), attr("count", 2)]);
    xml.start("entry", &[]);
    reference(xml, "FilterValueId", "key", ids[3]);
    xml.start("EnvValueSet", &[attr("xs.n", "value")]);
    reference(xml, "FilterValueId", "id", ids[3]);
    reference(xml, "CLayeredImageGuid", "value", plan.layered_image_guid);
    xml.text("l", &[attr("xs.n", "updateTimeMs")], "0");
    xml.end("EnvValueSet");
    xml.end("entry");
    xml.start("entry", &[]);
    reference(xml, "FilterValueId", "key", ids[1]);
    xml.start("EnvValueSet", &[attr("xs.n", "value")]);
    reference(xml, "FilterValueId", "id", ids[1]);
    xml.start("CLayerSelectorMap", &[attr("xs.n", "value")]);
    xml.start(
        "linked_map",
        &[attr("xs.n", "_imageToLayerInput"), attr("count", 1)],
    );
    xml.start("entry", &[]);
    reference(xml, "CLayeredImageGuid", "key", plan.layered_image_guid);
    xml.start("array_list", &[attr("xs.n", "value"), attr("count", 1)]);
    xml.start("CLayerInputData", &[]);
    reference(xml, "CLayer", "layer", page.layer);
    affine(xml, "affine");
    xml.empty("null", &[attr("xs.n", "clippingOnTexturePx")]);
    xml.end("CLayerInputData");
    xml.end("array_list");
    xml.end("entry");
    xml.end("linked_map");
    xml.end("CLayerSelectorMap");
    xml.text("l", &[attr("xs.n", "updateTimeMs")], "0");
    xml.end("EnvValueSet");
    xml.end("entry");
    xml.end("hash_map");
    xml.end("FilterEnv");
    xml.end("ModelImageFilterEnv");
    reference(xml, "CImageResource", "_filteredImage", page.image_resource);
    xml.empty("null", &[attr("xs.n", "icon16")]);
    affine(xml, "_materialLocalToCanvasTransform");
    reference(xml, "CModelImageGroup", "_group", plan.image_group);
    xml.start(
        "carray_list",
        &[attr("xs.n", "linkedRawImageGuids"), attr("count", 1)],
    );
    reference_without_name(xml, "CLayeredImageGuid", plan.layered_image_guid);
    xml.end("carray_list");
    cache::write_manager(xml, "cachedImageManager", page.image_resource, texture);
    xml.empty("s", &[attr("xs.n", "memo")]);
    xml.end("CModelImage");
}
