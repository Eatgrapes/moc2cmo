use crate::Result;

use super::{
    ArtMesh, ArtMeshKeyform,
    layout::{
        Counts, OFFSET_COUNT, checked_range, checked_scaled_range, invalid, nonnegative,
        optional_unbounded_index, read_colors, read_f32, read_i16, read_i32, read_i32_or, read_u8,
    },
};
use crate::moc3::reader::Reader;

pub(super) fn parse_art_meshes(
    reader: &Reader<'_>,
    offsets: &[usize; OFFSET_COUNT],
    ids: Vec<String>,
    counts: &Counts,
) -> Result<Vec<ArtMesh>> {
    let count = ids.len();
    let band_indices = read_i32_or(reader, offsets, 34, count, -1)?;
    let keyform_begins = read_i32(reader, offsets, 35, count)?;
    let keyform_counts = read_i32(reader, offsets, 36, count)?;
    let parent_parts = read_i32(reader, offsets, 39, count)?;
    let parent_deformers = read_i32_or(reader, offsets, 40, count, -1)?;
    let texture_indices = read_i32(reader, offsets, 41, count)?;
    let flags = read_u8(reader, offsets, 42, count)?;
    let vertex_counts = read_i32(reader, offsets, 43, count)?;
    let uv_begins = read_i32(reader, offsets, 44, count)?;
    let index_begins = read_i32(reader, offsets, 45, count)?;
    let index_counts = read_i32(reader, offsets, 46, count)?;
    let mask_begins = read_i32(reader, offsets, 47, count)?;
    let mask_counts = read_i32(reader, offsets, 48, count)?;
    let keyform_opacities = read_f32(reader, offsets, 68, counts.art_mesh_keyforms)?;
    let keyform_draw_orders = read_f32(reader, offsets, 69, counts.art_mesh_keyforms)?;
    let position_begins = read_i32(reader, offsets, 70, counts.art_mesh_keyforms)?;
    let positions = read_f32(reader, offsets, 71, counts.keyform_positions)?;
    let uv_values = read_f32(reader, offsets, 78, counts.uvs)?;
    let position_indices = read_i16(reader, offsets, 79, counts.position_indices)?;
    let masks = read_i32(reader, offsets, 80, counts.drawable_masks)?;
    let color_begins = read_i32_or(reader, offsets, 107, count, -1)?;
    let multiply = read_colors(
        reader,
        offsets,
        [108, 109, 110],
        counts.keyform_multiply_colors,
        [1.0; 3],
    )?;
    let screen = read_colors(
        reader,
        offsets,
        [111, 112, 113],
        counts.keyform_screen_colors,
        [0.0; 3],
    )?;

    ids.into_iter()
        .enumerate()
        .map(|(index, id)| {
            let vertex_count = nonnegative(vertex_counts[index], "vertex count")?;
            let uv_range =
                checked_scaled_range(uv_begins[index], vertex_count, 2, uv_values.len(), "uv")?;
            let uvs = uv_values[uv_range]
                .chunks_exact(2)
                .map(|xy| [xy[0], xy[1]])
                .collect();
            let index_range = checked_range(
                index_begins[index],
                index_counts[index],
                position_indices.len(),
                "triangle indices",
            )?;
            let triangle_indices = position_indices[index_range]
                .iter()
                .map(|value| u16::try_from(*value).map_err(|_| invalid("negative triangle index")))
                .collect::<Result<Vec<_>>>()?;
            let mask_range = checked_range(
                mask_begins[index],
                mask_counts[index],
                masks.len(),
                "drawable masks",
            )?;
            let mask_drawable_indices = masks[mask_range]
                .iter()
                .map(|value| {
                    let value =
                        usize::try_from(*value).map_err(|_| invalid("negative mask index"))?;
                    (value < count)
                        .then_some(value)
                        .ok_or_else(|| invalid("mask index is outside the drawable table"))
                })
                .collect::<Result<Vec<_>>>()?;
            let keyform_range = checked_range(
                keyform_begins[index],
                keyform_counts[index],
                counts.art_mesh_keyforms,
                "art mesh keyforms",
            )?;
            let keyforms = parse_keyforms(
                index,
                keyform_range,
                vertex_count,
                color_begins[index],
                &position_begins,
                &positions,
                &keyform_opacities,
                &keyform_draw_orders,
                &multiply,
                &screen,
            )?;

            Ok(ArtMesh {
                id,
                texture_index: usize::try_from(texture_indices[index])
                    .map_err(|_| invalid("texture index is negative"))?,
                drawable_flags: flags[index],
                parent_part_index: optional_unbounded_index(parent_parts[index])?,
                parent_deformer_index: optional_unbounded_index(parent_deformers[index])?,
                binding_band_index: optional_unbounded_index(band_indices[index])?,
                uvs,
                triangle_indices,
                mask_drawable_indices,
                keyforms,
            })
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn parse_keyforms(
    _mesh_index: usize,
    keyform_range: std::ops::Range<usize>,
    vertex_count: usize,
    color_begin: i32,
    position_begins: &[i32],
    positions: &[f32],
    opacities: &[f32],
    draw_orders: &[f32],
    multiply: &[[f32; 3]],
    screen: &[[f32; 3]],
) -> Result<Vec<ArtMeshKeyform>> {
    let mut keyforms = Vec::with_capacity(keyform_range.len());
    for (local_index, keyform_index) in keyform_range.enumerate() {
        let position_range = checked_scaled_range(
            position_begins[keyform_index],
            vertex_count,
            2,
            positions.len(),
            "keyform positions",
        )?;
        let keyform_positions = positions[position_range]
            .chunks_exact(2)
            .map(|xy| [xy[0], xy[1]])
            .collect();
        let color_index = if color_begin < 0 {
            None
        } else {
            Some(
                usize::try_from(color_begin)
                    .map_err(|_| invalid("color index is too large"))?
                    .checked_add(local_index)
                    .ok_or_else(|| invalid("color index overflows"))?,
            )
        };
        keyforms.push(ArtMeshKeyform {
            positions: keyform_positions,
            opacity: opacities[keyform_index],
            draw_order: draw_orders[keyform_index],
            multiply_color: color_index
                .and_then(|index| multiply.get(index).copied())
                .unwrap_or([1.0; 3]),
            screen_color: color_index
                .and_then(|index| screen.get(index).copied())
                .unwrap_or([0.0; 3]),
        });
    }
    Ok(keyforms)
}
