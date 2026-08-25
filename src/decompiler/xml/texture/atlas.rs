use uuid::Uuid;

use crate::decompiler::Texture;

use super::super::{
    grid::{empty_shared, reference, start_shared},
    plan::PagePlan,
    writer::{XmlWriter, attr},
};
use super::cache;

pub(super) fn write(xml: &mut XmlWriter, page: &PagePlan, texture: &Texture, index: usize) {
    empty_shared(
        xml,
        "CTextureAtlasGuid",
        page.texture_atlas_guid,
        vec![
            attr("uuid", Uuid::new_v4()),
            attr("note", format!("Texture atlas {}", index + 1)),
        ],
    );
    start_shared(xml, "CTextureAtlas", page.texture_atlas);
    xml.text(
        "s",
        &[attr("xs.n", "name")],
        &format!("TextureAtlas{}", index + 1),
    );
    xml.text("i", &[attr("xs.n", "width")], &texture.width().to_string());
    xml.text(
        "i",
        &[attr("xs.n", "height")],
        &texture.height().to_string(),
    );
    reference(
        xml,
        "CImageResource",
        "cachedAtlasImage",
        page.image_resource,
    );
    xml.text("b", &[attr("xs.n", "lockCachedAtlasImage")], "false");
    reference(xml, "CTextureAtlasGuid", "guid", page.texture_atlas_guid);
    xml.empty(
        "carray_list",
        &[attr("xs.n", "modelImages"), attr("count", 0)],
    );
    cache::write_manager(xml, "cachedImageManager", page.image_resource, texture);
    xml.end("CTextureAtlas");
}
