use std::io::Write;

use flate2::{Compression as DeflateLevel, write::DeflateEncoder};

use crate::{Error, Result};

const MAGIC: &[u8; 4] = b"CAFF";
const FORMAT_ID: &[u8; 4] = b"----";
const NO_PREVIEW: u8 = 127;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) enum Compression {
    Raw = 16,
    Fast = 33,
}

pub(super) struct ArchiveEntry {
    pub(super) path: String,
    pub(super) tag: String,
    pub(super) bytes: Vec<u8>,
    pub(super) compression: Compression,
}

struct StoredEntry {
    path: String,
    tag: String,
    bytes: Vec<u8>,
    compression: Compression,
    position_patch: usize,
}

pub(super) fn encode_archive(entries: Vec<ArchiveEntry>, key: i32) -> Result<Vec<u8>> {
    let mut entries = entries
        .into_iter()
        .map(|entry| {
            let bytes = match entry.compression {
                Compression::Raw => entry.bytes,
                Compression::Fast => zip_contents(&entry.bytes)?,
            };
            Ok(StoredEntry {
                path: entry.path,
                tag: entry.tag,
                bytes,
                compression: entry.compression,
                position_patch: 0,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let mut writer = CaffWriter::new();
    writer.bytes(MAGIC, 0);
    writer.bytes(&[0, 0, 0], 0);
    writer.bytes(FORMAT_ID, 0);
    writer.bytes(&[0, 0, 0], 0);
    writer.i32(key, 0);
    writer.zeros(8);

    writer.byte(NO_PREVIEW, 0);
    writer.byte(NO_PREVIEW, 0);
    writer.zeros(2);
    writer.i16(0, 0);
    writer.i16(0, 0);
    writer.i64(0, 0);
    writer.i32(0, 0);
    writer.zeros(8);

    let entry_count = i32::try_from(entries.len())
        .map_err(|_| Error::InvalidCmo3("too many archive entries".into()))?;
    writer.i32(entry_count, key);
    for entry in &mut entries {
        writer.string(&entry.path, key)?;
        writer.string(&entry.tag, key)?;
        entry.position_patch = writer.position();
        writer.i64(0, key);
        let size = i32::try_from(entry.bytes.len())
            .map_err(|_| Error::InvalidCmo3(format!("resource {:?} is too large", entry.path)))?;
        writer.i32(size, key);
        writer.byte(1, key);
        writer.byte(entry.compression as u8, key);
        writer.zeros(8);
    }

    for entry in &entries {
        let start = writer.position();
        writer.patch_i64(entry.position_patch, start as i64, key);
        writer.bytes(&entry.bytes, key);
    }
    writer.bytes(&[98, 99], 0);
    Ok(writer.finish())
}

fn zip_contents(content: &[u8]) -> Result<Vec<u8>> {
    const NAME: &[u8] = b"contents";

    let mut encoder = DeflateEncoder::new(Vec::new(), DeflateLevel::fast());
    encoder
        .write_all(content)
        .map_err(|error| Error::InvalidCmo3(format!("deflate failed: {error}")))?;
    let compressed = encoder
        .finish()
        .map_err(|error| Error::InvalidCmo3(format!("deflate failed: {error}")))?;
    let crc = crc32fast::hash(content);
    let compressed_len = u32::try_from(compressed.len())
        .map_err(|_| Error::InvalidCmo3("compressed main.xml is too large".into()))?;
    let content_len = u32::try_from(content.len())
        .map_err(|_| Error::InvalidCmo3("main.xml is too large".into()))?;
    let name_len = u16::try_from(NAME.len()).expect("contents filename fits in u16");

    let mut zip = Vec::with_capacity(compressed.len() + 128);
    push_u32_le(&mut zip, 0x0403_4b50);
    push_u16_le(&mut zip, 20);
    push_u16_le(&mut zip, 0);
    push_u16_le(&mut zip, 8);
    push_u16_le(&mut zip, 0);
    push_u16_le(&mut zip, 0);
    push_u32_le(&mut zip, crc);
    push_u32_le(&mut zip, compressed_len);
    push_u32_le(&mut zip, content_len);
    push_u16_le(&mut zip, name_len);
    push_u16_le(&mut zip, 0);
    zip.extend_from_slice(NAME);
    zip.extend_from_slice(&compressed);

    let central_offset = u32::try_from(zip.len())
        .map_err(|_| Error::InvalidCmo3("ZIP offset is too large".into()))?;
    push_u32_le(&mut zip, 0x0201_4b50);
    push_u16_le(&mut zip, 20);
    push_u16_le(&mut zip, 20);
    push_u16_le(&mut zip, 0);
    push_u16_le(&mut zip, 8);
    push_u16_le(&mut zip, 0);
    push_u16_le(&mut zip, 0);
    push_u32_le(&mut zip, crc);
    push_u32_le(&mut zip, compressed_len);
    push_u32_le(&mut zip, content_len);
    push_u16_le(&mut zip, name_len);
    push_u16_le(&mut zip, 0);
    push_u16_le(&mut zip, 0);
    push_u16_le(&mut zip, 0);
    push_u16_le(&mut zip, 0);
    push_u32_le(&mut zip, 0);
    push_u32_le(&mut zip, 0);
    zip.extend_from_slice(NAME);

    let central_size = u32::try_from(zip.len())
        .ok()
        .and_then(|end| end.checked_sub(central_offset))
        .ok_or_else(|| Error::InvalidCmo3("ZIP directory size overflows".into()))?;
    push_u32_le(&mut zip, 0x0605_4b50);
    push_u16_le(&mut zip, 0);
    push_u16_le(&mut zip, 0);
    push_u16_le(&mut zip, 1);
    push_u16_le(&mut zip, 1);
    push_u32_le(&mut zip, central_size);
    push_u32_le(&mut zip, central_offset);
    push_u16_le(&mut zip, 0);
    Ok(zip)
}

struct CaffWriter {
    bytes: Vec<u8>,
}

impl CaffWriter {
    fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    fn position(&self) -> usize {
        self.bytes.len()
    }

    fn byte(&mut self, value: u8, key: i32) {
        self.bytes.push(value ^ key as u8);
    }

    fn i16(&mut self, value: i16, key: i32) {
        let encoded = value as u16 ^ key as u16;
        self.bytes.extend_from_slice(&encoded.to_be_bytes());
    }

    fn i32(&mut self, value: i32, key: i32) {
        let encoded = value as u32 ^ key as u32;
        self.bytes.extend_from_slice(&encoded.to_be_bytes());
    }

    fn i64(&mut self, value: i64, key: i32) {
        let encoded = value as u64 ^ int64_mask(key);
        self.bytes.extend_from_slice(&encoded.to_be_bytes());
    }

    fn bytes(&mut self, bytes: &[u8], key: i32) {
        if key == 0 {
            self.bytes.extend_from_slice(bytes);
        } else {
            self.bytes
                .extend(bytes.iter().map(|byte| *byte ^ key as u8));
        }
    }

    fn string(&mut self, value: &str, key: i32) -> Result<()> {
        self.number(value.len(), key)?;
        self.bytes(value.as_bytes(), key);
        Ok(())
    }

    fn number(&mut self, value: usize, key: i32) -> Result<()> {
        if value >= 1 << 28 {
            return Err(Error::InvalidCmo3("archive string is too long".into()));
        }
        if value < 128 {
            self.byte(value as u8, key);
        } else if value < 16_384 {
            self.byte(((value >> 7) as u8 & 127) | 128, key);
            self.byte((value as u8) & 127, key);
        } else if value < 2_097_152 {
            self.byte(((value >> 14) as u8 & 127) | 128, key);
            self.byte(((value >> 7) as u8 & 127) | 128, key);
            self.byte((value as u8) & 127, key);
        } else {
            self.byte(((value >> 21) as u8 & 127) | 128, key);
            self.byte(((value >> 14) as u8 & 127) | 128, key);
            self.byte(((value >> 7) as u8 & 127) | 128, key);
            self.byte((value as u8) & 127, key);
        }
        Ok(())
    }

    fn zeros(&mut self, count: usize) {
        self.bytes.resize(self.bytes.len() + count, 0);
    }

    fn patch_i64(&mut self, offset: usize, value: i64, key: i32) {
        let encoded = (value as u64 ^ int64_mask(key)).to_be_bytes();
        self.bytes[offset..offset + 8].copy_from_slice(&encoded);
    }

    fn finish(self) -> Vec<u8> {
        self.bytes
    }
}

fn int64_mask(key: i32) -> u64 {
    if key < 0 {
        (u64::from(u32::MAX) << 32) | u64::from(key as u32)
    } else {
        let key = u64::from(key as u32);
        (key << 32) | key
    }
}

fn push_u16_le(target: &mut Vec<u8>, value: u16) {
    target.extend_from_slice(&value.to_le_bytes());
}

fn push_u32_le(target: &mut Vec<u8>, value: u32) {
    target.extend_from_slice(&value.to_le_bytes());
}
