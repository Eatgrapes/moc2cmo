use std::io::{Cursor, Read};

use flate2::read::DeflateDecoder;

use crate::Result;

use super::{ArchiveEntry, Compression, int64_mask, invalid};

const HEADER_SIZE: usize = 54;

pub(crate) struct DecodedArchive {
    pub(crate) key: i32,
    pub(crate) entries: Vec<ArchiveEntry>,
}

pub(crate) fn decode_archive(bytes: &[u8]) -> Result<DecodedArchive> {
    if bytes.get(..4) != Some(b"CAFF") {
        return Err(invalid("missing CAFF signature"));
    }
    let key = i32::from_be_bytes(read_array(bytes, 14)?);
    let mut reader = CaffReader::new(bytes, HEADER_SIZE, key);
    let entry_count = reader.encoded_i32()?;
    let entry_count = usize::try_from(entry_count).map_err(|_| invalid("negative entry count"))?;
    let maximum_entry_count = bytes.len().saturating_sub(HEADER_SIZE) / 24;
    if entry_count > maximum_entry_count {
        return Err(invalid("entry count exceeds the archive metadata"));
    }
    let mut metadata = Vec::with_capacity(entry_count);
    for _ in 0..entry_count {
        let path = reader.string()?;
        let tag = reader.string()?;
        let position = reader.encoded_i64()?;
        let position = usize::try_from(position).map_err(|_| invalid("negative entry position"))?;
        let size = reader.encoded_i32()?;
        let size = usize::try_from(size).map_err(|_| invalid("negative entry size"))?;
        let _enabled = reader.encoded_byte()?;
        let compression = Compression::try_from(reader.encoded_byte()?)?;
        reader.skip(8)?;
        metadata.push((path, tag, position, size, compression));
    }

    let mut entries = Vec::with_capacity(entry_count);
    for (path, tag, position, size, compression) in metadata {
        let end = position
            .checked_add(size)
            .ok_or_else(|| invalid("entry range overflows"))?;
        let encoded = bytes
            .get(position..end)
            .ok_or_else(|| invalid(format!("entry {path:?} is incomplete")))?;
        let stored = encoded
            .iter()
            .map(|byte| *byte ^ key as u8)
            .collect::<Vec<_>>();
        let bytes = match compression {
            Compression::Raw => stored,
            Compression::Fast => unzip_contents(&stored)?,
        };
        entries.push(ArchiveEntry {
            path,
            tag,
            bytes,
            compression,
        });
    }
    Ok(DecodedArchive { key, entries })
}

fn unzip_contents(bytes: &[u8]) -> Result<Vec<u8>> {
    if let Ok(mut archive) = zip::ZipArchive::new(Cursor::new(bytes)) {
        let mut file = archive.by_name("contents").map_err(zip_error)?;
        let capacity =
            usize::try_from(file.size()).map_err(|_| invalid("ZIP entry is too large"))?;
        let mut contents = Vec::with_capacity(capacity);
        file.read_to_end(&mut contents).map_err(zip_error)?;
        return Ok(contents);
    }

    let descriptor = bytes
        .len()
        .checked_sub(16)
        .filter(|position| bytes.get(*position..*position + 4) == Some(b"PK\x07\x08"))
        .ok_or_else(|| invalid("streaming ZIP data descriptor is missing"))?;
    let compressed_size = u32::from_le_bytes(
        bytes[descriptor + 8..descriptor + 12]
            .try_into()
            .expect("descriptor size was checked"),
    );
    let uncompressed_size = u32::from_le_bytes(
        bytes[descriptor + 12..descriptor + 16]
            .try_into()
            .expect("descriptor size was checked"),
    );
    if bytes.get(..4) != Some(b"PK\x03\x04") {
        return Err(invalid("streaming ZIP local header is missing"));
    }
    let name_length = usize::from(u16::from_le_bytes(read_array(bytes, 26)?));
    let extra_length = usize::from(u16::from_le_bytes(read_array(bytes, 28)?));
    let data_start = 30usize
        .checked_add(name_length)
        .and_then(|value| value.checked_add(extra_length))
        .ok_or_else(|| invalid("streaming ZIP header overflows"))?;
    let compressed_size = usize::try_from(compressed_size)
        .map_err(|_| invalid("streaming ZIP payload is too large"))?;
    let uncompressed_size = usize::try_from(uncompressed_size)
        .map_err(|_| invalid("streaming ZIP contents are too large"))?;
    let data_end = data_start
        .checked_add(compressed_size)
        .filter(|end| *end <= descriptor)
        .ok_or_else(|| invalid("streaming ZIP payload is incomplete"))?;
    let name = bytes
        .get(30..30 + name_length)
        .ok_or_else(|| invalid("streaming ZIP file name is incomplete"))?;
    if name != b"contents" {
        return Err(invalid("unexpected streaming ZIP entry name"));
    }
    let mut decoder = DeflateDecoder::new(&bytes[data_start..data_end]);
    let mut contents = Vec::with_capacity(uncompressed_size);
    decoder.read_to_end(&mut contents).map_err(zip_error)?;
    if contents.len() != uncompressed_size {
        return Err(invalid("streaming ZIP uncompressed size does not match"));
    }
    Ok(contents)
}

fn zip_error(error: impl std::fmt::Display) -> crate::Error {
    invalid(format!("ZIP decoding failed: {error}"))
}

struct CaffReader<'a> {
    bytes: &'a [u8],
    position: usize,
    key: i32,
}

impl<'a> CaffReader<'a> {
    fn new(bytes: &'a [u8], position: usize, key: i32) -> Self {
        Self {
            bytes,
            position,
            key,
        }
    }

    fn encoded_byte(&mut self) -> Result<u8> {
        let byte = self
            .bytes
            .get(self.position)
            .copied()
            .ok_or_else(|| invalid("archive metadata is incomplete"))?;
        self.position += 1;
        Ok(byte ^ self.key as u8)
    }

    fn encoded_i32(&mut self) -> Result<i32> {
        let raw = u32::from_be_bytes(self.array()?);
        Ok((raw ^ self.key as u32) as i32)
    }

    fn encoded_i64(&mut self) -> Result<i64> {
        let raw = u64::from_be_bytes(self.array()?);
        Ok((raw ^ int64_mask(self.key)) as i64)
    }

    fn string(&mut self) -> Result<String> {
        let length = self.number()?;
        let end = self
            .position
            .checked_add(length)
            .ok_or_else(|| invalid("archive string range overflows"))?;
        let encoded = self
            .bytes
            .get(self.position..end)
            .ok_or_else(|| invalid("archive string is incomplete"))?;
        self.position = end;
        let decoded = encoded
            .iter()
            .map(|byte| *byte ^ self.key as u8)
            .collect::<Vec<_>>();
        String::from_utf8(decoded).map_err(|_| invalid("archive string is not UTF-8"))
    }

    fn number(&mut self) -> Result<usize> {
        let mut value = 0usize;
        for _ in 0..4 {
            let byte = self.encoded_byte()?;
            value = value
                .checked_shl(7)
                .and_then(|value| value.checked_add(usize::from(byte & 127)))
                .ok_or_else(|| invalid("archive number overflows"))?;
            if byte & 128 == 0 {
                return Ok(value);
            }
        }
        Err(invalid("archive number is too long"))
    }

    fn skip(&mut self, count: usize) -> Result<()> {
        self.position = self
            .position
            .checked_add(count)
            .filter(|end| *end <= self.bytes.len())
            .ok_or_else(|| invalid("archive metadata is incomplete"))?;
        Ok(())
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N]> {
        let end = self
            .position
            .checked_add(N)
            .ok_or_else(|| invalid("archive metadata range overflows"))?;
        let value = self
            .bytes
            .get(self.position..end)
            .and_then(|bytes| bytes.try_into().ok())
            .ok_or_else(|| invalid("archive metadata is incomplete"))?;
        self.position = end;
        Ok(value)
    }
}

fn read_array<const N: usize>(bytes: &[u8], position: usize) -> Result<[u8; N]> {
    let end = position
        .checked_add(N)
        .ok_or_else(|| invalid("archive header range overflows"))?;
    bytes
        .get(position..end)
        .and_then(|bytes| bytes.try_into().ok())
        .ok_or_else(|| invalid("archive header is incomplete"))
}
