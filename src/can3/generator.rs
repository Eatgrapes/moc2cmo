use uuid::Uuid;

use super::xml::{IMPORTS, XmlWriter, attr};
use crate::{
    Result,
    model3::Model3Group,
    motion3::{Motion3, MotionCurve, MotionPoint, MotionSegment},
};

/// Builds a CAN3 document from a model link and parsed motion files.
pub(crate) fn generate(
    animation_name: &str,
    model_path: &str,
    motions: &[MotionInstance],
    groups: &[Model3Group],
) -> Result<Vec<u8>> {
    let mut xml = String::new();
    write_header(&mut xml);
    xml.push_str("<root fileFormatVersion=\"401000005\"><shared>");
    let mut ids = 10u32;
    let animation = ids;
    ids += 1;
    let resource_manager = ids;
    ids += 1;
    let resource_group = ids;
    ids += 1;
    let resource = ids;
    ids += 1;
    let resource_data = ids;
    ids += 1;
    let mut scene_refs = Vec::new();
    for input in motions {
        let name = &input.name;
        let motion = &input.motion;
        let scene = ids;
        ids += 1;
        let group = ids;
        ids += 1;
        let root_track_guid = ids;
        ids += 1;
        let track_guid = ids;
        ids += 1;
        let track = ids;
        ids += 1;
        let parameter_effect = ids;
        ids += 1;
        let parts_effect = ids;
        ids += 1;
        let visual_effect = ids;
        ids += 1;
        let eye_effect = ids;
        ids += 1;
        let lip_effect = ids;
        ids += 1;
        let scene_guid = ids;
        ids += 1;
        scene_refs.push((scene, scene_guid));
        write_group_track(&mut xml, group, scene, root_track_guid, track_guid);
        write_scene(&mut xml, scene, scene_guid, group, track, motion, name);
        write_model_track(
            &mut xml,
            TrackIds {
                track,
                guid: track_guid,
                scene,
                parameter: parameter_effect,
                parts: parts_effect,
                visual: visual_effect,
                eye: eye_effect,
                lip: lip_effect,
            },
            motion,
            resource,
            root_track_guid,
        )?;
        write_parameter_effect(
            &mut xml,
            parameter_effect,
            track,
            motion,
            FadeTimes {
                in_time: input.fade_in_time,
                out_time: input.fade_out_time,
            },
        )?;
        write_parts_effect(
            &mut xml,
            parts_effect,
            track,
            motion,
            FadeTimes {
                in_time: input.fade_in_time,
                out_time: input.fade_out_time,
            },
        );
        write_visual_effect(
            &mut xml,
            visual_effect,
            track,
            motion,
            FadeTimes {
                in_time: input.fade_in_time,
                out_time: input.fade_out_time,
            },
        );
        write_special_effects(
            &mut xml,
            TrackIds {
                track,
                guid: track_guid,
                scene,
                parameter: parameter_effect,
                parts: parts_effect,
                visual: visual_effect,
                eye: eye_effect,
                lip: lip_effect,
            },
            groups,
            motion,
            FadeTimes {
                in_time: input.fade_in_time,
                out_time: input.fade_out_time,
            },
        );
    }
    write_animation(
        &mut xml,
        animation,
        animation_name,
        &scene_refs,
        resource_manager,
    );
    write_resource_manager(
        &mut xml,
        resource_manager,
        resource_group,
        resource,
        resource_data,
        model_path,
    );
    xml.push_str("</shared><main>");
    let mut writer = XmlWriter::new(&mut xml);
    writer.empty("CAnimation", &[attr("xs.ref", ref_id(animation))]);
    xml.push_str("</main></root>");
    Ok(xml.into_bytes())
}

/// One parsed motion and its optional manifest fade settings.
#[derive(Debug, Clone)]
pub struct MotionInstance {
    /// Name used for the generated scene.
    pub name: String,
    /// Parsed motion data.
    pub motion: Motion3,
    /// Optional fade-in duration in seconds.
    pub fade_in_time: Option<f32>,
    /// Optional fade-out duration in seconds.
    pub fade_out_time: Option<f32>,
}

#[derive(Copy, Clone)]
struct TrackIds {
    track: u32,
    guid: u32,
    scene: u32,
    parameter: u32,
    parts: u32,
    visual: u32,
    eye: u32,
    lip: u32,
}

#[derive(Copy, Clone)]
struct FadeTimes {
    in_time: Option<f32>,
    out_time: Option<f32>,
}

fn write_header(xml: &mut String) {
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<?version CSceneSource:3?>\n<?version CAnimation:4?>\n<?version CMvParameter_Group:1?>\n<?version SerializeFormatVersion:2?>\n<?version CMvMovieInfo:3?>\n<?version CBezierCtrlPt:2?>\n");
    for import in IMPORTS {
        xml.push_str("<?import ");
        xml.push_str(import);
        xml.push_str("?>\n");
    }
}

fn ref_id(id: u32) -> String {
    format!("#{id}")
}

fn reference(xml: &mut XmlWriter<'_>, tag: &str, name: &str, id: u32) {
    xml.empty(tag, &[attr("xs.n", name), attr("xs.ref", ref_id(id))]);
}

fn write_bounds(xml: &mut XmlWriter<'_>, name: &str, width: f32, height: f32) {
    xml.start("GRectF", &[attr("xs.n", name)]);
    xml.text("f", &[attr("xs.n", "x")], 0);
    xml.text("f", &[attr("xs.n", "y")], 0);
    xml.text("f", &[attr("xs.n", "width")], width);
    xml.text("f", &[attr("xs.n", "height")], height);
    xml.end("GRectF");
}

fn write_group_track(xml: &mut String, group: u32, scene: u32, guid: u32, track: u32) {
    let mut xml = XmlWriter::new(xml);
    xml.start("CMvTrack_Group_Source", &[attr("xs.id", ref_id(group))]);
    xml.start("ICMvTrack_Source", &[attr("xs.n", "super")]);
    xml.text("s", &[attr("xs.n", "name")], "Root");
    xml.text("b", &[attr("xs.n", "isUserRenamed")], false);
    reference(&mut xml, "CTrackGuid", "guid", guid);
    for name in ["start", "internalOffset", "duration"] {
        xml.text("i", &[attr("xs.n", name)], 0);
    }
    for (name, value) in [
        ("editable", true),
        ("visible", true),
        ("mute", false),
        ("isGuide", false),
        ("isRepeat", false),
        ("soloSwitch", false),
    ] {
        xml.text("b", &[attr("xs.n", name)], value);
    }
    xml.start("CVisualHandler", &[attr("xs.n", "visualHandler")]);
    xml.empty(
        "CMvTrack_Group_Source",
        &[attr("xs.n", "track"), attr("xs.ref", ref_id(group))],
    );
    xml.end("CVisualHandler");
    xml.start("CSoundHandler", &[attr("xs.n", "soundHandler")]);
    xml.empty(
        "CMvTrack_Group_Source",
        &[attr("xs.n", "track"), attr("xs.ref", ref_id(group))],
    );
    xml.end("CSoundHandler");
    xml.empty("null", &[attr("xs.n", "soundEffect")]);
    xml.empty("null", &[attr("xs.n", "visualEffect")]);
    xml.start("CMvEffectManager", &[attr("xs.n", "effectManager")]);
    xml.empty(
        "array",
        &[
            attr("xs.n", "effectList"),
            attr("count", 0),
            attr("type", "ICMvEffect"),
        ],
    );
    xml.end("CMvEffectManager");
    xml.empty("null", &[attr("xs.n", "parentGuid")]);
    reference(&mut xml, "CSceneSource", "_sceneSource", scene);
    xml.empty(
        "hash_map",
        &[
            attr("xs.n", "userData"),
            attr("count", 0),
            attr("keyType", "string"),
        ],
    );
    xml.end("ICMvTrack_Source");
    xml.start(
        "carray_list",
        &[attr("xs.n", "_childTrackGuids"), attr("count", 1)],
    );
    xml.empty("CTrackGuid", &[attr("xs.ref", ref_id(track))]);
    xml.end("carray_list");
    write_bounds(&mut xml, "bounds", 640.0, 480.0);
    xml.end("CMvTrack_Group_Source");
    xml.empty(
        "CTrackGuid",
        &[attr("uuid", Uuid::new_v4()), attr("xs.id", ref_id(guid))],
    );
}

fn write_scene(
    xml: &mut String,
    scene: u32,
    scene_guid: u32,
    group: u32,
    track: u32,
    motion: &Motion3,
    name: &str,
) {
    let frames = (motion.meta().duration() * motion.meta().fps())
        .ceil()
        .max(1.0) as u32;
    let mut xml = XmlWriter::new(xml);
    xml.start("CSceneSource", &[attr("xs.id", ref_id(scene))]);
    xml.text("s", &[attr("xs.n", "sceneName")], name);
    xml.start("CImageCanvas", &[attr("xs.n", "canvas")]);
    xml.text("i", &[attr("xs.n", "pixelWidth")], 320);
    xml.text("i", &[attr("xs.n", "pixelHeight")], 240);
    xml.empty("CColor", &[attr("xs.n", "background")]);
    xml.end("CImageCanvas");
    reference(&mut xml, "CSceneGuid", "guid", scene_guid);
    xml.empty("s", &[attr("xs.n", "tag")]);
    xml.start("CTrackSourceSet", &[attr("xs.n", "trackSourceSet")]);
    xml.start("carray_list", &[attr("xs.n", "_sources"), attr("count", 2)]);
    xml.empty("CMvTrack_Group_Source", &[attr("xs.ref", ref_id(group))]);
    xml.empty(
        "CMvTrack_Live2DModel_Source",
        &[attr("xs.ref", ref_id(track))],
    );
    xml.end("carray_list");
    xml.end("CTrackSourceSet");
    reference(&mut xml, "CMvTrack_Group_Source", "rootTrack", group);
    xml.start("CMvMovieInfo", &[attr("xs.n", "movieInfo")]);
    for (name, value) in [
        ("width", 320),
        ("height", 240),
        ("duration", frames + 1),
        ("workspaceStart", 0),
        ("workspaceEnd", frames),
        ("fadeInMSec", u32::MAX),
        ("fadeOutMSec", u32::MAX),
        ("startFrame", 0),
    ] {
        xml.text(
            "i",
            &[attr("xs.n", name)],
            if value == u32::MAX {
                "-1".into()
            } else {
                value.to_string()
            },
        );
    }
    xml.text("d", &[attr("xs.n", "fps")], motion.meta().fps());
    xml.empty("CColor", &[attr("xs.n", "background")]);
    xml.text(
        "b",
        &[attr("xs.n", "isBezierRestricted")],
        motion.meta().are_beziers_restricted(),
    );
    xml.text(
        "b",
        &[attr("xs.n", "isLoopMotion")],
        motion.meta().is_looping(),
    );
    xml.empty(
        "CFrameIndexType",
        &[attr("xs.n", "frameIndexType"), attr("v", "ZERO_INDEX")],
    );
    xml.end("CMvMovieInfo");
    xml.start(
        "hash_map",
        &[
            attr("xs.n", "marker"),
            attr("count", motion.user_data().len()),
            attr("keyType", "string"),
        ],
    );
    for event in motion.user_data() {
        xml.start("entry", &[]);
        xml.text("s", &[attr("xs.n", "key")], event.time);
        xml.text("s", &[attr("xs.n", "value")], &event.value);
        xml.end("entry");
    }
    xml.end("hash_map");
    xml.empty(
        "CCurveType",
        &[
            attr("xs.n", "defaultParameterCurveType"),
            attr("v", "SMOOTH"),
        ],
    );
    xml.empty(
        "CCurveType",
        &[attr("xs.n", "defaultPartCurveType"), attr("v", "STEP")],
    );
    xml.text("b", &[attr("xs.n", "fixAspect")], true);
    xml.empty(
        "Animation",
        &[attr("xs.n", "targetVersion"), attr("v", "FOR_SDK")],
    );
    xml.end("CSceneSource");
    xml.empty(
        "CSceneGuid",
        &[
            attr("uuid", Uuid::new_v4()),
            attr("xs.id", ref_id(scene_guid)),
        ],
    );
}

fn write_model_track(
    xml: &mut String,
    ids: TrackIds,
    motion: &Motion3,
    resource: u32,
    parent_guid: u32,
) -> Result<()> {
    let duration = (motion.meta().duration() * motion.meta().fps())
        .ceil()
        .max(1.0) as u32;
    let mut writer = XmlWriter::new(xml);
    writer.start(
        "CMvTrack_Live2DModel_Source",
        &[attr("xs.id", ref_id(ids.track))],
    );
    writer.start("ICMvTrack_Linked", &[attr("xs.n", "super")]);
    writer.start("ICMvTrack_Source", &[attr("xs.n", "super")]);
    writer.text("s", &[attr("xs.n", "name")], "Model");
    writer.text("b", &[attr("xs.n", "isUserRenamed")], true);
    writer.empty(
        "CTrackGuid",
        &[attr("xs.n", "guid"), attr("xs.ref", ref_id(ids.guid))],
    );
    writer.text("i", &[attr("xs.n", "start")], 0);
    writer.text("i", &[attr("xs.n", "internalOffset")], 0);
    writer.text("i", &[attr("xs.n", "duration")], duration);
    for (name, value) in [
        ("editable", true),
        ("visible", true),
        ("mute", false),
        ("isGuide", false),
        ("isRepeat", false),
        ("soloSwitch", false),
    ] {
        writer.text("b", &[attr("xs.n", name)], value);
    }
    writer.start("CVisualHandler", &[attr("xs.n", "visualHandler")]);
    writer.empty(
        "CMvTrack_Live2DModel_Source",
        &[attr("xs.n", "track"), attr("xs.ref", ref_id(ids.track))],
    );
    writer.end("CVisualHandler");
    writer.start("CSoundHandler", &[attr("xs.n", "soundHandler")]);
    writer.empty(
        "CMvTrack_Live2DModel_Source",
        &[attr("xs.n", "track"), attr("xs.ref", ref_id(ids.track))],
    );
    writer.end("CSoundHandler");
    writer.empty("null", &[attr("xs.n", "soundEffect")]);
    writer.start("CMvEffectManager", &[attr("xs.n", "effectManager")]);
    writer.start(
        "array",
        &[
            attr("xs.n", "effectList"),
            attr("count", 5),
            attr("type", "ICMvEffect"),
        ],
    );
    for (tag, id) in [
        ("CMvEffect_EyeBlink", ids.eye),
        ("CMvEffect_LipSync", ids.lip),
        ("CMvEffect_Live2DParameter", ids.parameter),
        ("CMvEffect_Live2DPartsVisible", ids.parts),
        ("CMvEffect_VisualDefault", ids.visual),
    ] {
        writer.empty(tag, &[attr("xs.ref", ref_id(id))]);
    }
    writer.end("array");
    writer.end("CMvEffectManager");
    writer.empty(
        "CTrackGuid",
        &[
            attr("xs.n", "parentGuid"),
            attr("xs.ref", ref_id(parent_guid)),
        ],
    );
    writer.empty(
        "CSceneSource",
        &[
            attr("xs.n", "_sceneSource"),
            attr("xs.ref", ref_id(ids.scene)),
        ],
    );
    writer.empty(
        "hash_map",
        &[
            attr("xs.n", "userData"),
            attr("count", 0),
            attr("keyType", "string"),
        ],
    );
    writer.empty("null", &[attr("xs.n", "keys")]);
    writer.end("ICMvTrack_Source");
    writer.empty(
        "CResourceGuid",
        &[
            attr("xs.n", "_resourceGuid"),
            attr("xs.ref", ref_id(resource)),
        ],
    );
    writer.end("ICMvTrack_Linked");
    for (name, id) in [
        ("keyParamEffect", ids.parameter),
        ("partsVisibleEffect", ids.parts),
        ("visualEffect", ids.visual),
        ("eyeBlinkEffect", ids.eye),
        ("lipSyncEffect", ids.lip),
    ] {
        let tag = match name {
            "keyParamEffect" => "CMvEffect_Live2DParameter",
            "partsVisibleEffect" => "CMvEffect_Live2DPartsVisible",
            "visualEffect" => "CMvEffect_VisualDefault",
            "eyeBlinkEffect" => "CMvEffect_EyeBlink",
            _ => "CMvEffect_LipSync",
        };
        writer.empty(tag, &[attr("xs.n", name), attr("xs.ref", ref_id(id))]);
    }
    writer.empty("null", &[attr("xs.n", "formEditEffect")]);
    writer.start("FormAnimationSet", &[attr("xs.n", "formAnimationSet")]);
    writer.empty(
        "hash_map",
        &[
            attr("xs.n", "formMapOnGlobal"),
            attr("count", 0),
            attr("keyType", "string"),
        ],
    );
    writer.empty(
        "hash_map",
        &[
            attr("xs.n", "formMapOnLocal"),
            attr("count", 0),
            attr("keyType", "string"),
        ],
    );
    writer.empty(
        "CMvTrack_Live2DModel_Source",
        &[
            attr("xs.n", "trackSource"),
            attr("xs.ref", ref_id(ids.track)),
        ],
    );
    writer.end("FormAnimationSet");
    write_bounds(&mut writer, "bounds", 640.0, 1100.0);
    writer.end("CMvTrack_Live2DModel_Source");
    writer.empty(
        "CTrackGuid",
        &[
            attr("uuid", Uuid::new_v4()),
            attr("xs.id", ref_id(ids.guid)),
        ],
    );
    Ok(())
}

fn write_parameter_effect(
    xml: &mut String,
    effect: u32,
    track: u32,
    motion: &Motion3,
    fades: FadeTimes,
) -> Result<()> {
    let curves = motion
        .curves()
        .iter()
        .filter(|curve| curve.target() == "Parameter")
        .collect::<Vec<_>>();
    let mut writer = XmlWriter::new(xml);
    writer.start(
        "CMvEffect_Live2DParameter",
        &[attr("xs.id", ref_id(effect))],
    );
    write_effect_header(&mut writer, "Effects:Live2DParam", false, curves.len());
    for (index, curve) in curves.iter().enumerate() {
        write_attr(
            &mut writer,
            effect * 10_000 + index as u32,
            track,
            curve,
            motion.meta().fps(),
            fades.in_time,
            fades.out_time,
        );
    }
    writer.end("array");
    writer.start(
        "hash_map",
        &[
            attr("xs.n", "attrMap"),
            attr("count", curves.len()),
            attr("keyType", "string"),
        ],
    );
    for (index, curve) in curves.iter().enumerate() {
        let id = effect * 10_000 + index as u32;
        writer.start("entry", &[]);
        writer.empty(
            "CAttrId",
            &[attr("xs.n", "key"), attr("idstr", curve_key(curve))],
        );
        writer.empty(
            "CMvAttrF",
            &[attr("xs.n", "value"), attr("xs.ref", ref_id(id))],
        );
        writer.end("entry");
    }
    writer.end("hash_map");
    writer.empty(
        "CMvTrack_Live2DModel_Source",
        &[attr("xs.n", "track"), attr("xs.ref", ref_id(track))],
    );
    writer.end("ICMvEffect");
    writer.end("CMvEffect_Live2DParameter");
    Ok(())
}

fn write_attr(
    writer: &mut XmlWriter<'_>,
    id: u32,
    track: u32,
    curve: &MotionCurve,
    fps: f32,
    fade_in_time: Option<f32>,
    fade_out_time: Option<f32>,
) {
    let attr_id = if curve.target() == "PartOpacity" {
        format!("live2DPartsOpacity:{}", curve.id())
    } else {
        curve_key(curve)
    };
    let points = curve_points(curve, fps);
    let fade_in = curve
        .fade_in_time()
        .or(fade_in_time)
        .map_or(-1, |seconds| (seconds.max(0.0) * 1000.0).round() as i32);
    let fade_out = curve
        .fade_out_time()
        .or(fade_out_time)
        .map_or(-1, |seconds| (seconds.max(0.0) * 1000.0).round() as i32);
    writer.start("CMvAttrF", &[attr("xs.id", ref_id(id))]);
    writer.start("ICMvAttr", &[attr("xs.n", "super")]);
    writer.text("b", &[attr("xs.n", "isShyMode")], false);
    writer.empty("CAttrId", &[attr("xs.n", "id"), attr("idstr", &attr_id)]);
    writer.text("s", &[attr("xs.n", "name")], curve.id());
    writer.text("b", &[attr("xs.n", "isActive")], true);
    writer.start(
        "hash_map",
        &[
            attr("xs.n", "optionParam"),
            attr("count", 3),
            attr("keyType", "string"),
        ],
    );
    writer.text("i", &[attr("xs.n", "KEY_ATTR_FADE_OUT")], fade_out);
    writer.text("i", &[attr("xs.n", "KEY_ATTR_FADE_IN")], fade_in);
    writer.text("s", &[attr("xs.n", "KEY_PARAM_ID")], &attr_id);
    writer.end("hash_map");
    writer.empty(
        "CMvTrack_Live2DModel_Source",
        &[attr("xs.n", "track"), attr("xs.ref", ref_id(track))],
    );
    writer.end("ICMvAttr");
    writer.start("CMutableSequence", &[attr("xs.n", "valueData")]);
    writer.start("ACValueSequence", &[attr("xs.n", "super")]);
    writer.text(
        "d",
        &[attr("xs.n", "curMin")],
        points.iter().map(|p| p.value).fold(f32::INFINITY, f32::min),
    );
    writer.text(
        "d",
        &[attr("xs.n", "curMax")],
        points
            .iter()
            .map(|p| p.value)
            .fold(f32::NEG_INFINITY, f32::max),
    );
    writer.text("i", &[attr("xs.n", "posStart")], 0);
    writer.text("d", &[attr("xs.n", "baseValue")], curve.first_point().value);
    writer.end("ACValueSequence");
    writer.start(
        "array",
        &[
            attr("xs.n", "points"),
            attr("count", points.len()),
            attr("type", "CBezierPt"),
        ],
    );
    for point in &points {
        writer.start("CBezierPt", &[]);
        writer.start("CSeqPt", &[attr("xs.n", "anchor")]);
        writer.text("b", &[attr("xs.n", "isCorner")], false);
        writer.text("i", &[attr("xs.n", "pos")], point.frame);
        writer.text("d", &[attr("xs.n", "doubleValue")], point.value);
        writer.end("CSeqPt");
        write_control_point(writer, "next", point.next, fps);
        write_control_point(writer, "prev", point.prev, fps);
        writer.end("CBezierPt");
    }
    writer.end("array");
    writer.start(
        "carray_list",
        &[
            attr("xs.n", "curveTypes"),
            attr("count", points.len().saturating_sub(1)),
        ],
    );
    for point in points.iter().skip(1) {
        let curve_type = match point.kind {
            "LINEAR" => "LINEAR",
            "BEZIER" => "BEZIER",
            "STEP" => "STEP",
            "INVERSE_STEP" => "INVERSE_STEP",
            _ => "SMOOTH",
        };
        writer.empty("CCurveType", &[attr("v", curve_type)]);
    }
    writer.end("carray_list");
    writer.text("d", &[attr("xs.n", "rangeMin")], "-Infinity");
    writer.text("d", &[attr("xs.n", "rangeMax")], "Infinity");
    writer.text("b", &[attr("xs.n", "isRepeat")], false);
    writer.end("CMutableSequence");
    writer.end("CMvAttrF");
}

fn write_effect_header(xml: &mut XmlWriter<'_>, effect_id: &str, can_delete: bool, count: usize) {
    xml.start("ICMvEffect", &[attr("xs.n", "super")]);
    xml.empty("CEffectId", &[attr("xs.n", "id"), attr("idstr", effect_id)]);
    xml.text("b", &[attr("xs.n", "isActive")], true);
    xml.text("b", &[attr("xs.n", "canDelete")], can_delete);
    xml.start(
        "array",
        &[
            attr("xs.n", "attrList"),
            attr("count", count),
            attr("type", "ICMvAttr"),
        ],
    );
}

fn write_control_point(xml: &mut XmlWriter<'_>, name: &str, point: MotionPoint, fps: f32) {
    xml.start("CBezierCtrlPt", &[attr("xs.n", name)]);
    xml.text("f", &[attr("xs.n", "posF")], point.time);
    xml.text("i", &[attr("xs.n", "pos")], frame(point, fps));
    xml.text("d", &[attr("xs.n", "doubleValue")], point.value);
    xml.text("b", &[attr("xs.n", "isPosOptimized")], false);
    xml.end("CBezierCtrlPt");
}

fn curve_key(curve: &MotionCurve) -> String {
    match curve.target() {
        "PartOpacity" => format!("live2DPartsOpacity:{}", curve.id()),
        "Model" if curve.id() == "Opacity" => "opacity".into(),
        "Model" if curve.id() == "EyeBlink" => "eyeOpen".into(),
        "Model" if curve.id() == "LipSync" => "soundLevel".into(),
        "Model" => format!("model_{}", curve.id()),
        _ => format!("live2dParam_{}", curve.id()),
    }
}

struct CurvePoint {
    frame: i32,
    value: f32,
    kind: &'static str,
    next: MotionPoint,
    prev: MotionPoint,
}

fn curve_points(curve: &MotionCurve, fps: f32) -> Vec<CurvePoint> {
    let first = curve.first_point();
    let mut result = vec![CurvePoint {
        frame: frame(first, fps),
        value: first.value,
        kind: "",
        next: first,
        prev: first,
    }];
    let mut start = first;
    for segment in curve.segments() {
        let end = segment.end();
        let (next, prev) = match *segment {
            MotionSegment::Bezier {
                control1, control2, ..
            } => (control1, control2),
            MotionSegment::Linear { .. } => (
                MotionPoint {
                    time: start.time + (end.time - start.time) / 3.0,
                    value: start.value + (end.value - start.value) / 3.0,
                },
                MotionPoint {
                    time: start.time + (end.time - start.time) * 2.0 / 3.0,
                    value: start.value + (end.value - start.value) * 2.0 / 3.0,
                },
            ),
            MotionSegment::Stepped { .. } | MotionSegment::InverseStepped { .. } => (start, end),
        };
        if let Some(last) = result.last_mut() {
            last.next = next;
        }
        result.push(CurvePoint {
            frame: frame(end, fps),
            value: end.value,
            kind: segment_type(segment),
            next: end,
            prev,
        });
        start = end;
    }
    result
}

fn frame(point: MotionPoint, fps: f32) -> i32 {
    (point.time * fps).round() as i32
}
fn segment_type(segment: &MotionSegment) -> &'static str {
    match segment {
        MotionSegment::Linear { .. } => "LINEAR",
        MotionSegment::Bezier { .. } => "BEZIER",
        MotionSegment::Stepped { .. } => "STEP",
        MotionSegment::InverseStepped { .. } => "INVERSE_STEP",
    }
}

fn write_parts_effect(
    xml: &mut String,
    effect: u32,
    track: u32,
    motion: &Motion3,
    fades: FadeTimes,
) {
    let curves = motion
        .curves()
        .iter()
        .filter(|curve| curve.target() == "PartOpacity")
        .collect::<Vec<_>>();
    let mut writer = XmlWriter::new(xml);
    writer.start(
        "CMvEffect_Live2DPartsVisible",
        &[attr("xs.id", ref_id(effect))],
    );
    write_effect_header(
        &mut writer,
        "Effects:Live2DPartsOpacity",
        false,
        curves.len(),
    );
    for (index, curve) in curves.iter().enumerate() {
        write_attr(
            &mut writer,
            effect * 10_000 + index as u32,
            track,
            curve,
            motion.meta().fps(),
            fades.in_time,
            fades.out_time,
        );
    }
    writer.end("array");
    writer.start(
        "hash_map",
        &[
            attr("xs.n", "attrMap"),
            attr("count", curves.len()),
            attr("keyType", "string"),
        ],
    );
    for (index, curve) in curves.iter().enumerate() {
        let id = effect * 10_000 + index as u32;
        writer.start("entry", &[]);
        writer.empty(
            "CAttrId",
            &[
                attr("xs.n", "key"),
                attr("idstr", format!("live2DPartsOpacity:{}", curve.id())),
            ],
        );
        writer.empty(
            "CMvAttrF",
            &[attr("xs.n", "value"), attr("xs.ref", ref_id(id))],
        );
        writer.end("entry");
    }
    writer.end("hash_map");
    writer.empty(
        "CMvTrack_Live2DModel_Source",
        &[attr("xs.n", "track"), attr("xs.ref", ref_id(track))],
    );
    writer.end("ICMvEffect");
    writer.empty(
        "carray_list",
        &[attr("xs.n", "effectParameterAttrIds"), attr("count", 0)],
    );
    writer.end("CMvEffect_Live2DPartsVisible");
}

fn write_visual_effect(
    xml: &mut String,
    effect: u32,
    track: u32,
    motion: &Motion3,
    fades: FadeTimes,
) {
    let opacity = motion
        .curves()
        .iter()
        .find(|curve| curve.target() == "Model" && curve.id() == "Opacity");
    let count = usize::from(opacity.is_some());
    let mut writer = XmlWriter::new(xml);
    writer.start("CMvEffect_VisualDefault", &[attr("xs.id", ref_id(effect))]);
    write_effect_header(&mut writer, "VisualDefault", false, count);
    if let Some(curve) = opacity {
        write_attr(
            &mut writer,
            effect * 10_000,
            track,
            curve,
            motion.meta().fps(),
            fades.in_time,
            fades.out_time,
        );
    }
    writer.end("array");
    writer.empty(
        "hash_map",
        &[
            attr("xs.n", "attrMap"),
            attr("count", 0),
            attr("keyType", "string"),
        ],
    );
    writer.empty(
        "CMvTrack_Live2DModel_Source",
        &[attr("xs.n", "track"), attr("xs.ref", ref_id(track))],
    );
    writer.end("ICMvEffect");
    if opacity.is_some() {
        writer.empty(
            "CMvAttrF",
            &[
                attr("xs.n", "attrOpacity"),
                attr("xs.ref", ref_id(effect * 10_000)),
            ],
        );
    } else {
        writer.empty("null", &[attr("xs.n", "attrOpacity")]);
    }
    writer.end("CMvEffect_VisualDefault");
}

fn write_special_effects(
    xml: &mut String,
    track_ids: TrackIds,
    groups: &[Model3Group],
    motion: &Motion3,
    fades: FadeTimes,
) {
    for (id, name, target) in [
        (track_ids.eye, "EyeBlink", "EyeBlink"),
        (track_ids.lip, "LipSync", "LipSync"),
    ] {
        let group = groups
            .iter()
            .find(|group| group.name == name && group.target == "Parameter");
        let parameter_ids = group.map(|group| group.ids.as_slice()).unwrap_or(&[]);
        let model_id = if name == "EyeBlink" {
            "EyeBlink"
        } else {
            "LipSync"
        };
        let model_curve = motion
            .curves()
            .iter()
            .find(|curve| curve.target() == "Model" && curve.id() == model_id);
        let kind = if target == "EyeBlink" {
            "CMvEffect_EyeBlink"
        } else {
            "CMvEffect_LipSync"
        };
        let mut writer = XmlWriter::new(xml);
        writer.start(kind, &[attr("xs.id", ref_id(id))]);
        write_effect_header(
            &mut writer,
            &format!("Effects:{target}"),
            true,
            usize::from(model_curve.is_some()),
        );
        if let Some(curve) = model_curve {
            write_attr(
                &mut writer,
                id * 10_000,
                track_ids.track,
                curve,
                motion.meta().fps(),
                fades.in_time,
                fades.out_time,
            );
        }
        writer.end("array");
        writer.start(
            "hash_map",
            &[
                attr("xs.n", "attrMap"),
                attr("count", usize::from(model_curve.is_some())),
                attr("keyType", "string"),
            ],
        );
        if let Some(curve) = model_curve {
            writer.start("entry", &[]);
            writer.empty(
                "CAttrId",
                &[attr("xs.n", "key"), attr("idstr", curve_key(curve))],
            );
            writer.empty(
                "CMvAttrF",
                &[attr("xs.n", "value"), attr("xs.ref", ref_id(id * 10_000))],
            );
            writer.end("entry");
        }
        writer.end("hash_map");
        writer.empty(
            "CMvTrack_Live2DModel_Source",
            &[
                attr("xs.n", "track"),
                attr("xs.ref", ref_id(track_ids.track)),
            ],
        );
        writer.end("ICMvEffect");
        writer.start(
            "carray_list",
            &[
                attr("xs.n", "effectParameterAttrIds"),
                attr("count", parameter_ids.len()),
            ],
        );
        for parameter in parameter_ids {
            writer.empty(
                "CAttrId",
                &[attr("idstr", format!("live2dParam_{parameter}"))],
            );
        }
        writer.end("carray_list");
        writer.end(kind);
    }
}

fn write_animation(
    xml: &mut String,
    animation: u32,
    name: &str,
    scenes: &[(u32, u32)],
    manager: u32,
) {
    let mut writer = XmlWriter::new(xml);
    writer.start("CAnimation", &[attr("xs.id", ref_id(animation))]);
    writer.text("s", &[attr("xs.n", "name")], name);
    writer.empty("file", &[attr("xs.n", "file")]);
    writer.start(
        "carray_list",
        &[attr("xs.n", "_scenes"), attr("count", scenes.len())],
    );
    for (scene, _) in scenes {
        writer.empty("CSceneSource", &[attr("xs.ref", ref_id(*scene))]);
    }
    writer.end("carray_list");
    if let Some((scene, _)) = scenes.first() {
        writer.empty(
            "CSceneSource",
            &[attr("xs.n", "currentScene"), attr("xs.ref", ref_id(*scene))],
        );
    } else {
        writer.empty("null", &[attr("xs.n", "currentScene")]);
    }
    writer.empty(
        "CResourceManager",
        &[
            attr("xs.n", "resourceManager"),
            attr("xs.ref", ref_id(manager)),
        ],
    );
    writer.start("EditorEdition", &[attr("xs.n", "editorEdition")]);
    writer.text("i", &[attr("xs.n", "edition")], 15);
    writer.end("EditorEdition");
    writer.empty(
        "Animation",
        &[attr("xs.n", "targetVersion"), attr("v", "FOR_SDK")],
    );
    writer.end("CAnimation");
}

fn write_resource_manager(
    xml: &mut String,
    manager: u32,
    group: u32,
    resource: u32,
    data: u32,
    model_path: &str,
) {
    let mut writer = XmlWriter::new(xml);
    writer.start("CResourceManager", &[attr("xs.id", ref_id(manager))]);
    writer.start("CResourceGroup", &[attr("xs.n", "rootGroup")]);
    writer.empty(
        "CResourceGroupGuid",
        &[attr("xs.n", "guid"), attr("xs.ref", ref_id(group))],
    );
    writer.start(
        "carray_list",
        &[attr("xs.n", "_childGuids"), attr("count", 1)],
    );
    writer.empty("CResourceGuid", &[attr("xs.ref", ref_id(resource))]);
    writer.end("carray_list");
    writer.text("s", &[attr("xs.n", "name")], "Resources");
    writer.end("CResourceGroup");
    writer.start(
        "carray_list",
        &[attr("xs.n", "_resourceRefList"), attr("count", 1)],
    );
    writer.empty("CResourceData", &[attr("xs.ref", ref_id(data))]);
    writer.end("carray_list");
    writer.start("CResourceData", &[attr("xs.id", ref_id(data))]);
    writer.start("CResource_Linked_Model", &[attr("xs.n", "resourceRef")]);
    writer.start("ACResource_File", &[attr("xs.n", "super")]);
    writer.text("file", &[attr("xs.n", "srcFile")], model_path);
    writer.empty(
        "CResourceGuid",
        &[attr("xs.n", "guid"), attr("xs.ref", ref_id(resource))],
    );
    writer.text("s", &[attr("xs.n", "name")], model_path);
    writer.end("ACResource_File");
    writer.end("CResource_Linked_Model");
    writer.end("CResourceData");
    writer.end("CResourceManager");
    writer.empty("CResourceGroupGuid", &[attr("xs.id", ref_id(group))]);
    writer.empty("CResourceGuid", &[attr("xs.id", ref_id(resource))]);
}
