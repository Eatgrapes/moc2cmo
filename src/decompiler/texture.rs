use crate::{Error, Result, moc3::Moc3Model};

const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
const TRANSPARENT_PNG: &[u8] = &[
    137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1, 8, 6, 0,
    0, 0, 31, 21, 196, 137, 0, 0, 0, 11, 73, 68, 65, 84, 120, 218, 99, 96, 0, 2, 0, 0, 5, 0, 1,
    226, 38, 5, 155, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
];

/// One PNG texture atlas page supplied to the decompiler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Texture {
    png: Vec<u8>,
    width: u32,
    height: u32,
}

impl Texture {
    /// Validates the PNG signature and IHDR header, then reads its dimensions.
    ///
    /// # Errors
    ///
    /// Returns an error when the input lacks a valid PNG header or has a zero
    /// width or height.
    pub fn from_png(png: impl Into<Vec<u8>>) -> Result<Self> {
        let png = png.into();
        if png.len() < 24 || png.get(..8) != Some(PNG_SIGNATURE) || png.get(12..16) != Some(b"IHDR")
        {
            return Err(Error::InvalidCmo3(
                "texture is not a valid PNG header".into(),
            ));
        }
        let width = u32::from_be_bytes(png[16..20].try_into().expect("validated PNG width"));
        let height = u32::from_be_bytes(png[20..24].try_into().expect("validated PNG height"));
        if width == 0 || height == 0 {
            return Err(Error::InvalidCmo3(
                "texture dimensions must be non-zero".into(),
            ));
        }
        Ok(Self { png, width, height })
    }

    /// Returns the encoded PNG bytes.
    pub fn png(&self) -> &[u8] {
        &self.png
    }

    /// Returns the pixel width.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Returns the pixel height.
    pub fn height(&self) -> u32 {
        self.height
    }

    fn transparent() -> Self {
        Self {
            png: TRANSPARENT_PNG.to_vec(),
            width: 1,
            height: 1,
        }
    }
}

pub(super) fn complete_texture_set(model: &Moc3Model, supplied: &[Texture]) -> Vec<Texture> {
    let required = model
        .art_meshes()
        .iter()
        .map(|mesh| mesh.texture_index())
        .max()
        .map_or(0, |index| index + 1);
    let mut textures = supplied.to_vec();
    textures.resize_with(required, Texture::transparent);
    textures
}
