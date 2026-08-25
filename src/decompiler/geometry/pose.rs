use crate::{
    Error, Result,
    moc3::{
        ArtMeshKeyform, BindingBand, Deformer, Moc3Model, RotationDeformer,
        RotationDeformerKeyform, WarpDeformer, WarpDeformerKeyform,
    },
};

use super::warp;

const EPS_KEY: f32 = 0.001;
const EPS_SPAN: f32 = 0.0015;
const MAX_CORNERS: usize = 16;

pub(super) struct DefaultPose {
    mesh_positions: Vec<Vec<[f32; 2]>>,
}

impl DefaultPose {
    pub(super) fn evaluate(model: &Moc3Model) -> Result<Self> {
        let rotation_ancestors = super::rotation_ancestor_flags(model)?;
        let mut worlds = std::iter::repeat_with(|| None)
            .take(model.deformers().len())
            .collect::<Vec<_>>();
        let mut visiting = vec![false; model.deformers().len()];
        for index in 0..model.deformers().len() {
            build_world(
                model,
                index,
                &rotation_ancestors,
                &mut worlds,
                &mut visiting,
            )?;
        }

        let mesh_positions = model
            .art_meshes()
            .iter()
            .map(|mesh| {
                let corners = grid_corners(model, mesh.binding_band_index(), mesh.id())?;
                let mut positions = if mesh.keyforms().is_empty() {
                    vec![[0.0; 2]; mesh.uvs().len()]
                } else {
                    blend_points(
                        &corners,
                        mesh.keyforms(),
                        ArtMeshKeyform::positions,
                        mesh.uvs().len(),
                        mesh.id(),
                    )?
                };
                match mesh.parent_deformer_index() {
                    Some(index) => {
                        let world =
                            worlds.get(index).and_then(Option::as_ref).ok_or_else(|| {
                                Error::InvalidCmo3(format!(
                                    "ArtMesh {:?} references missing deformer {index}",
                                    mesh.id()
                                ))
                            })?;
                        for position in &mut positions {
                            *position = world.apply(*position);
                        }
                    }
                    None => {
                        for position in &mut positions {
                            *position = canvas_position(model, *position);
                        }
                    }
                }
                Ok(positions)
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { mesh_positions })
    }

    pub(super) fn mesh_positions(&self, index: usize) -> Result<&[[f32; 2]]> {
        self.mesh_positions
            .get(index)
            .map(Vec::as_slice)
            .ok_or_else(|| Error::InvalidCmo3(format!("default ArtMesh pose {index} is missing")))
    }
}

#[derive(Debug)]
enum DeformerWorld {
    Warp {
        control_points: Vec<[f32; 2]>,
        columns: usize,
        rows: usize,
        accumulated_scale: f32,
    },
    Rotation {
        transform: RotationTransform,
        accumulated_scale: f32,
    },
}

impl DeformerWorld {
    fn apply(&self, point: [f32; 2]) -> [f32; 2] {
        match self {
            Self::Warp {
                control_points,
                columns,
                rows,
                ..
            } => warp::apply(control_points, *columns, *rows, point),
            Self::Rotation { transform, .. } => transform.apply(point),
        }
    }

    fn accumulated_scale(&self) -> f32 {
        match self {
            Self::Warp {
                accumulated_scale, ..
            }
            | Self::Rotation {
                accumulated_scale, ..
            } => *accumulated_scale,
        }
    }

    fn is_rotation(&self) -> bool {
        matches!(self, Self::Rotation { .. })
    }
}

#[derive(Debug, Copy, Clone)]
struct RotationTransform {
    xx: f32,
    xy: f32,
    yx: f32,
    yy: f32,
    origin: [f32; 2],
}

impl RotationTransform {
    fn new(angle_degrees: f32, scale: f32, reflected: [bool; 2], origin: [f32; 2]) -> Self {
        let radians = angle_degrees * std::f32::consts::PI / 180.0;
        let sin = f64::from(radians).sin() as f32;
        let cos = f64::from(radians).cos() as f32;
        let reflect_x = if reflected[0] { -1.0 } else { 1.0 };
        let reflect_y = if reflected[1] { -1.0 } else { 1.0 };
        Self {
            xx: cos * scale * reflect_x,
            xy: -sin * scale * reflect_y,
            yx: sin * scale * reflect_x,
            yy: cos * scale * reflect_y,
            origin,
        }
    }

    fn apply(self, point: [f32; 2]) -> [f32; 2] {
        [
            self.xx * point[0] + self.xy * point[1] + self.origin[0],
            self.yx * point[0] + self.yy * point[1] + self.origin[1],
        ]
    }
}

#[derive(Debug, Copy, Clone)]
struct WeightedCell {
    index: usize,
    weight: f32,
}

fn build_world(
    model: &Moc3Model,
    index: usize,
    rotation_ancestors: &[bool],
    worlds: &mut [Option<DeformerWorld>],
    visiting: &mut [bool],
) -> Result<()> {
    if worlds.get(index).is_some_and(Option::is_some) {
        return Ok(());
    }
    let deformer = model.deformers().get(index).ok_or_else(|| {
        Error::InvalidCmo3(format!("deformer {index} is outside the deformer table"))
    })?;
    if visiting[index] {
        return Err(Error::InvalidCmo3(format!(
            "deformer {index} has a cyclic parent chain"
        )));
    }
    visiting[index] = true;
    if let Some(parent_index) = deformer.parent_deformer_index() {
        build_world(model, parent_index, rotation_ancestors, worlds, visiting)?;
    }
    let world = {
        let parent = deformer
            .parent_deformer_index()
            .and_then(|parent_index| worlds[parent_index].as_ref());
        match deformer {
            Deformer::Warp(warp) => build_warp_world(model, warp, parent)?,
            Deformer::Rotation(rotation) => {
                build_rotation_world(model, rotation, parent, rotation_ancestors[index])?
            }
        }
    };
    visiting[index] = false;
    worlds[index] = Some(world);
    Ok(())
}

fn build_warp_world(
    model: &Moc3Model,
    deformer: &WarpDeformer,
    parent: Option<&DeformerWorld>,
) -> Result<DeformerWorld> {
    if deformer.columns() == 0 || deformer.rows() == 0 {
        return Err(Error::InvalidCmo3(
            "warp deformer has a zero-sized lattice".into(),
        ));
    }
    let point_count = (deformer.columns() + 1)
        .checked_mul(deformer.rows() + 1)
        .ok_or_else(|| Error::InvalidCmo3("warp lattice size overflows".into()))?;
    let corners = grid_corners(model, deformer.binding_band_index(), "warp deformer")?;
    let mut control_points = blend_points(
        &corners,
        deformer.keyforms(),
        WarpDeformerKeyform::positions,
        point_count,
        "warp deformer",
    )?;
    if let Some(parent) = parent {
        for point in &mut control_points {
            *point = parent.apply(*point);
        }
    } else {
        for point in &mut control_points {
            *point = canvas_position(model, *point);
        }
    }
    Ok(DeformerWorld::Warp {
        control_points,
        columns: deformer.columns(),
        rows: deformer.rows(),
        accumulated_scale: parent.map_or(1.0, DeformerWorld::accumulated_scale),
    })
}

fn build_rotation_world(
    model: &Moc3Model,
    deformer: &RotationDeformer,
    parent: Option<&DeformerWorld>,
    has_rotation_ancestor: bool,
) -> Result<DeformerWorld> {
    let corners = grid_corners(model, deformer.binding_band_index(), "rotation deformer")?;
    let (origin, keyed_angle, keyed_scale) = blend_rotation(&corners, deformer.keyforms())?;
    let floor = deformer
        .keyforms()
        .get(corners.first().map_or(0, |corner| corner.index))
        .ok_or_else(|| Error::InvalidCmo3("rotation floor keyform is missing".into()))?;
    let local_scale = keyed_scale
        * if has_rotation_ancestor {
            1.0
        } else {
            effective_pixels_per_unit(model)
        };
    let accumulated_scale = parent.map_or(1.0, DeformerWorld::accumulated_scale) * local_scale;
    let local_angle = deformer.base_angle_degrees() + keyed_angle;
    let (world_origin, world_angle) = match parent {
        Some(parent) => {
            let world_origin = parent.apply(origin);
            let displacement = if parent.is_rotation() { -10.0 } else { -0.1 };
            let mut delta = [0.0; 2];
            let mut probe_scale = 1.0;
            for _ in 0..10 {
                let probe = parent.apply([origin[0], origin[1] + displacement * probe_scale]);
                delta = [probe[0] - world_origin[0], probe[1] - world_origin[1]];
                if delta != [0.0; 2] {
                    break;
                }
                let probe = parent.apply([origin[0], origin[1] - displacement * probe_scale]);
                delta = [world_origin[0] - probe[0], world_origin[1] - probe[1]];
                if delta != [0.0; 2] {
                    break;
                }
                probe_scale *= 0.1;
            }
            let inherited = if delta == [0.0; 2] {
                0.0
            } else {
                normalize_radians(displacement.atan2(0.0) - delta[1].atan2(delta[0]))
            };
            (
                world_origin,
                local_angle - inherited * 180.0 / std::f32::consts::PI,
            )
        }
        None => (canvas_position(model, origin), local_angle),
    };
    Ok(DeformerWorld::Rotation {
        transform: RotationTransform::new(
            world_angle,
            accumulated_scale,
            floor.reflected(),
            world_origin,
        ),
        accumulated_scale,
    })
}

fn blend_rotation(
    corners: &[WeightedCell],
    keyforms: &[RotationDeformerKeyform],
) -> Result<([f32; 2], f32, f32)> {
    let mut origin = [0.0; 2];
    let mut angle = 0.0;
    let mut scale = 0.0;
    for corner in corners {
        let keyform = keyforms.get(corner.index).ok_or_else(|| {
            Error::InvalidCmo3(format!("rotation keyform {} is missing", corner.index))
        })?;
        let keyed_origin = keyform.origin();
        origin[0] += keyed_origin[0] * corner.weight;
        origin[1] += keyed_origin[1] * corner.weight;
        angle += keyform.angle_degrees() * corner.weight;
        scale += keyform.scale() * corner.weight;
    }
    Ok((origin, angle, scale))
}

fn blend_points<T>(
    corners: &[WeightedCell],
    keyforms: &[T],
    positions: impl Fn(&T) -> &[[f32; 2]],
    point_count: usize,
    owner: &str,
) -> Result<Vec<[f32; 2]>> {
    let mut blended = vec![[0.0; 2]; point_count];
    for corner in corners {
        let keyform = keyforms.get(corner.index).ok_or_else(|| {
            Error::InvalidCmo3(format!("{owner} keyform {} is missing", corner.index))
        })?;
        let keyed_positions = positions(keyform);
        if keyed_positions.len() != point_count {
            return Err(Error::InvalidCmo3(format!(
                "{owner} keyform {} has {} points, expected {point_count}",
                corner.index,
                keyed_positions.len()
            )));
        }
        for (target, source) in blended.iter_mut().zip(keyed_positions) {
            target[0] += source[0] * corner.weight;
            target[1] += source[1] * corner.weight;
        }
    }
    Ok(blended)
}

fn grid_corners(
    model: &Moc3Model,
    band_index: Option<usize>,
    owner: &str,
) -> Result<Vec<WeightedCell>> {
    let Some(index) = band_index else {
        return Ok(vec![WeightedCell {
            index: 0,
            weight: 1.0,
        }]);
    };
    let band = model
        .binding_bands()
        .get(index)
        .ok_or_else(|| Error::InvalidCmo3(format!("{owner} binding band {index} is missing")))?;
    corners_for_band(model, band, owner)
}

fn corners_for_band(
    model: &Moc3Model,
    band: &BindingBand,
    owner: &str,
) -> Result<Vec<WeightedCell>> {
    let mut corners = vec![WeightedCell {
        index: 0,
        weight: 1.0,
    }];
    let mut stride = 1usize;
    for binding in band.bindings() {
        let parameter = model
            .parameters()
            .get(binding.parameter_index())
            .ok_or_else(|| {
                Error::InvalidCmo3(format!(
                    "{owner} references missing parameter {}",
                    binding.parameter_index()
                ))
            })?;
        let (lower, fraction) = bracket(binding.keys(), parameter.default()).ok_or_else(|| {
            Error::InvalidCmo3(format!(
                "default value {} for parameter {:?} is outside {owner}'s key range",
                parameter.default(),
                parameter.id()
            ))
        })?;
        if fraction > 0.0 && corners.len() * 2 <= MAX_CORNERS {
            let mut expanded = Vec::with_capacity(corners.len() * 2);
            for corner in corners {
                expanded.push(WeightedCell {
                    index: corner.index + lower * stride,
                    weight: corner.weight * (1.0 - fraction),
                });
                expanded.push(WeightedCell {
                    index: corner.index + (lower + 1) * stride,
                    weight: corner.weight * fraction,
                });
            }
            corners = expanded;
        } else {
            for corner in &mut corners {
                corner.index += lower * stride;
            }
        }
        stride = stride
            .checked_mul(binding.keys().len().max(1))
            .ok_or_else(|| Error::InvalidCmo3(format!("{owner} keyform grid overflows")))?;
    }
    Ok(corners)
}

fn bracket(keys: &[f32], value: f32) -> Option<(usize, f32)> {
    match keys {
        [] => Some((0, 0.0)),
        [key] => (value > key - EPS_KEY && value < key + EPS_KEY).then_some((0, 0.0)),
        _ => {
            if value < keys[0] - EPS_KEY || value >= keys[keys.len() - 1] + EPS_KEY {
                return None;
            }
            if value < keys[0] + EPS_KEY {
                return Some((0, 0.0));
            }
            let mut upper = 1;
            while upper < keys.len() && keys[upper] + EPS_KEY <= value {
                upper += 1;
            }
            if value <= keys[upper] - EPS_KEY {
                let span = keys[upper] - keys[upper - 1];
                let fraction = if span >= EPS_SPAN {
                    (value - keys[upper - 1]) / span
                } else {
                    0.0
                };
                Some((upper - 1, fraction))
            } else {
                Some((upper, 0.0))
            }
        }
    }
}

fn canvas_position(model: &Moc3Model, position: [f32; 2]) -> [f32; 2] {
    let scale = effective_pixels_per_unit(model);
    let origin = model.canvas().origin();
    [
        position[0] * scale + origin[0],
        position[1] * scale + origin[1],
    ]
}

fn effective_pixels_per_unit(model: &Moc3Model) -> f32 {
    let scale = model.canvas().pixels_per_unit();
    if scale == 0.0 { 1.0 } else { scale }
}

fn normalize_radians(mut value: f32) -> f32 {
    while value > std::f32::consts::PI {
        value -= 2.0 * std::f32::consts::PI;
    }
    while value < -std::f32::consts::PI {
        value += 2.0 * std::f32::consts::PI;
    }
    value
}
