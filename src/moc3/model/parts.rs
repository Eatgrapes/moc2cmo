use crate::Result;

use super::{
    Part, PartKeyform,
    layout::{
        Counts, OFFSET_COUNT, checked_range, optional_index, optional_unbounded_index, read_f32,
        read_i32,
    },
};
use crate::moc3::reader::Reader;

pub(super) fn parse_parts(
    reader: &Reader<'_>,
    offsets: &[usize; OFFSET_COUNT],
    ids: Vec<String>,
    counts: &Counts,
) -> Result<Vec<Part>> {
    let count = ids.len();
    let band_indices = read_i32(reader, offsets, 4, count)?;
    let keyform_begins = read_i32(reader, offsets, 5, count)?;
    let keyform_counts = read_i32(reader, offsets, 6, count)?;
    let parents = read_i32(reader, offsets, 9, count)?;
    let draw_orders = read_f32(reader, offsets, 58, counts.part_keyforms)?;
    let opacities = read_f32(reader, offsets, 59, counts.part_keyforms)?;

    ids.into_iter()
        .enumerate()
        .map(|(index, id)| {
            let range = checked_range(
                keyform_begins[index],
                keyform_counts[index],
                counts.part_keyforms,
                "part keyforms",
            )?;
            let keyforms = range
                .map(|keyform_index| PartKeyform {
                    opacity: opacities[keyform_index],
                    draw_order: draw_orders[keyform_index],
                })
                .collect();
            Ok(Part {
                id,
                parent_part_index: optional_index(parents[index], count, "part parent")?,
                binding_band_index: optional_unbounded_index(band_indices[index])?,
                keyforms,
            })
        })
        .collect()
}
