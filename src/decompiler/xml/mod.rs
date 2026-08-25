mod constants;
mod document;
mod entities;
mod grid;
mod plan;
mod texture;
mod writer;

use crate::{Result, moc3::Moc3Model};

use self::{
    constants::{IMPORT_INSTRUCTIONS, VERSION_INSTRUCTIONS},
    plan::ProjectPlan,
    writer::{XmlWriter, attr},
};
use super::Texture;

pub(super) fn generate(
    model: &Moc3Model,
    model_name: &str,
    textures: &[Texture],
) -> Result<Vec<u8>> {
    let plan = ProjectPlan::new(model, textures.len());
    let mut xml = XmlWriter::new();
    xml.declaration("<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
    for (name, version) in VERSION_INSTRUCTIONS {
        xml.declaration(&format!("<?version {name}:{version}?>"));
    }
    for import in IMPORT_INSTRUCTIONS {
        xml.declaration(&format!("<?import {import}?>"));
    }
    xml.start("root", &[attr("fileFormatVersion", "402030000")]);
    xml.start("shared", &[]);
    entities::write_shared(&mut xml, &plan, model)?;
    texture::write_texture_shared(&mut xml, &plan, model, textures);
    xml.end("shared");
    xml.start("main", &[]);
    document::write_main(&mut xml, &plan, model, model_name);
    xml.end("main");
    xml.end("root");
    Ok(xml.finish())
}
