mod atlas;
mod cache;
mod filters;
mod layers;
mod model_image;

use uuid::Uuid;

use crate::{decompiler::Texture, moc3::Moc3Model};

use super::{
    grid::{empty_shared, reference, start_shared, start_shared_with},
    plan::{PagePlan, ProjectPlan},
    writer::{XmlWriter, attr},
};

pub(super) fn write_texture_shared(
    xml: &mut XmlWriter,
    plan: &ProjectPlan,
    model: &Moc3Model,
    textures: &[Texture],
) {
    filters::write_globals(xml, &plan.filters);
    empty_shared(
        xml,
        "CLayeredImageGuid",
        plan.layered_image_guid,
        vec![
            attr("uuid", Uuid::new_v4()),
            attr("note", "moc2cmo source layers"),
        ],
    );
    for (index, (page, texture)) in plan.pages.iter().zip(textures).enumerate() {
        write_page(xml, plan, page, texture, index);
    }
    layers::write_group(xml, plan);
    layers::write_layered_image(xml, plan, model, textures.len());
    model_image::write_group(xml, plan, textures);
}

fn write_page(
    xml: &mut XmlWriter,
    project: &ProjectPlan,
    page: &PagePlan,
    texture: &Texture,
    index: usize,
) {
    empty_shared(
        xml,
        "CModelImageGuid",
        page.model_image_guid,
        vec![
            attr("uuid", Uuid::new_v4()),
            attr("note", format!("Texture {index}")),
        ],
    );
    empty_shared(
        xml,
        "GTextureGuid",
        page.texture_guid,
        vec![
            attr("uuid", Uuid::new_v4()),
            attr("note", format!("Texture {index}")),
        ],
    );
    start_shared_with(
        xml,
        "CImageResource",
        page.image_resource,
        vec![
            attr("width", texture.width()),
            attr("height", texture.height()),
            attr("type", "INT_ARGB"),
            attr("imageFileBuf_size", texture.png().len()),
            attr("previewFileBuf_size", 0),
        ],
    );
    xml.empty(
        "file",
        &[
            attr("xs.n", "imageFileBuf"),
            attr("path", format!("imageFileBuf_{index}.png")),
        ],
    );
    xml.end("CImageResource");

    atlas::write(xml, page, texture, index);
    layers::write_layer(xml, project, page, texture, index);
    filters::write_page(xml, project, page, index);
    write_texture(xml, page, index);
}

fn write_texture(xml: &mut XmlWriter, page: &PagePlan, index: usize) {
    start_shared(xml, "GTexture2D", page.texture);
    xml.start("GTexture", &[attr("xs.n", "super")]);
    xml.text("s", &[attr("xs.n", "name")], &format!("Texture {index}"));
    xml.empty(
        "WrapMode",
        &[attr("xs.n", "wrapMode"), attr("v", "CLAMP_TO_BORDER")],
    );
    xml.start("FilterMode", &[attr("xs.n", "filterMode")]);
    reference(xml, "GTexture2D", "owner", page.texture);
    xml.empty(
        "MinFilter",
        &[attr("xs.n", "minFilter"), attr("v", "LINEAR_MIPMAP_LINEAR")],
    );
    xml.empty(
        "MagFilter",
        &[attr("xs.n", "magFilter"), attr("v", "LINEAR")],
    );
    xml.end("FilterMode");
    reference(xml, "GTextureGuid", "guid", page.texture_guid);
    xml.empty("Anisotropy", &[attr("xs.n", "anisotropy"), attr("v", "ON")]);
    xml.end("GTexture");
    reference(
        xml,
        "CImageResource",
        "srcImageResource",
        page.image_resource,
    );
    affine(xml, "transformImageResource01toLogical01");
    xml.text("i", &[attr("xs.n", "mipmapLevel")], "64");
    xml.text("b", &[attr("xs.n", "isPremultiplied")], "true");
    xml.end("GTexture2D");
}

pub(super) fn affine(xml: &mut XmlWriter, name: &str) {
    xml.empty(
        "CAffine",
        &[
            attr("xs.n", name),
            attr("m00", "1.0"),
            attr("m01", "0.0"),
            attr("m02", "0.0"),
            attr("m10", "0.0"),
            attr("m11", "1.0"),
            attr("m12", "0.0"),
        ],
    );
}
