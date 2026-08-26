use std::fmt::Write;

use quick_xml::escape::escape;
use uuid::Uuid;

use crate::{
    Error, Result,
    model3::Model3Group,
    motion3::{Motion3, MotionCurve, MotionPoint, MotionSegment},
};

/// Builds a CAN3 document from a model link and parsed motion files.
pub(crate) fn generate(
    animation_name: &str,
    model_path: &str,
    motions: &[(String, Motion3)],
    groups: &[Model3Group],
) -> Result<Vec<u8>> {
    if motions.is_empty() {
        return Err(Error::InvalidCan3("model has no motion files".into()));
    }
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
    for (name, motion) in motions {
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
            track,
            track_guid,
            scene,
            parameter_effect,
            parts_effect,
            eye_effect,
            lip_effect,
            motion,
            groups,
            resource,
            root_track_guid,
        )?;
        write_parameter_effect(&mut xml, parameter_effect, track, motion)?;
        write_parts_effect(&mut xml, parts_effect, track);
        write_special_effects(&mut xml, eye_effect, lip_effect, track, groups);
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
    writeln!(xml, "<CAnimation xs.ref=\"#{}\" />", animation).unwrap();
    xml.push_str("</main></root>");
    Ok(xml.into_bytes())
}

fn write_header(xml: &mut String) {
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<?version CSceneSource:3?>\n<?version CAnimation:4?>\n<?version CMvParameter_Group:1?>\n<?version SerializeFormatVersion:2?>\n<?version CMvMovieInfo:3?>\n<?version CBezierCtrlPt:2?>\n");
}

fn write_group_track(xml: &mut String, group: u32, scene: u32, guid: u32, track: u32) {
    writeln!(xml, "<CMvTrack_Group_Source xs.id=\"#{}\"><ICMvTrack_Source xs.n=\"super\"><s xs.n=\"name\">Root</s><b xs.n=\"isUserRenamed\">false</b><CTrackGuid xs.n=\"guid\" xs.ref=\"#{}\" /><i xs.n=\"start\">0</i><i xs.n=\"internalOffset\">0</i><i xs.n=\"duration\">0</i><b xs.n=\"editable\">true</b><b xs.n=\"visible\">true</b><b xs.n=\"mute\">false</b><b xs.n=\"isGuide\">false</b><b xs.n=\"isRepeat\">false</b><b xs.n=\"soloSwitch\">false</b><null xs.n=\"soundEffect\" /><null xs.n=\"visualEffect\" /><CMvEffectManager xs.n=\"effectManager\"><array xs.n=\"effectList\" count=\"0\" type=\"ICMvEffect\" /></CMvEffectManager><null xs.n=\"parentGuid\" /><CSceneSource xs.n=\"_sceneSource\" xs.ref=\"#{}\" /><hash_map xs.n=\"userData\" count=\"0\" keyType=\"string\" /></ICMvTrack_Source><carray_list xs.n=\"_childTrackGuids\" count=\"1\"><CTrackGuid xs.ref=\"#{}\" /></carray_list><GRectF xs.n=\"bounds\"><f xs.n=\"x\">0.0</f><f xs.n=\"y\">0.0</f><f xs.n=\"width\">640.0</f><f xs.n=\"height\">480.0</f></GRectF></CMvTrack_Group_Source><CTrackGuid uuid=\"{}\" xs.id=\"#{}\" />", group, guid, scene, track, Uuid::new_v4(), guid).unwrap();
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
    writeln!(xml, "<CSceneSource xs.id=\"#{}\"><s xs.n=\"sceneName\">{}</s><CImageCanvas xs.n=\"canvas\"><i xs.n=\"pixelWidth\">320</i><i xs.n=\"pixelHeight\">240</i><CColor xs.n=\"background\" /></CImageCanvas><CSceneGuid xs.n=\"guid\" xs.ref=\"#{}\" /><s xs.n=\"tag\" /><CTrackSourceSet xs.n=\"trackSourceSet\"><carray_list xs.n=\"_sources\" count=\"2\"><CMvTrack_Group_Source xs.ref=\"#{}\" /><CMvTrack_Live2DModel_Source xs.ref=\"#{}\" /></carray_list></CTrackSourceSet><CMvTrack_Group_Source xs.n=\"rootTrack\" xs.ref=\"#{}\" /><CMvMovieInfo xs.n=\"movieInfo\"><i xs.n=\"width\">320</i><i xs.n=\"height\">240</i><i xs.n=\"duration\">{}</i><d xs.n=\"fps\">{}</d><i xs.n=\"workspaceStart\">0</i><i xs.n=\"workspaceEnd\">{}</i><CColor xs.n=\"background\" /><i xs.n=\"fadeInMSec\">-1</i><i xs.n=\"fadeOutMSec\">-1</i><b xs.n=\"isBezierRestricted\">{}</b><b xs.n=\"isLoopMotion\">{}</b><i xs.n=\"startFrame\">0</i><CFrameIndexType xs.n=\"frameIndexType\" v=\"ZERO_INDEX\" /></CMvMovieInfo><hash_map xs.n=\"marker\" count=\"0\" keyType=\"string\" /><CCurveType xs.n=\"defaultParameterCurveType\" v=\"SMOOTH\" /><CCurveType xs.n=\"defaultPartCurveType\" v=\"STEP\" /><b xs.n=\"fixAspect\">true</b><Animation xs.n=\"targetVersion\" v=\"FOR_SDK\" /></CSceneSource><CSceneGuid uuid=\"{}\" xs.id=\"#{}\" />", scene, escape(name), scene_guid, group, track, group, frames + 1, motion.meta().fps(), frames, motion.meta().are_beziers_restricted(), motion.meta().is_looping(), Uuid::new_v4(), scene_guid).unwrap();
}

fn write_model_track(
    xml: &mut String,
    track: u32,
    guid: u32,
    scene: u32,
    parameter: u32,
    parts: u32,
    eye: u32,
    lip: u32,
    motion: &Motion3,
    groups: &[Model3Group],
    resource: u32,
    parent_guid: u32,
) -> Result<()> {
    let duration = (motion.meta().duration() * motion.meta().fps())
        .ceil()
        .max(1.0) as u32;
    writeln!(xml, "<CMvTrack_Live2DModel_Source xs.id=\"#{}\"><ICMvTrack_Linked xs.n=\"super\"><ICMvTrack_Source xs.n=\"super\"><s xs.n=\"name\">Model</s><b xs.n=\"isUserRenamed\">true</b><CTrackGuid xs.n=\"guid\" xs.ref=\"#{}\" /><i xs.n=\"start\">0</i><i xs.n=\"internalOffset\">0</i><i xs.n=\"duration\">{}</i><b xs.n=\"editable\">true</b><b xs.n=\"visible\">true</b><b xs.n=\"mute\">false</b><b xs.n=\"isGuide\">false</b><b xs.n=\"isRepeat\">false</b><b xs.n=\"soloSwitch\">false</b><null xs.n=\"soundEffect\" /><CMvEffectManager xs.n=\"effectManager\"><array xs.n=\"effectList\" count=\"4\" type=\"ICMvEffect\"><CMvEffect_EyeBlink xs.ref=\"#{}\" /><CMvEffect_LipSync xs.ref=\"#{}\" /><CMvEffect_Live2DParameter xs.ref=\"#{}\" /><CMvEffect_Live2DPartsVisible xs.ref=\"#{}\" /></array></CMvEffectManager><CTrackGuid xs.n=\"parentGuid\" xs.ref=\"#{}\" /><CSceneSource xs.n=\"_sceneSource\" xs.ref=\"#{}\" /><hash_map xs.n=\"userData\" count=\"0\" keyType=\"string\" /></ICMvTrack_Source><CResourceGuid xs.n=\"_resourceGuid\" xs.ref=\"#{}\" /></ICMvTrack_Linked><CMvEffect_Live2DParameter xs.n=\"keyParamEffect\" xs.ref=\"#{}\" /><CMvEffect_Live2DPartsVisible xs.n=\"partsVisibleEffect\" xs.ref=\"#{}\" /><CMvEffect_EyeBlink xs.n=\"eyeBlinkEffect\" xs.ref=\"#{}\" /><CMvEffect_LipSync xs.n=\"lipSyncEffect\" xs.ref=\"#{}\" /><null xs.n=\"formEditEffect\" /><GRectF xs.n=\"bounds\"><f xs.n=\"x\">0</f><f xs.n=\"y\">0</f><f xs.n=\"width\">640</f><f xs.n=\"height\">1100</f></GRectF></CMvTrack_Live2DModel_Source><CTrackGuid uuid=\"{}\" xs.id=\"#{}\" />", track, guid, duration, eye, lip, parameter, parts, parent_guid, scene, resource, parameter, parts, eye, lip, Uuid::new_v4(), guid).unwrap();
    let _ = groups;
    Ok(())
}

fn write_parameter_effect(
    xml: &mut String,
    effect: u32,
    track: u32,
    motion: &Motion3,
) -> Result<()> {
    let curves = motion
        .curves()
        .iter()
        .filter(|curve| curve.target() == "Parameter")
        .collect::<Vec<_>>();
    writeln!(xml, "<CMvEffect_Live2DParameter xs.id=\"#{}\"><ICMvEffect xs.n=\"super\"><CEffectId xs.n=\"id\" idstr=\"Effects:Live2DParam\" /><b xs.n=\"isActive\">true</b><b xs.n=\"canDelete\">false</b><array xs.n=\"attrList\" count=\"{}\" type=\"ICMvAttr\">", effect, curves.len()).unwrap();
    for (index, curve) in curves.iter().enumerate() {
        write_attr(
            xml,
            effect * 10_000 + index as u32,
            track,
            curve,
            motion.meta().fps(),
        );
    }
    xml.push_str("</array><hash_map xs.n=\"attrMap\" count=\"");
    write!(xml, "{}\">", curves.len()).unwrap();
    for (index, curve) in curves.iter().enumerate() {
        let id = effect * 10_000 + index as u32;
        writeln!(xml, "<entry><CAttrId xs.n=\"key\" idstr=\"live2dParam_{}\" /><CMvAttrF xs.n=\"value\" xs.ref=\"#{}\" /></entry>", escape(curve.id()), id).unwrap();
    }
    xml.push_str("</hash_map>");
    writeln!(xml, "<CMvTrack_Live2DModel_Source xs.n=\"track\" xs.ref=\"#{}\" /></ICMvEffect></CMvEffect_Live2DParameter>", track).unwrap();
    Ok(())
}

fn write_attr(xml: &mut String, id: u32, track: u32, curve: &MotionCurve, fps: f32) {
    let attr_id = if curve.target() == "PartOpacity" {
        format!("live2DPartsOpacity:{}", curve.id())
    } else {
        format!("live2dParam_{}", curve.id())
    };
    let points = curve_points(curve, fps);
    writeln!(xml, "<CMvAttrF xs.id=\"#{}\"><ICMvAttr xs.n=\"super\"><b xs.n=\"isShyMode\">false</b><CAttrId xs.n=\"id\" idstr=\"{}\" /><s xs.n=\"name\">{}</s><b xs.n=\"isActive\">true</b><hash_map xs.n=\"optionParam\" count=\"0\" keyType=\"string\" /><CMvTrack_Live2DModel_Source xs.n=\"track\" xs.ref=\"#{}\" /></ICMvAttr><CMutableSequence xs.n=\"valueData\"><ACValueSequence xs.n=\"super\"><d xs.n=\"curMin\">{}</d><d xs.n=\"curMax\">{}</d><i xs.n=\"posStart\">0</i><d xs.n=\"baseValue\">{}</d></ACValueSequence><array xs.n=\"points\" count=\"{}\" type=\"CBezierPt\">", id, escape(&attr_id), escape(curve.id()), track, points.iter().map(|p| p.value).fold(f32::INFINITY, f32::min), points.iter().map(|p| p.value).fold(f32::NEG_INFINITY, f32::max), curve.first_point().value, points.len()).unwrap();
    for point in &points {
        writeln!(xml, "<CBezierPt><CSeqPt xs.n=\"anchor\"><b xs.n=\"isCorner\">false</b><i xs.n=\"pos\">{}</i><d xs.n=\"doubleValue\">{}</d></CSeqPt><CBezierCtrlPt xs.n=\"next\"><f xs.n=\"posF\">{}</f><i xs.n=\"pos\">{}</i><d xs.n=\"doubleValue\">{}</d><b xs.n=\"isPosOptimized\">false</b></CBezierCtrlPt><CBezierCtrlPt xs.n=\"prev\"><f xs.n=\"posF\">{}</f><i xs.n=\"pos\">{}</i><d xs.n=\"doubleValue\">{}</d><b xs.n=\"isPosOptimized\">false</b></CBezierCtrlPt></CBezierPt>", point.frame, point.value, point.next.time, frame(point.next, fps), point.next.value, point.prev.time, frame(point.prev, fps), point.prev.value).unwrap();
    }
    writeln!(
        xml,
        "</array><carray_list xs.n=\"curveTypes\" count=\"{}\">",
        points.len().saturating_sub(1)
    )
    .unwrap();
    for point in points.iter().skip(1) {
        let curve_type = match point.kind {
            "LINEAR" => "LINEAR",
            "BEZIER" => "BEZIER",
            "STEP" => "STEP",
            "INVERSE_STEP" => "INVERSE_STEP",
            _ => "SMOOTH",
        };
        writeln!(xml, "<CCurveType v=\"{}\" />", curve_type).unwrap();
    }
    xml.push_str("</carray_list><d xs.n=\"rangeMin\">-Infinity</d><d xs.n=\"rangeMax\">Infinity</d><b xs.n=\"isRepeat\">false</b></CMutableSequence></CMvAttrF>");
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

fn write_parts_effect(xml: &mut String, effect: u32, track: u32) {
    writeln!(xml, "<CMvEffect_Live2DPartsVisible xs.id=\"#{}\"><ICMvEffect xs.n=\"super\"><CEffectId xs.n=\"id\" idstr=\"Effects:Live2DPartsOpacity\" /><b xs.n=\"isActive\">true</b><b xs.n=\"canDelete\">false</b><array xs.n=\"attrList\" count=\"0\" type=\"ICMvAttr\" /><hash_map xs.n=\"attrMap\" count=\"0\" keyType=\"string\" /><CMvTrack_Live2DModel_Source xs.n=\"track\" xs.ref=\"#{}\" /></ICMvEffect><carray_list xs.n=\"effectParameterAttrIds\" count=\"0\" /></CMvEffect_Live2DPartsVisible>", effect, track).unwrap();
}

fn write_special_effects(xml: &mut String, eye: u32, lip: u32, track: u32, groups: &[Model3Group]) {
    for (id, name, target) in [(eye, "EyeBlink", "EyeBlink"), (lip, "LipSync", "LipSync")] {
        let group = groups
            .iter()
            .find(|group| group.name == name && group.target == "Parameter");
        let ids = group.map(|group| group.ids.as_slice()).unwrap_or(&[]);
        let kind = if target == "EyeBlink" {
            "CMvEffect_EyeBlink"
        } else {
            "CMvEffect_LipSync"
        };
        writeln!(xml, "<{} xs.id=\"#{}\"><ICMvEffect xs.n=\"super\"><CEffectId xs.n=\"id\" idstr=\"Effects:{}\" /><b xs.n=\"isActive\">true</b><b xs.n=\"canDelete\">true</b><array xs.n=\"attrList\" count=\"0\" type=\"ICMvAttr\" /><hash_map xs.n=\"attrMap\" count=\"0\" keyType=\"string\" /><CMvTrack_Live2DModel_Source xs.n=\"track\" xs.ref=\"#{}\" /></ICMvEffect><carray_list xs.n=\"effectParameterAttrIds\" count=\"{}\">", kind, id, target, track, ids.len()).unwrap();
        for parameter in ids {
            writeln!(
                xml,
                "<CAttrId idstr=\"live2dParam_{}\" />",
                escape(parameter)
            )
            .unwrap();
        }
        writeln!(xml, "</carray_list></{}>", kind).unwrap();
    }
}

fn write_animation(
    xml: &mut String,
    animation: u32,
    name: &str,
    scenes: &[(u32, u32)],
    manager: u32,
) {
    writeln!(xml, "<CAnimation xs.id=\"#{}\"><s xs.n=\"name\">{}</s><file xs.n=\"file\" /><carray_list xs.n=\"_scenes\" count=\"{}\">", animation, escape(name), scenes.len()).unwrap();
    for (scene, _) in scenes {
        writeln!(xml, "<CSceneSource xs.ref=\"#{}\" />", scene).unwrap();
    }
    writeln!(xml, "</carray_list><CSceneSource xs.n=\"currentScene\" xs.ref=\"#{}\" /><CResourceManager xs.n=\"resourceManager\" xs.ref=\"#{}\" /><EditorEdition xs.n=\"editorEdition\"><i xs.n=\"edition\">15</i></EditorEdition><Animation xs.n=\"targetVersion\" v=\"FOR_SDK\" /></CAnimation>", scenes[0].0, manager).unwrap();
}

fn write_resource_manager(
    xml: &mut String,
    manager: u32,
    group: u32,
    resource: u32,
    data: u32,
    model_path: &str,
) {
    writeln!(xml, "<CResourceManager xs.id=\"#{}\"><CResourceGroup xs.n=\"rootGroup\"><CResourceGroupGuid xs.n=\"guid\" xs.ref=\"#{}\" /><carray_list xs.n=\"_childGuids\" count=\"1\"><CResourceGuid xs.ref=\"#{}\" /></carray_list><s xs.n=\"name\">Resources</s></CResourceGroup><carray_list xs.n=\"_resourceRefList\" count=\"1\"><CResourceData xs.ref=\"#{}\" /></carray_list><CResourceData xs.id=\"#{}\"><CResource_Linked_Model xs.n=\"resourceRef\"><ACResource_File xs.n=\"super\"><file xs.n=\"srcFile\">{}</file><CResourceGuid xs.n=\"guid\" xs.ref=\"#{}\" /><s xs.n=\"name\">{}</s></ACResource_File></CResource_Linked_Model></CResourceData></CResourceManager><CResourceGroupGuid xs.id=\"#{}\" /><CResourceGuid xs.id=\"#{}\" />", manager, group, resource, data, data, escape(model_path), resource, escape(model_path), group, resource).unwrap();
}
