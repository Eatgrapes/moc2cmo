mod reader;
mod writer;

pub(crate) use reader::decode_archive;
pub(crate) use writer::encode_archive;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum Compression {
    Raw = 16,
    Fast = 33,
}

impl TryFrom<u8> for Compression {
    type Error = crate::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            16 => Ok(Self::Raw),
            33 => Ok(Self::Fast),
            _ => Err(crate::Error::InvalidCaff(format!(
                "unsupported entry compression {value}"
            ))),
        }
    }
}

pub(crate) struct ArchiveEntry {
    pub(crate) path: String,
    pub(crate) tag: String,
    pub(crate) bytes: Vec<u8>,
    pub(crate) compression: Compression,
}

pub(crate) fn int64_mask(key: i32) -> u64 {
    if key < 0 {
        (u64::from(u32::MAX) << 32) | u64::from(key as u32)
    } else {
        let key = u64::from(key as u32);
        (key << 32) | key
    }
}

pub(crate) fn invalid(message: impl Into<String>) -> crate::Error {
    crate::Error::InvalidCaff(message.into())
}
