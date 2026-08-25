use super::super::{
    constants::{FILTER_LAYER_UUID, FILTER_SELECTOR_UUID},
    grid::{empty_shared, reference, start_shared, start_shared_with},
    plan::{FilterGlobals, PagePlan, ProjectPlan, RefId},
    writer::{XmlWriter, attr},
};

pub(super) fn write_globals(xml: &mut XmlWriter, filters: &FilterGlobals) {
    empty_shared(
        xml,
        "StaticFilterDefGuid",
        filters.selector_definition,
        vec![
            attr("uuid", FILTER_SELECTOR_UUID),
            attr("note", "CLayerSelector"),
        ],
    );
    empty_shared(
        xml,
        "StaticFilterDefGuid",
        filters.filter_definition,
        vec![
            attr("uuid", FILTER_LAYER_UUID),
            attr("note", "CLayerFilter"),
        ],
    );

    let value_id_names = [
        "ilf_outputLayerData",
        "mi_input_layerInputData",
        "ilf_inputLayerData",
        "mi_currentImageGuid",
        "ilf_currentImageGuid",
        "mi_output_image",
        "mi_output_transform",
        "ilf_inputLayer",
    ];
    for (id, name) in filters.value_ids.iter().zip(value_id_names) {
        empty_shared(xml, "FilterValueId", *id, vec![attr("idstr", name)]);
    }

    let values = [
        ("Select Layer", Some(filters.value_ids[0]), None),
        ("Import Layer", Some(filters.value_ids[1]), None),
        ("Import Layer selection", Some(filters.value_ids[2]), None),
        ("Current GUID", Some(filters.value_ids[3]), None),
        (
            "GUID of Selected Source Image",
            Some(filters.value_ids[4]),
            None,
        ),
        ("Output image", Some(filters.value_ids[5]), None),
        (
            "Output Image (Resource Format)",
            None,
            Some("ilf_outputImageRes"),
        ),
        ("LayerToCanvas transform", Some(filters.value_ids[6]), None),
        ("LayerToCanvas transform", None, Some("ilf_outputTransform")),
    ];
    for (plan, (name, reference_id, inline_id)) in filters.values.iter().zip(values) {
        start_shared(xml, "FilterValue", *plan);
        xml.text("s", &[attr("xs.n", "name")], name);
        if let Some(id) = reference_id {
            reference(xml, "FilterValueId", "id", id);
        } else {
            xml.empty(
                "FilterValueId",
                &[
                    attr("xs.n", "id"),
                    attr("idstr", inline_id.unwrap_or_default()),
                ],
            );
        }
        xml.empty("null", &[attr("xs.n", "defaultValueInitializer")]);
        xml.end("FilterValue");
    }
}

pub(super) fn write_page(
    xml: &mut XmlWriter,
    project: &ProjectPlan,
    page: &PagePlan,
    index: usize,
) {
    let ids = &project.filters.value_ids;
    let values = &project.filters.values;
    empty_shared(
        xml,
        "FilterInstanceId",
        page.filter_instance_ids[0],
        vec![attr("idstr", format!("filter0_{index}"))],
    );
    empty_shared(
        xml,
        "FilterInstanceId",
        page.filter_instance_ids[1],
        vec![attr("idstr", format!("filter1_{index}"))],
    );

    start_shared(xml, "FilterOutputValueConnector", page.filter_output);
    xml.empty("AValueConnector", &[attr("xs.n", "super")]);
    reference(xml, "FilterInstance", "instance", page.filter_instances[0]);
    reference(xml, "FilterValueId", "id", ids[0]);
    reference(xml, "FilterValue", "valueDef", values[0]);
    xml.end("FilterOutputValueConnector");

    write_selector(xml, project, page);
    write_layer_filter(xml, project, page);
    write_filter_set(xml, project, page);
}

fn write_selector(xml: &mut XmlWriter, project: &ProjectPlan, page: &PagePlan) {
    let ids = &project.filters.value_ids;
    start_shared_with(
        xml,
        "FilterInstance",
        page.filter_instances[0],
        vec![attr("filterName", "CLayerSelector")],
    );
    reference(
        xml,
        "StaticFilterDefGuid",
        "filterDefGuid",
        project.filters.selector_definition,
    );
    xml.empty("null", &[attr("xs.n", "filterDef")]);
    reference(
        xml,
        "FilterInstanceId",
        "filterId",
        page.filter_instance_ids[0],
    );
    xml.start(
        "hash_map",
        &[attr("xs.n", "inputConnectors"), attr("count", 2)],
    );
    env_connector(xml, ids[2], ids[1]);
    env_connector(xml, ids[4], ids[3]);
    xml.end("hash_map");
    xml.start(
        "hash_map",
        &[attr("xs.n", "outputConnectors"), attr("count", 1)],
    );
    xml.start("entry", &[]);
    reference(xml, "FilterValueId", "key", ids[0]);
    reference(
        xml,
        "FilterOutputValueConnector",
        "value",
        page.filter_output,
    );
    xml.end("entry");
    xml.end("hash_map");
    reference(
        xml,
        "ModelImageFilterSet",
        "ownerFilterSet",
        page.filter_set,
    );
    xml.end("FilterInstance");
}

fn env_connector(xml: &mut XmlWriter, key: RefId, env: RefId) {
    xml.start("entry", &[]);
    reference(xml, "FilterValueId", "key", key);
    xml.start("EnvValueConnector", &[attr("xs.n", "value")]);
    xml.empty("AValueConnector", &[attr("xs.n", "super")]);
    reference(xml, "FilterValueId", "envValueId", env);
    xml.end("EnvValueConnector");
    xml.end("entry");
}

fn write_layer_filter(xml: &mut XmlWriter, project: &ProjectPlan, page: &PagePlan) {
    let ids = &project.filters.value_ids;
    start_shared_with(
        xml,
        "FilterInstance",
        page.filter_instances[1],
        vec![attr("filterName", "CLayerFilter")],
    );
    reference(
        xml,
        "StaticFilterDefGuid",
        "filterDefGuid",
        project.filters.filter_definition,
    );
    xml.empty("null", &[attr("xs.n", "filterDef")]);
    reference(
        xml,
        "FilterInstanceId",
        "filterId",
        page.filter_instance_ids[1],
    );
    xml.start(
        "hash_map",
        &[attr("xs.n", "inputConnectors"), attr("count", 1)],
    );
    xml.start("entry", &[]);
    reference(xml, "FilterValueId", "key", ids[7]);
    reference(
        xml,
        "FilterOutputValueConnector",
        "value",
        page.filter_output,
    );
    xml.end("entry");
    xml.end("hash_map");
    xml.empty(
        "hash_map",
        &[
            attr("xs.n", "outputConnectors"),
            attr("count", 0),
            attr("keyType", "string"),
        ],
    );
    reference(
        xml,
        "ModelImageFilterSet",
        "ownerFilterSet",
        page.filter_set,
    );
    xml.end("FilterInstance");
}

fn write_filter_set(xml: &mut XmlWriter, project: &ProjectPlan, page: &PagePlan) {
    let ids = &project.filters.value_ids;
    let values = &project.filters.values;
    start_shared(xml, "ModelImageFilterSet", page.filter_set);
    xml.start("FilterSet", &[attr("xs.n", "super")]);
    xml.start("linked_map", &[attr("xs.n", "filterMap"), attr("count", 2)]);
    filter_map_entry(xml, page.filter_instance_ids[0], page.filter_instances[0]);
    filter_map_entry(xml, page.filter_instance_ids[1], page.filter_instances[1]);
    xml.end("linked_map");
    xml.start(
        "linked_map",
        &[attr("xs.n", "_externalInputs"), attr("count", 2)],
    );
    env_connection(xml, ids[1], values[1], page.filter_instances[0], values[2]);
    env_connection(xml, ids[3], values[3], page.filter_instances[0], values[4]);
    xml.end("linked_map");
    xml.start(
        "linked_map",
        &[attr("xs.n", "_externalOutputs"), attr("count", 2)],
    );
    env_connection(xml, ids[5], values[5], page.filter_instances[1], values[6]);
    env_connection(xml, ids[6], values[7], page.filter_instances[1], values[8]);
    xml.end("linked_map");
    xml.end("FilterSet");
    xml.end("ModelImageFilterSet");
}

fn filter_map_entry(xml: &mut XmlWriter, id: RefId, instance: RefId) {
    xml.start("entry", &[]);
    reference(xml, "FilterInstanceId", "key", id);
    reference(xml, "FilterInstance", "value", instance);
    xml.end("entry");
}

fn env_connection(
    xml: &mut XmlWriter,
    key: RefId,
    env_value: RefId,
    instance: RefId,
    filter_value: RefId,
) {
    xml.start("entry", &[]);
    reference(xml, "FilterValueId", "key", key);
    xml.start("EnvConnection", &[attr("xs.n", "value")]);
    reference(xml, "FilterValue", "_envValueDef", env_value);
    reference(xml, "FilterInstance", "filter", instance);
    reference(xml, "FilterValue", "filterValueDef", filter_value);
    xml.end("EnvConnection");
    xml.end("entry");
}
