use std::path::Path;

use quick_xml::Reader;

use crate::{Error, Result};

use crate::caff::{ArchiveEntry, Compression, encode_archive};

/// A CMO3 project before it is encoded into a CAFF archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cmo3Project {
    main_xml: Vec<u8>,
    resources: Vec<ProjectResource>,
    obfuscation_key: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProjectResource {
    path: String,
    bytes: Vec<u8>,
}

impl Cmo3Project {
    /// Creates a project from its UTF-8 `main.xml` document.
    pub fn new(main_xml: impl Into<Vec<u8>>) -> Self {
        Self {
            main_xml: main_xml.into(),
            resources: Vec::new(),
            obfuscation_key: 42,
        }
    }

    /// Sets the signed 32-bit XOR key stored in the CAFF header.
    pub fn set_obfuscation_key(&mut self, key: i32) {
        self.obfuscation_key = key;
    }

    /// Returns the current CAFF obfuscation key.
    pub fn obfuscation_key(&self) -> i32 {
        self.obfuscation_key
    }

    /// Adds or replaces a binary resource at an archive-relative path.
    pub fn insert_resource(&mut self, path: impl Into<String>, bytes: impl Into<Vec<u8>>) {
        let path = path.into();
        let bytes = bytes.into();
        if let Some(resource) = self.resources.iter_mut().find(|entry| entry.path == path) {
            resource.bytes = bytes;
        } else {
            self.resources.push(ProjectResource { path, bytes });
        }
    }

    /// Returns the uncompressed `main.xml` bytes.
    pub fn main_xml(&self) -> &[u8] {
        &self.main_xml
    }

    /// Validates the project's UTF-8 XML document.
    pub fn validate_xml(&self) -> Result<()> {
        let mut reader = Reader::from_reader(self.main_xml.as_slice());
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer) {
                Ok(quick_xml::events::Event::Eof) => break,
                Ok(_) => buffer.clear(),
                Err(error) => {
                    return Err(Error::InvalidCmo3(format!("main.xml is invalid: {error}")));
                }
            }
        }
        Ok(())
    }

    /// Encodes the project as a complete `.cmo3` byte buffer.
    pub fn encode(&self) -> Result<Vec<u8>> {
        if self.main_xml.is_empty() {
            return Err(Error::InvalidCmo3("main.xml is empty".into()));
        }
        self.validate_xml()?;

        let mut entries = Vec::with_capacity(self.resources.len() + 1);
        for resource in &self.resources {
            validate_resource_path(&resource.path)?;
            entries.push(ArchiveEntry {
                path: resource.path.clone(),
                tag: String::new(),
                bytes: resource.bytes.clone(),
                compression: Compression::Raw,
            });
        }
        entries.push(ArchiveEntry {
            path: "main.xml".into(),
            tag: "main_xml".into(),
            bytes: self.main_xml.clone(),
            compression: Compression::Fast,
        });

        encode_archive(entries, self.obfuscation_key)
    }

    /// Encodes and writes the project to a `.cmo3` file.
    pub fn write_to(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let bytes = self.encode()?;
        std::fs::write(path, bytes).map_err(|source| Error::Io {
            path: path.to_path_buf(),
            source,
        })
    }
}

fn validate_resource_path(path: &str) -> Result<()> {
    if path.is_empty() {
        return Err(Error::InvalidCmo3("resource path is empty".into()));
    }
    let normalized = path.replace('\\', "/");
    if normalized.starts_with('/')
        || normalized.split('/').any(|component| component == "..")
        || normalized == "main.xml"
    {
        return Err(Error::InvalidCmo3(format!(
            "invalid resource path {path:?}"
        )));
    }
    Ok(())
}
