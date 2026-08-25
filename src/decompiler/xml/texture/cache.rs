use crate::decompiler::Texture;

use super::super::{
    grid::reference,
    plan::RefId,
    writer::{XmlWriter, attr},
};
use super::affine;

pub(super) fn write_manager(
    xml: &mut XmlWriter,
    name: &str,
    image_resource: RefId,
    texture: &Texture,
) {
    xml.start("CCachedImageManager", &[attr("xs.n", name)]);
    xml.empty(
        "CachedImageType",
        &[attr("xs.n", "defaultCacheType"), attr("v", "SCALE_1")],
    );
    reference(xml, "CImageResource", "rawImage", image_resource);
    xml.start(
        "array_list",
        &[attr("xs.n", "cachedImages"), attr("count", 1)],
    );
    xml.start("CCachedImage", &[]);
    reference(
        xml,
        "CImageResource",
        "_cachedImageResource",
        image_resource,
    );
    xml.text("b", &[attr("xs.n", "isSharedImage")], "true");
    xml.empty(
        "CSize",
        &[
            attr("xs.n", "rawImageSize"),
            attr("width", texture.width()),
            attr("height", texture.height()),
        ],
    );
    xml.text("i", &[attr("xs.n", "reductionRatio")], "1");
    xml.text("i", &[attr("xs.n", "mipmapLevel")], "64");
    xml.text("b", &[attr("xs.n", "hasMargin")], "false");
    xml.text("b", &[attr("xs.n", "isCleaned")], "false");
    affine(xml, "transformRawImageToCachedImage");
    xml.end("CCachedImage");
    xml.end("array_list");
    xml.text("i", &[attr("xs.n", "requiredMipmapLevel")], "64");
    xml.end("CCachedImageManager");
}
