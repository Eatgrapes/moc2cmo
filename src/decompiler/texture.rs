use std::io::Cursor;

use image::{DynamicImage, ImageFormat, RgbaImage};

use crate::{Error, Result, moc3::Moc3Model};

/// One PNG texture atlas page supplied to the decompiler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Texture {
    png: Vec<u8>,
    width: u32,
    height: u32,
}

impl Texture {
    /// Decodes a PNG texture and reads its dimensions.
    ///
    /// # Errors
    ///
    /// Returns an error when the input is not a valid PNG image.
    pub fn from_png(png: impl Into<Vec<u8>>) -> Result<Self> {
        let png = png.into();
        let image = image::load_from_memory_with_format(&png, ImageFormat::Png)
            .map_err(|error| Error::InvalidCmo3(format!("texture PNG decode failed: {error}")))?
            .into_rgba8();
        let (width, height) = image.dimensions();
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
        let image = RgbaImage::new(1, 1);
        let mut png = Cursor::new(Vec::new());
        DynamicImage::ImageRgba8(image)
            .write_to(&mut png, ImageFormat::Png)
            .expect("encoding a PNG into memory cannot fail");
        Self::from_png(png.into_inner()).expect("image encoded this PNG")
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
