mod geometry;
mod texture;
mod xml;

use std::path::Path;

use crate::{
    Error, Result, can3::Can3Project, cmo3::Cmo3Project, moc3::Moc3Model, model3::Model3,
    motion3::Motion3,
};

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

/// Reads a Cubism `model3.json` manifest and writes matching CMO3 and CAN3 files.
///
/// All paths in the manifest are resolved relative to the manifest's directory.
/// The output directory is created when it does not exist. The generated files
/// use the manifest file stem as their base name.
///
/// # Errors
///
/// Returns an error when a referenced model, texture, motion, or output file
/// cannot be read or written.
pub fn decompile_model3_to_files(
    model3_path: impl AsRef<Path>,
    output_directory: impl AsRef<Path>,
) -> Result<()> {
    let model3_path = model3_path.as_ref();
    let base_directory = model3_path.parent().unwrap_or_else(|| Path::new("."));
    let manifest_bytes = std::fs::read(model3_path).map_err(|source| Error::Io {
        path: model3_path.to_path_buf(),
        source,
    })?;
    let manifest = Model3::from_json_bytes(&manifest_bytes)?;
    let references = manifest.file_references();
    let moc3_path = base_directory.join(&references.moc);
    let moc3 = std::fs::read(&moc3_path).map_err(|source| Error::Io {
        path: moc3_path.clone(),
        source,
    })?;

    let mut decompiler = Decompiler::new();
    let model_name = model3_path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("model");
    decompiler.set_model_name(model_name);
    for texture in &references.textures {
        let path = base_directory.join(texture);
        let bytes = std::fs::read(&path).map_err(|source| Error::Io {
            path: path.clone(),
            source,
        })?;
        decompiler.push_texture(Texture::from_png(bytes)?);
    }

    let output_directory = output_directory.as_ref();
    std::fs::create_dir_all(output_directory).map_err(|source| Error::Io {
        path: output_directory.to_path_buf(),
        source,
    })?;
    let cmo3_path = output_directory.join(format!("{model_name}.cmo3"));
    let mut cmo3 = decompiler.decompile_project(&moc3)?;
    cmo3.insert_resource("moc2cmo.model3.json", manifest_bytes);
    for resource in referenced_json_paths(references) {
        let path = base_directory.join(&resource);
        let bytes = std::fs::read(&path).map_err(|source| Error::Io {
            path: path.clone(),
            source,
        })?;
        cmo3.insert_resource(resource, bytes);
    }
    cmo3.write_to(&cmo3_path)?;

    let mut motions = Vec::new();
    for (group, entries) in &references.motions {
        for (index, entry) in entries.iter().enumerate() {
            let path = base_directory.join(&entry.file);
            let bytes = std::fs::read(&path).map_err(|source| Error::Io {
                path: path.clone(),
                source,
            })?;
            let motion = Motion3::from_json_bytes(&bytes)?;
            motions.push(crate::can3::MotionInstance {
                name: format!("{group}_{index}"),
                motion,
                fade_in_time: entry.fade_in_time,
                fade_out_time: entry.fade_out_time,
            });
        }
    }
    let model_file_name = cmo3_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("model.cmo3");
    let can3 = Can3Project::from_model3(model_name, model_file_name, &motions, manifest.groups())?;
    can3.write_to(output_directory.join(format!("{model_name}.can3")))
}

fn referenced_json_paths(references: &crate::model3::Model3References) -> Vec<String> {
    let mut paths = Vec::new();
    for path in [
        references.physics.clone(),
        references.pose.clone(),
        references.user_data.clone(),
        references.display_info.clone(),
    ]
    .into_iter()
    .flatten()
    {
        paths.push(path);
    }
    paths.extend(
        references
            .expressions
            .iter()
            .map(|expression| expression.file.clone()),
    );
    paths
}
