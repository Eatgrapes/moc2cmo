use crate::{Error, Result, moc3::Moc3Model};

use super::{
    plan::{GridPlan, ParameterPlan, RefId},
    writer::{XmlWriter, attr},
};

pub(super) fn write_grid(
    xml: &mut XmlWriter,
    plan: &GridPlan,
    model: &Moc3Model,
    band_index: Option<usize>,
    parameters: &[ParameterPlan],
    description: &str,
) -> Result<()> {
    let bindings = match band_index {
        Some(index) => model
            .binding_bands()
            .get(index)
            .ok_or_else(|| Error::InvalidCmo3(format!("binding band {index} is missing")))?
            .bindings(),
        None => &[],
    };
    if bindings.len() != plan.bindings.len() {
        return Err(Error::InvalidCmo3(
            "binding plan does not match model".into(),
        ));
    }

    start_shared(xml, "KeyformGridSource", plan.grid);
    xml.start(
        "array_list",
        &[
            attr("xs.n", "keyformsOnGrid"),
            attr("count", plan.forms.len()),
        ],
    );
    for (form_index, form) in plan.forms.iter().copied().enumerate() {
        xml.start("KeyformOnGrid", &[]);
        xml.start("KeyformGridAccessKey", &[attr("xs.n", "accessKey")]);
        xml.start(
            "array_list",
            &[
                attr("xs.n", "_keyOnParameterList"),
                attr("count", bindings.len()),
            ],
        );
        let mut stride = 1usize;
        for (binding_index, binding) in bindings.iter().enumerate() {
            let key_index = if binding.keys().is_empty() {
                0
            } else {
                (form_index / stride) % binding.keys().len()
            };
            xml.start("KeyOnParameter", &[]);
            reference(
                xml,
                "KeyformBindingSource",
                "binding",
                plan.bindings[binding_index],
            );
            xml.text("i", &[attr("xs.n", "keyIndex")], &key_index.to_string());
            xml.end("KeyOnParameter");
            stride = stride.saturating_mul(binding.keys().len().max(1));
        }
        xml.end("array_list");
        xml.end("KeyformGridAccessKey");
        reference(xml, "CFormGuid", "keyformGuid", form);
        xml.end("KeyformOnGrid");
    }
    xml.end("array_list");
    xml.start(
        "array_list",
        &[
            attr("xs.n", "keyformBindings"),
            attr("count", plan.bindings.len()),
        ],
    );
    for binding in &plan.bindings {
        reference_without_name(xml, "KeyformBindingSource", *binding);
    }
    xml.end("array_list");
    xml.end("KeyformGridSource");

    for (binding, binding_plan) in bindings.iter().zip(&plan.bindings) {
        let parameter = parameters.get(binding.parameter_index()).ok_or_else(|| {
            Error::InvalidCmo3(format!(
                "parameter binding {} is outside the parameter table",
                binding.parameter_index()
            ))
        })?;
        start_shared(xml, "KeyformBindingSource", *binding_plan);
        reference(xml, "KeyformGridSource", "_gridSource", plan.grid);
        reference(xml, "CParameterGuid", "parameterGuid", parameter.guid);
        xml.start(
            "array_list",
            &[attr("xs.n", "keys"), attr("count", binding.keys().len())],
        );
        for key in binding.keys() {
            xml.text("f", &[], &float(*key));
        }
        xml.end("array_list");
        xml.empty(
            "InterpolationType",
            &[attr("xs.n", "interpolationType"), attr("v", "LINEAR")],
        );
        xml.empty(
            "ExtendedInterpolationType",
            &[
                attr("xs.n", "extendedInterpolationType"),
                attr("v", "LINEAR"),
            ],
        );
        xml.text("i", &[attr("xs.n", "insertPointCount")], "1");
        xml.text("f", &[attr("xs.n", "extendedInterpolationScale")], "1.0");
        xml.text("s", &[attr("xs.n", "description")], description);
        xml.end("KeyformBindingSource");
    }
    Ok(())
}

pub(super) fn start_shared(xml: &mut XmlWriter, tag: &str, id: RefId) {
    start_shared_with(xml, tag, id, Vec::new());
}

pub(super) fn start_shared_with(
    xml: &mut XmlWriter,
    tag: &str,
    id: RefId,
    mut attributes: Vec<(&'static str, String)>,
) {
    attributes.push(attr("xs.id", id.value()));
    attributes.push(attr("xs.idx", id.index()));
    xml.start(tag, &attributes);
}

pub(super) fn empty_shared(
    xml: &mut XmlWriter,
    tag: &str,
    id: RefId,
    mut attributes: Vec<(&'static str, String)>,
) {
    attributes.push(attr("xs.id", id.value()));
    attributes.push(attr("xs.idx", id.index()));
    xml.empty(tag, &attributes);
}

pub(super) fn reference(xml: &mut XmlWriter, tag: &str, name: &str, id: RefId) {
    xml.empty(tag, &[attr("xs.n", name), attr("xs.ref", id.value())]);
}

pub(super) fn reference_without_name(xml: &mut XmlWriter, tag: &str, id: RefId) {
    xml.empty(tag, &[attr("xs.ref", id.value())]);
}

pub(super) fn float(value: f32) -> String {
    if value.is_finite() {
        value.to_string()
    } else {
        "0.0".into()
    }
}

pub(super) fn float_array(values: impl IntoIterator<Item = f32>) -> String {
    values.into_iter().map(float).collect::<Vec<_>>().join(" ")
}
