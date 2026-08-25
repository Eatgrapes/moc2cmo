use std::path::Path;

use quick_xml::escape::escape;

use crate::{
    Error, Result,
    caff::{ArchiveEntry, Compression, decode_archive, encode_archive},
};

/// A decoded Cubism `.can3` animation project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Can3Project {
    main_xml: Vec<u8>,
    resources: Vec<AnimationResource>,
    obfuscation_key: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AnimationResource {
    path: String,
    tag: String,
    bytes: Vec<u8>,
    compression: Compression,
}

impl Can3Project {
    /// Decodes a complete `.can3` CAFF archive.
    ///
    /// Both Cubism's streaming ZIP payloads and regular ZIP payloads are
    /// accepted.
    ///
    /// # Errors
    ///
    /// Returns an error when the CAFF archive or its `main.xml` entry is
    /// malformed.
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let archive = decode_archive(bytes)?;
        let mut main_xml = None;
        let mut resources = Vec::new();
        for entry in archive.entries {
            if entry.path == "main.xml" {
                if main_xml.replace(entry.bytes).is_some() {
                    return Err(Error::InvalidCan3("multiple main.xml entries".into()));
                }
            } else {
                resources.push(AnimationResource {
                    path: entry.path,
                    tag: entry.tag,
                    bytes: entry.bytes,
                    compression: entry.compression,
                });
            }
        }
        let main_xml = main_xml.ok_or_else(|| Error::InvalidCan3("main.xml is missing".into()))?;
        std::str::from_utf8(&main_xml)
            .map_err(|_| Error::InvalidCan3("main.xml is not UTF-8".into()))?;
        Ok(Self {
            main_xml,
            resources,
            obfuscation_key: archive.key,
        })
    }

    /// Returns the decoded animation XML.
    pub fn main_xml(&self) -> &[u8] {
        &self.main_xml
    }

    /// Returns the signed CAFF XOR key used when encoding.
    pub fn obfuscation_key(&self) -> i32 {
        self.obfuscation_key
    }

    /// Sets the signed CAFF XOR key used when encoding.
    pub fn set_obfuscation_key(&mut self, key: i32) {
        self.obfuscation_key = key;
    }

    /// Changes the linked `.cmo3` model path stored by the animation project.
    ///
    /// Relative paths are resolved by Cubism from the animation project's
    /// directory.
    ///
    /// # Errors
    ///
    /// Returns an error when the path is empty or the animation XML does not
    /// contain a linked-model resource.
    pub fn relink_model(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let path_text = path.to_string_lossy();
        if path_text.is_empty() {
            return Err(Error::InvalidCan3("linked model path is empty".into()));
        }
        let file_name = path
            .file_name()
            .map(|name| name.to_string_lossy())
            .unwrap_or_else(|| path_text.clone());
        let xml = std::str::from_utf8(&self.main_xml)
            .map_err(|_| Error::InvalidCan3("main.xml is not UTF-8".into()))?;
        let block_start = xml
            .find("<CResource_Linked_Model xs.n=\"resourceRef\">")
            .ok_or_else(|| Error::InvalidCan3("linked-model resource is missing".into()))?;
        let relative_end = xml[block_start..]
            .find("</CResource_Linked_Model>")
            .ok_or_else(|| Error::InvalidCan3("linked-model resource is incomplete".into()))?;
        let block_end = block_start + relative_end + "</CResource_Linked_Model>".len();
        let block = &xml[block_start..block_end];
        let block = replace_text_element(block, "file", "srcFile", &path_text)?;
        let block = replace_text_element(&block, "s", "name", &file_name)?;
        let mut updated =
            String::with_capacity(xml.len() - (block_end - block_start) + block.len());
        updated.push_str(&xml[..block_start]);
        updated.push_str(&block);
        updated.push_str(&xml[block_end..]);
        self.main_xml = updated.into_bytes();
        Ok(())
    }

    /// Encodes the project as a complete `.can3` byte buffer.
    ///
    /// # Errors
    ///
    /// Returns an error when the CAFF archive cannot be encoded.
    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut entries = self
            .resources
            .iter()
            .map(|resource| ArchiveEntry {
                path: resource.path.clone(),
                tag: resource.tag.clone(),
                bytes: resource.bytes.clone(),
                compression: resource.compression,
            })
            .collect::<Vec<_>>();
        entries.push(ArchiveEntry {
            path: "main.xml".into(),
            tag: "main_xml".into(),
            bytes: self.main_xml.clone(),
            compression: Compression::Fast,
        });
        encode_archive(entries, self.obfuscation_key)
    }

    /// Encodes and writes the project to a `.can3` file.
    ///
    /// # Errors
    ///
    /// Returns an error when encoding fails or the destination cannot be
    /// written.
    pub fn write_to(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let bytes = self.encode()?;
        std::fs::write(path, bytes).map_err(|source| Error::Io {
            path: path.to_path_buf(),
            source,
        })
    }
}

fn replace_text_element(block: &str, tag: &str, name: &str, value: &str) -> Result<String> {
    let start_marker = format!("<{tag} xs.n=\"{name}\">");
    let end_marker = format!("</{tag}>");
    let start = block
        .find(&start_marker)
        .ok_or_else(|| Error::InvalidCan3(format!("{name} element is missing")))?;
    let value_start = start + start_marker.len();
    let relative_end = block[value_start..]
        .find(&end_marker)
        .ok_or_else(|| Error::InvalidCan3(format!("{name} element is incomplete")))?;
    let value_end = value_start + relative_end;
    let escaped = escape(value);
    let mut output = String::with_capacity(block.len() - (value_end - value_start) + escaped.len());
    output.push_str(&block[..value_start]);
    output.push_str(&escaped);
    output.push_str(&block[value_end..]);
    Ok(output)
}
