use std::collections::BTreeSet;

use crate::{
    Error, Result,
    moc3::{ArtMesh, Moc3Model},
};

use super::{
    super::{
        grid::{float, float_array, reference, reference_without_name, start_shared, write_grid},
        plan::{MeshPlan, PagePlan, ProjectPlan},
        texture::affine,
        writer::{XmlWriter, attr},
    },
    common::{color, start_parameter_controllable, write_form_header},
};

pub(super) fn write_meshes(
    xml: &mut XmlWriter,
    plan: &ProjectPlan,
    model: &Moc3Model,
) -> Result<()> {
    for (mesh, mesh_plan) in model.art_meshes().iter().zip(&plan.meshes) {
        let page = plan.pages.get(mesh.texture_index()).ok_or_else(|| {
            Error::InvalidCmo3(format!(
                "ArtMesh {:?} references missing texture {}",
                mesh.id(),
                mesh.texture_index()
            ))
        })?;
        write_texture_input(xml, mesh_plan, page);
        write_grid(
            xml,
            &mesh_plan.grid,
            model,
            mesh.binding_band_index(),
            &plan.parameters,
            mesh.id(),
        )?;
        write_mesh_source(xml, plan, model, mesh_plan, mesh, page);
    }
    Ok(())
}

fn write_texture_input(xml: &mut XmlWriter, plan: &MeshPlan, page: &PagePlan) {
    start_shared(xml, "CTextureInput_ModelImage", plan.texture_input);
    xml.start("ACTextureInput", &[attr("xs.n", "super")]);
    affine(xml, "optionalTransformOnCanvas");
    reference(
        xml,
        "CTextureInputExtension",
        "_owner",
        plan.texture_extension,
    );
    xml.end("ACTextureInput");
    reference(
        xml,
        "CModelImageGuid",
        "_modelImageGuid",
        page.model_image_guid,
    );
    xml.end("CTextureInput_ModelImage");

    start_shared(
        xml,
        "CTextureInput_TextureAtlasRegion",
        plan.texture_atlas_input,
    );
    xml.start("ACTextureInput", &[attr("xs.n", "super")]);
    affine(xml, "optionalTransformOnCanvas");
    reference(
        xml,
        "CTextureInputExtension",
        "_owner",
        plan.texture_extension,
    );
    xml.end("ACTextureInput");
    reference(
        xml,
        "CTextureAtlasGuid",
        "textureAtlasGuid",
        page.texture_atlas_guid,
    );
    affine(xml, "inputImageLocalToCanvasTransform");
    xml.end("CTextureInput_TextureAtlasRegion");

    start_shared(xml, "CTextureInputExtension", plan.texture_extension);
    xml.start("ACExtension", &[attr("xs.n", "super")]);
    reference(xml, "CExtensionGuid", "guid", plan.texture_extension_guid);
    reference(xml, "CArtMeshSource", "_owner", plan.source);
    xml.end("ACExtension");
    xml.start(
        "carray_list",
        &[attr("xs.n", "_textureInputs"), attr("count", 2)],
    );
    reference_without_name(xml, "CTextureInput_ModelImage", plan.texture_input);
    reference_without_name(
        xml,
        "CTextureInput_TextureAtlasRegion",
        plan.texture_atlas_input,
    );
    xml.end("carray_list");
    reference(
        xml,
        "CTextureInput_TextureAtlasRegion",
        "currentTextureInputData",
        plan.texture_atlas_input,
    );
    xml.end("CTextureInputExtension");
}

fn write_mesh_source(
    xml: &mut XmlWriter,
    project: &ProjectPlan,
    model: &Moc3Model,
    plan: &MeshPlan,
    mesh: &ArtMesh,
    page: &PagePlan,
) {
    let parent_part = mesh
        .parent_part_index()
        .and_then(|index| project.parts.get(index))
        .map_or(project.root_part.guid, |part| part.guid);
    let target_deformer = mesh
        .parent_deformer_index()
        .and_then(|index| project.deformers.get(index))
        .map_or(project.root_deformer_guid, |deformer| deformer.guid);
    let editable_positions = editable_positions(model, mesh);
    let edges = triangle_edges(mesh.triangle_indices());

    start_shared(xml, "CArtMeshSource", plan.source);
    xml.start("ACDrawableSource", &[attr("xs.n", "super")]);
    start_parameter_controllable(xml, mesh.id(), Some(parent_part), plan.grid.grid);
    xml.start(
        "carray_list",
        &[attr("xs.n", "_extensions"), attr("count", 3)],
    );
    write_editable_extension(xml, project, plan, &editable_positions, &edges);
    reference_without_name(xml, "CTextureInputExtension", plan.texture_extension);
    write_generator_extension(xml, plan);
    xml.end("carray_list");
    xml.empty("null", &[attr("xs.n", "internalColor_direct_argb")]);
    xml.end("ACParameterControllableSource");
    xml.empty(
        "CDrawableId",
        &[attr("xs.n", "id"), attr("idstr", mesh.id())],
    );
    reference(xml, "CDrawableGuid", "guid", plan.guid);
    reference(xml, "CDeformerGuid", "targetDeformerGuid", target_deformer);
    xml.start(
        "carray_list",
        &[
            attr("xs.n", "clipGuidList"),
            attr("count", mesh.mask_drawable_indices().len()),
        ],
    );
    for mask in mesh.mask_drawable_indices() {
        if let Some(mask_plan) = project.meshes.get(*mask) {
            reference_without_name(xml, "CDrawableGuid", mask_plan.guid);
        }
    }
    xml.end("carray_list");
    xml.text(
        "b",
        &[attr("xs.n", "invertClippingMask")],
        if mesh.drawable_flags() & 8 != 0 {
            "true"
        } else {
            "false"
        },
    );
    xml.end("ACDrawableSource");

    let indices = mesh
        .triangle_indices()
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(" ");
    xml.text(
        "int-array",
        &[
            attr("xs.n", "indices"),
            attr("count", mesh.triangle_indices().len()),
        ],
        &indices,
    );
    write_keyforms(xml, project, model, plan, mesh);
    xml.text(
        "float-array",
        &[
            attr("xs.n", "positions"),
            attr("count", editable_positions.len()),
        ],
        &float_array(editable_positions.iter().copied()),
    );
    let uvs = mesh.uvs().iter().flat_map(|uv| uv.iter().copied());
    xml.text(
        "float-array",
        &[attr("xs.n", "uvs"), attr("count", mesh.uvs().len() * 2)],
        &float_array(uvs),
    );
    reference(xml, "GTexture2D", "texture", page.texture);
    xml.empty(
        "ColorComposition",
        &[
            attr("xs.n", "colorComposition"),
            attr("v", color_composition(mesh.drawable_flags())),
        ],
    );
    xml.text(
        "b",
        &[attr("xs.n", "culling")],
        if mesh.drawable_flags() & 4 == 0 {
            "true"
        } else {
            "false"
        },
    );
    xml.empty(
        "TextureState",
        &[attr("xs.n", "textureState"), attr("v", "TEXTURE_ATLAS")],
    );
    xml.empty("s", &[attr("xs.n", "userData")]);
    xml.end("CArtMeshSource");
}

fn write_editable_extension(
    xml: &mut XmlWriter,
    project: &ProjectPlan,
    plan: &MeshPlan,
    positions: &[f32],
    edges: &[u16],
) {
    let point_count = positions.len() / 2;
    xml.start("CEditableMeshExtension", &[]);
    xml.start("ACExtension", &[attr("xs.n", "super")]);
    reference(xml, "CExtensionGuid", "guid", plan.editable_extension_guid);
    reference(xml, "CArtMeshSource", "_owner", plan.source);
    xml.end("ACExtension");
    xml.start(
        "GEditableMesh2",
        &[
            attr("xs.n", "editableMesh"),
            attr("nextPointUid", point_count),
            attr("useDelaunayTriangulation", "true"),
        ],
    );
    xml.text(
        "float-array",
        &[attr("xs.n", "point"), attr("count", positions.len())],
        &float_array(positions.iter().copied()),
    );
    xml.text(
        "byte-array",
        &[attr("xs.n", "pointPriority"), attr("count", point_count)],
        &repeated_values("20", point_count),
    );
    xml.text(
        "short-array",
        &[attr("xs.n", "edge"), attr("count", edges.len())],
        &edges
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" "),
    );
    xml.text(
        "byte-array",
        &[attr("xs.n", "edgePriority"), attr("count", edges.len() / 2)],
        &repeated_values("30", edges.len() / 2),
    );
    xml.text(
        "int-array",
        &[attr("xs.n", "pointUid"), attr("count", point_count)],
        &(0..point_count)
            .map(|index| index.to_string())
            .collect::<Vec<_>>()
            .join(" "),
    );
    reference(
        xml,
        "GEditableMeshGuid",
        "meshGuid",
        plan.editable_mesh_guid,
    );
    reference(xml, "CoordType", "coordType", project.coord_type);
    xml.end("GEditableMesh2");
    xml.text("b", &[attr("xs.n", "isLocked")], "false");
    xml.end("CEditableMeshExtension");
}

fn write_generator_extension(xml: &mut XmlWriter, plan: &MeshPlan) {
    xml.start("CMeshGeneratorExtension", &[]);
    xml.start("ACExtension", &[attr("xs.n", "super")]);
    reference(xml, "CExtensionGuid", "guid", plan.generator_extension_guid);
    reference(xml, "CArtMeshSource", "_owner", plan.source);
    xml.end("ACExtension");
    xml.start(
        "MeshGenerateSetting",
        &[attr("xs.n", "meshGenerateSetting")],
    );
    for (name, value) in [
        ("polygonOuterDensity", 100),
        ("polygonInnerDensity", 100),
        ("polygonMargin", 20),
        ("polygonInnerMargin", 20),
        ("polygonMinMargin", 5),
        ("polygonMinBoundsPt", 5),
        ("thresholdAlpha", 0),
    ] {
        xml.text("i", &[attr("xs.n", name)], &value.to_string());
    }
    xml.end("MeshGenerateSetting");
    xml.end("CMeshGeneratorExtension");
}

fn write_keyforms(
    xml: &mut XmlWriter,
    project: &ProjectPlan,
    model: &Moc3Model,
    plan: &MeshPlan,
    mesh: &ArtMesh,
) {
    let is_canvas = mesh.parent_deformer_index().is_none();
    xml.start(
        "carray_list",
        &[
            attr("xs.n", "keyforms"),
            attr("count", plan.grid.forms.len()),
        ],
    );
    for (index, form_guid) in plan.grid.forms.iter().copied().enumerate() {
        let keyform = mesh.keyforms().get(index);
        xml.start("CArtMeshForm", &[]);
        xml.start("ACDrawableForm", &[attr("xs.n", "super")]);
        write_form_header(xml, form_guid, "CArtMeshSource", plan.source);
        xml.text(
            "i",
            &[attr("xs.n", "drawOrder")],
            &keyform
                .map_or(500.0, |value| value.draw_order())
                .round()
                .to_string(),
        );
        xml.text(
            "f",
            &[attr("xs.n", "opacity")],
            &float(keyform.map_or(1.0, |value| value.opacity())),
        );
        color(
            xml,
            "multiplyColor",
            keyform.map_or([1.0; 3], |value| value.multiply_color()),
        );
        color(
            xml,
            "screenColor",
            keyform.map_or([0.0; 3], |value| value.screen_color()),
        );
        reference(
            xml,
            "CoordType",
            "coordType",
            super::common::form_coord_type(project, is_canvas),
        );
        xml.end("ACDrawableForm");
        let positions = keyform
            .map(|value| value.positions())
            .unwrap_or(&[])
            .iter()
            .flat_map(|position| super::common::form_position(model, *position, is_canvas));
        xml.text(
            "float-array",
            &[
                attr("xs.n", "positions"),
                attr(
                    "count",
                    keyform.map_or(0, |value| value.positions().len() * 2),
                ),
            ],
            &float_array(positions),
        );
        xml.end("CArtMeshForm");
    }
    xml.end("carray_list");
}

fn editable_positions(model: &Moc3Model, mesh: &ArtMesh) -> Vec<f32> {
    let is_canvas = mesh.parent_deformer_index().is_none();
    mesh.keyforms()
        .first()
        .map(|keyform| {
            keyform
                .positions()
                .iter()
                .flat_map(|position| super::common::form_position(model, *position, is_canvas))
                .collect()
        })
        .unwrap_or_else(|| vec![0.0; mesh.uvs().len() * 2])
}

fn triangle_edges(indices: &[u16]) -> Vec<u16> {
    let mut edges = BTreeSet::new();
    for triangle in indices.chunks_exact(3) {
        for (left, right) in [
            (triangle[0], triangle[1]),
            (triangle[1], triangle[2]),
            (triangle[2], triangle[0]),
        ] {
            edges.insert(if left <= right {
                (left, right)
            } else {
                (right, left)
            });
        }
    }
    edges
        .into_iter()
        .flat_map(|(left, right)| [left, right])
        .collect()
}

fn repeated_values(value: &str, count: usize) -> String {
    std::iter::repeat_n(value, count)
        .collect::<Vec<_>>()
        .join(" ")
}

fn color_composition(flags: u8) -> &'static str {
    if flags & 1 != 0 {
        "ADD"
    } else if flags & 2 != 0 {
        "MULTIPLY"
    } else {
        "NORMAL"
    }
}
