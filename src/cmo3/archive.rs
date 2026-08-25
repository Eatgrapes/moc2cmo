use std::io::{Cursor, Write};

use byteorder::{BigEndian, WriteBytesExt};
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

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
    let mut archive = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Some(1));
    archive.start_file("contents", options).map_err(zip_error)?;
    archive.write_all(content).map_err(zip_error)?;
    archive.finish().map_err(zip_error).map(Cursor::into_inner)
}

fn zip_error(error: impl std::fmt::Display) -> Error {
    Error::InvalidCmo3(format!("ZIP encoding failed: {error}"))
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
        self.bytes.write_u16::<BigEndian>(encoded).unwrap();
    }

    fn i32(&mut self, value: i32, key: i32) {
        let encoded = value as u32 ^ key as u32;
        self.bytes.write_u32::<BigEndian>(encoded).unwrap();
    }

    fn i64(&mut self, value: i64, key: i32) {
        let encoded = value as u64 ^ int64_mask(key);
        self.bytes.write_u64::<BigEndian>(encoded).unwrap();
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
