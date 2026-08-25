use crate::{Error, Result};

use super::Endianness;

pub(super) struct Reader<'a> {
    bytes: &'a [u8],
    endianness: Endianness,
}

impl<'a> Reader<'a> {
    pub(super) fn new(bytes: &'a [u8], endianness: Endianness) -> Self {
        Self { bytes, endianness }
    }

    pub(super) fn len(&self) -> usize {
        self.bytes.len()
    }

    pub(super) fn u8(&self, offset: usize) -> Result<u8> {
        self.bytes
            .get(offset)
            .copied()
            .ok_or_else(|| invalid(format!("byte at 0x{offset:x} is outside the file")))
    }

    pub(super) fn i16(&self, offset: usize) -> Result<i16> {
        let raw = self.array(offset)?;
        Ok(match self.endianness {
            Endianness::Little => i16::from_le_bytes(raw),
            Endianness::Big => i16::from_be_bytes(raw),
        })
    }

    pub(super) fn u16(&self, offset: usize) -> Result<u16> {
        let raw = self.array(offset)?;
        Ok(match self.endianness {
            Endianness::Little => u16::from_le_bytes(raw),
            Endianness::Big => u16::from_be_bytes(raw),
        })
    }

    pub(super) fn i32(&self, offset: usize) -> Result<i32> {
        let raw = self.array(offset)?;
        Ok(match self.endianness {
            Endianness::Little => i32::from_le_bytes(raw),
            Endianness::Big => i32::from_be_bytes(raw),
        })
    }

    pub(super) fn u32(&self, offset: usize) -> Result<u32> {
        let raw = self.array(offset)?;
        Ok(match self.endianness {
            Endianness::Little => u32::from_le_bytes(raw),
            Endianness::Big => u32::from_be_bytes(raw),
        })
    }

    pub(super) fn f32(&self, offset: usize) -> Result<f32> {
        let raw = self.array(offset)?;
        Ok(match self.endianness {
            Endianness::Little => f32::from_le_bytes(raw),
            Endianness::Big => f32::from_be_bytes(raw),
        })
    }

    pub(super) fn section_i16(&self, offset: usize, count: usize) -> Result<Vec<i16>> {
        self.section(offset, count, 2, Self::i16)
    }

    pub(super) fn section_u16(&self, offset: usize, count: usize) -> Result<Vec<u16>> {
        self.section(offset, count, 2, Self::u16)
    }

    pub(super) fn section_i32(&self, offset: usize, count: usize) -> Result<Vec<i32>> {
        self.section(offset, count, 4, Self::i32)
    }

    pub(super) fn section_u32(&self, offset: usize, count: usize) -> Result<Vec<u32>> {
        self.section(offset, count, 4, Self::u32)
    }

    pub(super) fn section_f32(&self, offset: usize, count: usize) -> Result<Vec<f32>> {
        self.section(offset, count, 4, Self::f32)
    }

    pub(super) fn section_u8(&self, offset: usize, count: usize) -> Result<Vec<u8>> {
        self.section(offset, count, 1, Self::u8)
    }

    pub(super) fn str64(&self, offset: usize) -> Result<String> {
        let end = offset
            .checked_add(64)
            .ok_or_else(|| invalid("string range overflows"))?;
        let raw = self
            .bytes
            .get(offset..end)
            .ok_or_else(|| invalid(format!("string at 0x{offset:x} is incomplete")))?;
        let len = raw.iter().position(|byte| *byte == 0).unwrap_or(raw.len());
        Ok(String::from_utf8_lossy(&raw[..len]).into_owned())
    }

    fn section<T>(
        &self,
        offset: usize,
        count: usize,
        width: usize,
        read: impl Fn(&Self, usize) -> Result<T>,
    ) -> Result<Vec<T>> {
        let byte_len = count
            .checked_mul(width)
            .ok_or_else(|| invalid("section size overflows"))?;
        let end = offset
            .checked_add(byte_len)
            .ok_or_else(|| invalid("section range overflows"))?;
        if end > self.bytes.len() {
            return Err(invalid(format!("section at 0x{offset:x} is incomplete")));
        }

        let mut values = Vec::with_capacity(count);
        for index in 0..count {
            values.push(read(self, offset + index * width)?);
        }
        Ok(values)
    }

    fn array<const N: usize>(&self, offset: usize) -> Result<[u8; N]> {
        let end = offset
            .checked_add(N)
            .ok_or_else(|| invalid("fixed-width read overflows"))?;
        self.bytes
            .get(offset..end)
            .and_then(|slice| slice.try_into().ok())
            .ok_or_else(|| invalid(format!("read at 0x{offset:x} is incomplete")))
    }
}

pub(super) fn invalid(message: impl Into<String>) -> Error {
    Error::InvalidMoc3(message.into())
}
