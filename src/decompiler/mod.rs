mod geometry;
mod texture;
mod xml;

use std::path::Path;

use crate::{Error, Result, cmo3::Cmo3Project, moc3::Moc3Model};

pub use texture::Texture;

/// Converts MOC3 bytes into a CMO3 project.
#[derive(Debug, Clone)]
pub struct Decompiler {
    model_name: String,
    textures: Vec<Texture>,
    obfuscation_key: i32,
}

impl Default for Decompiler {
    fn default() -> Self {
        Self {
            model_name: "Decompiled Model".into(),
            textures: Vec::new(),
            obfuscation_key: 42,
        }
    }
}

impl Decompiler {
    /// Creates a decompiler with default options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the display name stored in the generated project.
    pub fn set_model_name(&mut self, name: impl Into<String>) {
        self.model_name = name.into();
    }

    /// Replaces the texture atlas pages in MOC3 texture-index order.
    ///
    /// Missing pages are replaced with a transparent placeholder. Extra pages
    /// are retained in the generated project.
    pub fn set_textures(&mut self, textures: impl IntoIterator<Item = Texture>) {
        self.textures = textures.into_iter().collect();
    }

    /// Adds one texture atlas page.
    pub fn push_texture(&mut self, texture: Texture) {
        self.textures.push(texture);
    }

    /// Sets the signed CAFF XOR key.
    pub fn set_obfuscation_key(&mut self, key: i32) {
        self.obfuscation_key = key;
    }

    /// Parses the MOC3 model and builds an unencoded CMO3 project.
    ///
    /// The project contains a readable `main.xml`, the supplied atlas pages,
    /// and `moc2cmo.model.json` with the recovered runtime data.
    ///
    /// # Errors
    ///
    /// Returns an error when the MOC3 data is invalid or a project resource
    /// cannot be serialized.
    pub fn decompile_project(&self, moc3: &[u8]) -> Result<Cmo3Project> {
        let model = Moc3Model::parse(moc3)?;
        let textures = texture::complete_texture_set(&model, &self.textures);
        let main_xml = xml::generate(&model, &self.model_name, &textures)?;
        let mut project = Cmo3Project::new(main_xml);
        project.set_obfuscation_key(self.obfuscation_key);
        for (index, texture) in textures.iter().enumerate() {
            project.insert_resource(format!("imageFileBuf_{index}.png"), texture.png().to_vec());
        }
        let manifest = serde_json::to_vec_pretty(&model).map_err(|error| {
            Error::InvalidCmo3(format!("manifest serialization failed: {error}"))
        })?;
        project.insert_resource("moc2cmo.model.json", manifest);
        Ok(project)
    }

    /// Decompiles a MOC3 model into encoded `.cmo3` bytes.
    ///
    /// # Errors
    ///
    /// Returns an error when the input cannot be parsed or the CMO3 archive
    /// cannot be encoded.
    pub fn decompile(&self, moc3: &[u8]) -> Result<Vec<u8>> {
        self.decompile_project(moc3)?.encode()
    }

    /// Decompiles and writes a `.cmo3` file.
    ///
    /// # Errors
    ///
    /// Returns an error when conversion fails or the destination cannot be
    /// written.
    pub fn decompile_to_file(&self, moc3: &[u8], path: impl AsRef<Path>) -> Result<()> {
        self.decompile_project(moc3)?.write_to(path)
    }
}

/// Decompiles a MOC3 model with default options.
///
/// Texture pages are replaced with transparent placeholders. Use
/// [`Decompiler`] when atlas PNGs are available.
///
/// # Errors
///
/// Returns an error when the input cannot be parsed or the CMO3 archive cannot
/// be encoded.
pub fn decompile(moc3: &[u8]) -> Result<Vec<u8>> {
    Decompiler::new().decompile(moc3)
}

/// Decompiles a MOC3 model with default options and writes it to disk.
///
/// Texture pages are replaced with transparent placeholders. Use
/// [`Decompiler`] when atlas PNGs are available.
///
/// # Errors
///
/// Returns an error when conversion fails or the destination cannot be
/// written.
pub fn decompile_to_file(moc3: &[u8], path: impl AsRef<Path>) -> Result<()> {
    Decompiler::new().decompile_to_file(moc3, path)
}
