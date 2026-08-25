use crate::{Error, Result};

use super::{Canvas, Endianness, Moc3Version, Parameter};
use crate::moc3::reader::Reader;

const HEADER_SIZE: usize = 64;
pub(super) const OFFSET_COUNT: usize = 160;
const OFFSET_TABLE_START: usize = 0x40;
const OFFSET_TABLE_END: usize = OFFSET_TABLE_START + OFFSET_COUNT * 4;

#[derive(Default)]
pub(super) struct Counts {
    pub(super) parts: usize,
    pub(super) deformers: usize,
    pub(super) warp_deformers: usize,
    pub(super) rotation_deformers: usize,
    pub(super) art_meshes: usize,
    pub(super) parameters: usize,
    pub(super) part_keyforms: usize,
    pub(super) warp_deformer_keyforms: usize,
    pub(super) rotation_deformer_keyforms: usize,
    pub(super) art_mesh_keyforms: usize,
    pub(super) keyform_positions: usize,
    pub(super) parameter_binding_indices: usize,
    pub(super) keyform_bindings: usize,
    pub(super) parameter_bindings: usize,
    pub(super) keys: usize,
    pub(super) uvs: usize,
    pub(super) position_indices: usize,
    pub(super) drawable_masks: usize,
    pub(super) glues: usize,
    pub(super) glue_vertices: usize,
    pub(super) glue_keyforms: usize,
    pub(super) keyform_multiply_colors: usize,
    pub(super) keyform_screen_colors: usize,
}

pub(super) fn parse_header(bytes: &[u8]) -> Result<(Moc3Version, Endianness)> {
    if bytes.len() < HEADER_SIZE {
        return Err(invalid("header is shorter than 64 bytes"));
    }
    if bytes.get(..4) != Some(b"MOC3") {
        return Err(invalid("magic must be MOC3"));
    }
    let version = Moc3Version::from_raw(bytes[4])?;
    let endianness = match bytes[5] {
        0 => Endianness::Little,
        1 => Endianness::Big,
        value => return Err(invalid(format!("invalid endianness flag {value}"))),
    };
    Ok((version, endianness))
}

pub(super) fn parse_offsets(reader: &Reader<'_>) -> Result<[usize; OFFSET_COUNT]> {
    if reader.len() < OFFSET_TABLE_END {
        return Err(invalid("section offset table is incomplete"));
    }
    let mut offsets = [0; OFFSET_COUNT];
    for (index, target) in offsets.iter_mut().enumerate() {
        let value = usize::try_from(reader.u32(OFFSET_TABLE_START + index * 4)?)
            .map_err(|_| invalid("section offset is too large"))?;
        if value != 0
            && value != reader.len()
            && (value < OFFSET_TABLE_END || value >= reader.len() || value % 4 != 0)
        {
            return Err(invalid(format!(
                "invalid section {index} offset 0x{value:x}"
            )));
        }
        *target = value;
    }
    if offsets[0] == 0 || offsets[1] == 0 {
        return Err(invalid("count or canvas section is missing"));
    }
    Ok(offsets)
}

pub(super) fn parse_counts(
    reader: &Reader<'_>,
    offsets: &[usize; OFFSET_COUNT],
    version: Moc3Version,
) -> Result<Counts> {
    let values = reader.section_u32(offsets[0], version.count_words())?;
    let count = |index: usize, name: &str| -> Result<usize> {
        let value = values.get(index).copied().unwrap_or(0);
        usize::try_from(value).map_err(|_| invalid(format!("{name} count is too large")))
    };
    Ok(Counts {
        parts: count(0, "part")?,
        deformers: count(1, "deformer")?,
        warp_deformers: count(2, "warp deformer")?,
        rotation_deformers: count(3, "rotation deformer")?,
        art_meshes: count(4, "art mesh")?,
        parameters: count(5, "parameter")?,
        part_keyforms: count(6, "part keyform")?,
        warp_deformer_keyforms: count(7, "warp deformer keyform")?,
        rotation_deformer_keyforms: count(8, "rotation deformer keyform")?,
        art_mesh_keyforms: count(9, "art mesh keyform")?,
        keyform_positions: count(10, "keyform position")?,
        parameter_binding_indices: count(11, "parameter binding index")?,
        keyform_bindings: count(12, "keyform binding")?,
        parameter_bindings: count(13, "parameter binding")?,
        keys: count(14, "key")?,
        uvs: count(15, "uv")?,
        position_indices: count(16, "position index")?,
        drawable_masks: count(17, "drawable mask")?,
        glues: count(20, "glue")?,
        glue_vertices: count(21, "glue vertex")?,
        glue_keyforms: count(22, "glue keyform")?,
        keyform_multiply_colors: count(23, "multiply color")?,
        keyform_screen_colors: count(24, "screen color")?,
    })
}

pub(super) fn parse_canvas(reader: &Reader<'_>, offsets: &[usize; OFFSET_COUNT]) -> Result<Canvas> {
    let offset = offsets[1];
    Ok(Canvas {
        pixels_per_unit: reader.f32(offset)?,
        origin: [reader.f32(offset + 4)?, reader.f32(offset + 8)?],
        size: [reader.f32(offset + 12)?, reader.f32(offset + 16)?],
        reverse_y: reader.u8(offset + 20)? & 1 != 0,
    })
}

pub(super) fn parse_ids(
    reader: &Reader<'_>,
    offsets: &[usize; OFFSET_COUNT],
    slot: usize,
    count: usize,
) -> Result<Vec<String>> {
    let offset = required_offset(offsets, slot, count)?;
    (0..count)
        .map(|index| reader.str64(offset + index * 64))
        .collect()
}

pub(super) fn parse_parameters(
    reader: &Reader<'_>,
    offsets: &[usize; OFFSET_COUNT],
    ids: Vec<String>,
) -> Result<Vec<Parameter>> {
    let count = ids.len();
    let maximums = read_f32(reader, offsets, 51, count)?;
    let minimums = read_f32(reader, offsets, 52, count)?;
    let defaults = read_f32(reader, offsets, 53, count)?;
    Ok(ids
        .into_iter()
        .enumerate()
        .map(|(index, id)| Parameter {
            id,
            minimum: minimums[index],
            maximum: maximums[index],
            default: defaults[index],
        })
        .collect())
}

pub(super) fn read_i16(
    reader: &Reader<'_>,
    offsets: &[usize; OFFSET_COUNT],
    slot: usize,
    count: usize,
) -> Result<Vec<i16>> {
    reader.section_i16(required_offset(offsets, slot, count)?, count)
}

pub(super) fn read_i32(
    reader: &Reader<'_>,
    offsets: &[usize; OFFSET_COUNT],
    slot: usize,
    count: usize,
) -> Result<Vec<i32>> {
    reader.section_i32(required_offset(offsets, slot, count)?, count)
}

pub(super) fn read_i32_or(
    reader: &Reader<'_>,
    offsets: &[usize; OFFSET_COUNT],
    slot: usize,
    count: usize,
    default: i32,
) -> Result<Vec<i32>> {
    if count == 0 || offsets[slot] == 0 {
        Ok(vec![default; count])
    } else {
        reader.section_i32(offsets[slot], count)
    }
}

pub(super) fn read_f32(
    reader: &Reader<'_>,
    offsets: &[usize; OFFSET_COUNT],
    slot: usize,
    count: usize,
) -> Result<Vec<f32>> {
    reader.section_f32(required_offset(offsets, slot, count)?, count)
}

pub(super) fn read_f32_or(
    reader: &Reader<'_>,
    offsets: &[usize; OFFSET_COUNT],
    slot: usize,
    count: usize,
    default: f32,
) -> Result<Vec<f32>> {
    if count == 0 || offsets[slot] == 0 {
        Ok(vec![default; count])
    } else {
        reader.section_f32(offsets[slot], count)
    }
}

pub(super) fn read_u16(
    reader: &Reader<'_>,
    offsets: &[usize; OFFSET_COUNT],
    slot: usize,
    count: usize,
) -> Result<Vec<u16>> {
    reader.section_u16(required_offset(offsets, slot, count)?, count)
}

pub(super) fn read_u8(
    reader: &Reader<'_>,
    offsets: &[usize; OFFSET_COUNT],
    slot: usize,
    count: usize,
) -> Result<Vec<u8>> {
    reader.section_u8(required_offset(offsets, slot, count)?, count)
}

pub(super) fn read_colors(
    reader: &Reader<'_>,
    offsets: &[usize; OFFSET_COUNT],
    slots: [usize; 3],
    count: usize,
    default: [f32; 3],
) -> Result<Vec<[f32; 3]>> {
    if count == 0 || slots.iter().any(|slot| offsets[*slot] == 0) {
        return Ok(vec![default; count]);
    }
    let channels = [
        reader.section_f32(offsets[slots[0]], count)?,
        reader.section_f32(offsets[slots[1]], count)?,
        reader.section_f32(offsets[slots[2]], count)?,
    ];
    Ok((0..count)
        .map(|index| [channels[0][index], channels[1][index], channels[2][index]])
        .collect())
}

pub(super) fn checked_range(
    begin: i32,
    count: i32,
    source_len: usize,
    name: &str,
) -> Result<std::ops::Range<usize>> {
    let begin = nonnegative(begin, name)?;
    let count = nonnegative(count, name)?;
    let end = begin
        .checked_add(count)
        .ok_or_else(|| invalid(format!("{name} range overflows")))?;
    if end > source_len {
        return Err(invalid(format!("{name} range is outside its section")));
    }
    Ok(begin..end)
}

pub(super) fn checked_scaled_range(
    begin: i32,
    count: usize,
    scale: usize,
    source_len: usize,
    name: &str,
) -> Result<std::ops::Range<usize>> {
    let begin = nonnegative(begin, name)?;
    let len = count
        .checked_mul(scale)
        .ok_or_else(|| invalid(format!("{name} length overflows")))?;
    let end = begin
        .checked_add(len)
        .ok_or_else(|| invalid(format!("{name} range overflows")))?;
    if end > source_len {
        return Err(invalid(format!("{name} range is outside its section")));
    }
    Ok(begin..end)
}

pub(super) fn nonnegative(value: i32, name: &str) -> Result<usize> {
    usize::try_from(value).map_err(|_| invalid(format!("{name} is negative")))
}

pub(super) fn optional_index(value: i32, len: usize, name: &str) -> Result<Option<usize>> {
    let value = optional_unbounded_index(value)?;
    if value.is_some_and(|value| value >= len) {
        return Err(invalid(format!("{name} is outside its table")));
    }
    Ok(value)
}

pub(super) fn optional_unbounded_index(value: i32) -> Result<Option<usize>> {
    if value < 0 {
        Ok(None)
    } else {
        usize::try_from(value)
            .map(Some)
            .map_err(|_| invalid("index is too large"))
    }
}

fn required_offset(offsets: &[usize; OFFSET_COUNT], slot: usize, count: usize) -> Result<usize> {
    if count == 0 {
        return Ok(0);
    }
    let offset = offsets[slot];
    (offset != 0)
        .then_some(offset)
        .ok_or_else(|| invalid(format!("section {slot} is missing")))
}

pub(super) fn invalid(message: impl Into<String>) -> Error {
    Error::InvalidMoc3(message.into())
}
