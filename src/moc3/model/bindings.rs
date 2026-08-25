use crate::Result;

use super::{
    BindingBand, ParameterBinding,
    layout::{Counts, OFFSET_COUNT, checked_range, invalid, read_f32, read_i32},
};
use crate::moc3::reader::Reader;

pub(super) fn parse_bindings(
    reader: &Reader<'_>,
    offsets: &[usize; OFFSET_COUNT],
    counts: &Counts,
) -> Result<Vec<BindingBand>> {
    let parameter_begins = read_i32(reader, offsets, 56, counts.parameters)?;
    let binding_parameters =
        expand_binding_parameters(&parameter_begins, counts.parameter_bindings)?;
    let band_binding_indices = read_i32(reader, offsets, 72, counts.parameter_binding_indices)?;
    let band_begins = read_i32(reader, offsets, 73, counts.keyform_bindings)?;
    let band_counts = read_i32(reader, offsets, 74, counts.keyform_bindings)?;
    let key_begins = read_i32(reader, offsets, 75, counts.parameter_bindings)?;
    let key_counts = read_i32(reader, offsets, 76, counts.parameter_bindings)?;
    let key_values = read_f32(reader, offsets, 77, counts.keys)?;

    let mut parameter_bindings = Vec::with_capacity(counts.parameter_bindings);
    for index in 0..counts.parameter_bindings {
        let range = checked_range(
            key_begins[index],
            key_counts[index],
            key_values.len(),
            "binding keys",
        )?;
        parameter_bindings.push(ParameterBinding {
            parameter_index: binding_parameters[index],
            keys: key_values[range].to_vec(),
        });
    }

    let mut bands = Vec::with_capacity(counts.keyform_bindings);
    for index in 0..counts.keyform_bindings {
        let range = checked_range(
            band_begins[index],
            band_counts[index],
            band_binding_indices.len(),
            "binding band",
        )?;
        let bindings = band_binding_indices[range]
            .iter()
            .map(|value| {
                let binding_index =
                    usize::try_from(*value).map_err(|_| invalid("binding index is negative"))?;
                parameter_bindings
                    .get(binding_index)
                    .cloned()
                    .ok_or_else(|| invalid("binding index is outside the binding table"))
            })
            .collect::<Result<Vec<_>>>()?;
        bands.push(BindingBand { bindings });
    }
    Ok(bands)
}

fn expand_binding_parameters(begins: &[i32], binding_count: usize) -> Result<Vec<usize>> {
    let mut result = vec![None; binding_count];
    for (parameter_index, begin) in begins.iter().copied().enumerate() {
        let Ok(begin) = usize::try_from(begin) else {
            continue;
        };
        let end = begins[parameter_index + 1..]
            .iter()
            .filter_map(|value| usize::try_from(*value).ok())
            .find(|next| *next > begin)
            .unwrap_or(binding_count);
        let slots = result
            .get_mut(begin..end)
            .ok_or_else(|| invalid("parameter binding range is invalid"))?;
        for slot in slots {
            slot.get_or_insert(parameter_index);
        }
    }
    result
        .into_iter()
        .map(|value| value.ok_or_else(|| invalid("parameter binding has no owner")))
        .collect()
}
