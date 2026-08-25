use crate::moc3::{Deformer, Moc3Model};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) struct RefId(u32);

impl RefId {
    pub(super) fn value(self) -> String {
        format!("#{}", self.0)
    }

    pub(super) fn index(self) -> u32 {
        self.0
    }
}

#[derive(Default)]
struct Allocator {
    next: u32,
}

impl Allocator {
    fn id(&mut self) -> RefId {
        let id = RefId(self.next);
        self.next += 1;
        id
    }
}

pub(super) struct ProjectPlan {
    pub(super) parameter_group_guid: RefId,
    pub(super) model_guid: RefId,
    pub(super) root_deformer_guid: RefId,
    pub(super) coord_type: RefId,
    pub(super) blend_normal: RefId,
    pub(super) layered_image_guid: RefId,
    pub(super) layered_image: RefId,
    pub(super) layer_group: RefId,
    pub(super) filters: FilterGlobals,
    pub(super) parameters: Vec<ParameterPlan>,
    pub(super) pages: Vec<PagePlan>,
    pub(super) root_part: PartPlan,
    pub(super) parts: Vec<PartPlan>,
    pub(super) deformers: Vec<DeformerPlan>,
    pub(super) meshes: Vec<MeshPlan>,
    pub(super) glues: Vec<GluePlan>,
    pub(super) image_group: RefId,
}

pub(super) struct FilterGlobals {
    pub(super) selector_definition: RefId,
    pub(super) filter_definition: RefId,
    pub(super) value_ids: [RefId; 8],
    pub(super) values: [RefId; 9],
}

pub(super) struct ParameterPlan {
    pub(super) guid: RefId,
}

pub(super) struct PagePlan {
    pub(super) model_image_guid: RefId,
    pub(super) texture_guid: RefId,
    pub(super) image_resource: RefId,
    pub(super) layer: RefId,
    pub(super) filter_set: RefId,
    pub(super) filter_instance_ids: [RefId; 2],
    pub(super) filter_instances: [RefId; 2],
    pub(super) filter_output: RefId,
    pub(super) texture: RefId,
}

pub(super) struct GridPlan {
    pub(super) grid: RefId,
    pub(super) bindings: Vec<RefId>,
    pub(super) forms: Vec<RefId>,
}

pub(super) struct PartPlan {
    pub(super) guid: RefId,
    pub(super) source: RefId,
    pub(super) grid: GridPlan,
}

pub(super) struct DeformerPlan {
    pub(super) guid: RefId,
    pub(super) source: RefId,
    pub(super) grid: GridPlan,
}

pub(super) struct MeshPlan {
    pub(super) guid: RefId,
    pub(super) source: RefId,
    pub(super) grid: GridPlan,
    pub(super) editable_mesh_guid: RefId,
    pub(super) editable_extension_guid: RefId,
    pub(super) generator_extension_guid: RefId,
    pub(super) texture_extension_guid: RefId,
    pub(super) texture_extension: RefId,
    pub(super) texture_input: RefId,
}

pub(super) struct GluePlan {
    pub(super) guid: RefId,
    pub(super) source: RefId,
    pub(super) grid: GridPlan,
}

impl ProjectPlan {
    pub(super) fn new(model: &Moc3Model, texture_count: usize) -> Self {
        let mut ids = Allocator::default();
        let parameter_group_guid = ids.id();
        let model_guid = ids.id();
        let root_deformer_guid = ids.id();
        let coord_type = ids.id();
        let blend_normal = ids.id();
        let layered_image_guid = ids.id();
        let layered_image = ids.id();
        let layer_group = ids.id();
        let filters = FilterGlobals {
            selector_definition: ids.id(),
            filter_definition: ids.id(),
            value_ids: std::array::from_fn(|_| ids.id()),
            values: std::array::from_fn(|_| ids.id()),
        };
        let parameters = model
            .parameters()
            .iter()
            .map(|_| ParameterPlan { guid: ids.id() })
            .collect();
        let pages = (0..texture_count)
            .map(|_| PagePlan {
                model_image_guid: ids.id(),
                texture_guid: ids.id(),
                image_resource: ids.id(),
                layer: ids.id(),
                filter_set: ids.id(),
                filter_instance_ids: [ids.id(), ids.id()],
                filter_instances: [ids.id(), ids.id()],
                filter_output: ids.id(),
                texture: ids.id(),
            })
            .collect();
        let root_part = part_plan(&mut ids, 0, 1);
        let parts = model
            .parts()
            .iter()
            .map(|part| {
                part_plan(
                    &mut ids,
                    binding_count(model, part.binding_band_index()),
                    part.keyforms().len(),
                )
            })
            .collect();
        let deformers = model
            .deformers()
            .iter()
            .map(|deformer| {
                let keyform_count = match deformer {
                    Deformer::Warp(value) => value.keyforms().len(),
                    Deformer::Rotation(value) => value.keyforms().len(),
                };
                DeformerPlan {
                    guid: ids.id(),
                    source: ids.id(),
                    grid: grid_plan(
                        &mut ids,
                        binding_count(model, deformer.binding_band_index()),
                        keyform_count,
                    ),
                }
            })
            .collect();
        let meshes = model
            .art_meshes()
            .iter()
            .map(|mesh| MeshPlan {
                guid: ids.id(),
                source: ids.id(),
                grid: grid_plan(
                    &mut ids,
                    binding_count(model, mesh.binding_band_index()),
                    mesh.keyforms().len(),
                ),
                editable_mesh_guid: ids.id(),
                editable_extension_guid: ids.id(),
                generator_extension_guid: ids.id(),
                texture_extension_guid: ids.id(),
                texture_extension: ids.id(),
                texture_input: ids.id(),
            })
            .collect();
        let glues = model
            .glues()
            .iter()
            .map(|glue| GluePlan {
                guid: ids.id(),
                source: ids.id(),
                grid: grid_plan(
                    &mut ids,
                    binding_count(model, glue.binding_band_index()),
                    glue.intensities().len(),
                ),
            })
            .collect();
        let image_group = ids.id();

        Self {
            parameter_group_guid,
            model_guid,
            root_deformer_guid,
            coord_type,
            blend_normal,
            layered_image_guid,
            layered_image,
            layer_group,
            filters,
            parameters,
            pages,
            root_part,
            parts,
            deformers,
            meshes,
            glues,
            image_group,
        }
    }
}

fn part_plan(ids: &mut Allocator, binding_count: usize, keyform_count: usize) -> PartPlan {
    PartPlan {
        guid: ids.id(),
        source: ids.id(),
        grid: grid_plan(ids, binding_count, keyform_count),
    }
}

fn grid_plan(ids: &mut Allocator, binding_count: usize, keyform_count: usize) -> GridPlan {
    GridPlan {
        grid: ids.id(),
        bindings: (0..binding_count).map(|_| ids.id()).collect(),
        forms: (0..keyform_count.max(1)).map(|_| ids.id()).collect(),
    }
}

fn binding_count(model: &Moc3Model, band_index: Option<usize>) -> usize {
    band_index
        .and_then(|index| model.binding_bands().get(index))
        .map_or(0, |band| band.bindings().len())
}
