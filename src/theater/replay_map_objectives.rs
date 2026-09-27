//! Static objective projection with per-role display and point precedence rules.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ObjectiveRoleSpec {
    pub role: String,
    pub neutral: bool,
    pub points_only: bool,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplayMapObjectives {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub zones: Vec<ReplayMapObjectiveZone>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub markers: Vec<ReplayMapObjectiveMarker>,
}
fn zero(v: &f32) -> bool {
    *v == 0.
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ReplayMapObjectiveZone {
    pub role: String,
    pub team: i64,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub family: String,
    #[serde(skip_serializing_if = "zero")]
    pub half_x: f32,
    #[serde(skip_serializing_if = "zero")]
    pub half_y: f32,
    #[serde(skip_serializing_if = "zero")]
    pub radius: f32,
    pub fwd_x: f32,
    pub fwd_y: f32,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayMapObjectiveMarker {
    pub role: String,
    pub team: i64,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
fn team(t: i64, neutral: bool) -> i64 {
    if neutral { -1 } else { t }
}
pub fn build_replay_map_objectives(
    entry: &MapObjectivesEntry,
    specs: &[ObjectiveRoleSpec],
) -> Option<ReplayMapObjectives> {
    let mut out = ReplayMapObjectives::default();
    let mut seen = BTreeSet::new();
    for spec in specs {
        if !seen.insert(spec.role.as_str()) {
            continue;
        }
        let points = entry.points_of_role(&spec.role);
        // Canonical point camps intentionally use the role's neutral override;
        // the per-point neutral label is applied at marker publication below.
        let camps: BTreeSet<_> = if spec.points_only {
            points
                .iter()
                .map(|p| team(p.team_index, spec.neutral))
                .collect()
        } else {
            BTreeSet::new()
        };
        for z in entry.zones_of_role(&spec.role).zones {
            let display = team(z.team_index, spec.neutral);
            if spec.points_only {
                if !camps.contains(&display) {
                    out.markers.push(ReplayMapObjectiveMarker {
                        role: z.role,
                        team: display,
                        x: z.center.x as f32,
                        y: z.center.y as f32,
                        z: z.center.z as f32,
                    });
                }
                continue;
            }
            let mut dto = ReplayMapObjectiveZone {
                role: z.role,
                team: display,
                x: z.center.x as f32,
                y: z.center.y as f32,
                z: z.center.z as f32,
                family: z.shape.family.clone(),
                fwd_x: z.shape.forward.x as f32,
                fwd_y: z.shape.forward.y as f32,
                ..Default::default()
            };
            match z.shape.family.as_str() {
                "box" => {
                    dto.half_x = z.shape.half_x.unwrap_or(0.) as f32;
                    dto.half_y = z.shape.half_y.unwrap_or(0.) as f32;
                }
                "cylinder" => dto.radius = z.shape.radius.unwrap_or(0.) as f32,
                _ => {}
            }
            out.zones.push(dto);
        }
        out.markers
            .extend(points.into_iter().map(|p| ReplayMapObjectiveMarker {
                role: p.role,
                team: team(p.team_index, p.neutral || spec.neutral),
                x: p.center.x as f32,
                y: p.center.y as f32,
                z: p.center.z as f32,
            }));
    }
    if out.zones.is_empty() && out.markers.is_empty() {
        None
    } else {
        Some(out)
    }
}
