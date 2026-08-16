//! A three-detent sorting index: hollow, ascending, or descending.
//! The bronze pointer rises through one black aperture and settles on a
//! sprung cam; deactivation withdraws it below the casing.

#![deny(missing_docs)]

use std::{collections::HashMap, ops::Deref, sync::Arc};

use egui::{CursorIcon, Pos2, Rect, Sense, Stroke, Vec2, WidgetInfo, WidgetType};

use super::plunger::{BakedMesh, SpringLaw};
use super::{HOT, MechanismSize, foundry, plunger};

const APERTURE_HORIZON: f32 = -2.4;
const SPRING: SpringLaw = SpringLaw {
    stiffness: 760.0,
    damping: 31.0,
    restitution: 0.16,
    floor: baked::RETRACT - 0.5,
    ceiling: baked::CEILING,
};

#[derive(Clone, Copy)]
struct BakedSortPose {
    pointers: [BakedMesh; 2],
    shadows: [BakedMesh; 2],
}

#[derive(Clone, Copy)]
struct BakedSortGauge {
    side: u8,
    socket_half: f32,
    pointer_area: f32,
    poses: &'static [BakedSortPose],
}

mod baked {
    use super::{BakedMesh, BakedSortGauge, BakedSortPose};
    use crate::chrome::plunger::BakedVertex;

    include!(concat!(env!("OUT_DIR"), "/sort_toggle_atlas.rs"));
}

/// One of the sorting mechanism's three physical detents.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SortDetent {
    /// Empty black aperture with only its bronze casing visible.
    #[default]
    Off,
    /// Bronze pointer aimed up-screen.
    Ascending,
    /// Bronze pointer aimed down-screen.
    Descending,
}

impl SortDetent {
    const fn next(self) -> Self {
        match self {
            Self::Off => Self::Ascending,
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Off,
        }
    }

    const fn direction(self) -> Option<usize> {
        match self {
            Self::Off => None,
            Self::Ascending => Some(0),
            Self::Descending => Some(1),
        }
    }

    const fn accessible_name(self) -> &'static str {
        match self {
            Self::Off => "Sorting off",
            Self::Ascending => "Sort ascending",
            Self::Descending => "Sort descending",
        }
    }
}

/// A three-state Poolrooms sorting mechanism.
///
/// The `Off` detent is a literal hollow casing: a bronze rim around black
/// visible interior. Activation cycles `Off → Ascending → Descending → Off`.
/// [`SortToggle::size`] selects one independently forged S/M/L gauge.
pub struct SortToggle<'a> {
    detent: &'a mut SortDetent,
    size: MechanismSize,
}

impl<'a> SortToggle<'a> {
    /// Bind the mechanism to one sorting detent.
    pub const fn new(detent: &'a mut SortDetent) -> Self {
        Self {
            detent,
            size: MechanismSize::Large,
        }
    }

    /// Select one independently forged S/M/L casing.
    pub const fn size(mut self, size: MechanismSize) -> Self {
        self.size = size;
        self
    }

    /// Lay out, actuate, and paint the sorting mechanism.
    pub fn show(self, ui: &mut egui::Ui) -> SortToggleResponse {
        let atlas = self.size.atlas_index();
        let gauge = baked::GAUGES[atlas];
        debug_assert_eq!(gauge.side, self.size.side() as u8);
        let (rect, mut response) =
            ui.allocate_exact_size(Vec2::splat(self.size.side()), Sense::click());
        let enabled = ui.is_enabled();
        if enabled {
            response = response.on_hover_cursor(CursorIcon::PointingHand);
        }
        let activated = super::exact_activation(ui, &response);
        if enabled && activated {
            *self.detent = self.detent.next();
            response.mark_changed();
        }
        response.widget_info(|| {
            WidgetInfo::labeled(WidgetType::Button, enabled, self.detent.accessible_name())
        });

        let active = ui.ctx().data_mut(|data| {
            let key = response.id.with("sort-pointer-direction");
            if self.detent.direction().is_some() {
                let _old = data.insert_temp(key, *self.detent);
            }
            data.get_temp::<SortDetent>(key)
        });
        let target = if *self.detent == SortDetent::Off {
            baked::RETRACT
        } else {
            baked::REST
        };
        let strike = (enabled && activated).then_some(if *self.detent == SortDetent::Off {
            -24.0
        } else {
            24.0
        });
        let dt = ui
            .input(|input| input.stable_dt)
            .clamp(1.0 / 240.0, 1.0 / 30.0);
        let motion = plunger::motion(ui, response.id, target, target, strike, dt, SPRING);
        let socket = Rect::from_center_size(rect.center(), Vec2::splat(gauge.socket_half * 2.0));
        let painter = ui.painter();
        foundry::socket_bed(painter, socket);
        if motion.position > APERTURE_HORIZON
            && let Some(direction) = active.and_then(SortDetent::direction)
        {
            let pose = plunger::pose_index(
                motion.position,
                baked::RETRACT,
                baked::CEILING,
                gauge.poses.len(),
            );
            let rendered = ui.ctx().data_mut(|data| {
                data.get_temp_mut_or_default::<PoseCache>(response.id.with("sort-toggle-foundry"))
                    .prepare(socket.center(), atlas, pose, direction, gauge.poses)
            });
            let aperture = socket.shrink(foundry::RIM_WIDTH);
            foundry::paint_compiled(painter, aperture, &rendered.shadow);
            foundry::paint_compiled(painter, aperture, &rendered.pointer);
        }
        foundry::socket_rim(painter, socket);
        if response.has_focus() {
            let _focus = painter.rect_stroke(
                rect.shrink(0.5),
                1.0,
                Stroke::new(1.0, HOT.gamma_multiply(0.44)),
                egui::StrokeKind::Inside,
            );
        }
        super::tension(ui, &response);

        SortToggleResponse {
            wake: SortToggleWake::new(socket, motion.travel, gauge.pointer_area),
            response,
            activated: enabled && activated,
        }
    }
}

#[must_use = "the response carries both egui state and displaced-water volume"]
/// Interaction state and displaced-water geometry from one sort-toggle frame.
pub struct SortToggleResponse {
    response: egui::Response,
    wake: Option<SortToggleWake>,
    activated: bool,
}

impl SortToggleResponse {
    /// Whether pointer, accessibility, or exact keyboard activation changed detent.
    pub const fn changed(&self) -> bool {
        self.activated
    }

    /// The indicator volume swept since the preceding frame, if it moved.
    pub const fn wake(&self) -> Option<SortToggleWake> {
        self.wake
    }
}

impl Deref for SortToggleResponse {
    type Target = egui::Response;

    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

/// Signed volume swept by the triangular sorting pointer.
#[derive(Clone, Copy, Debug)]
pub struct SortToggleWake {
    rect: Rect,
    travel: f32,
    volume: f32,
}

impl SortToggleWake {
    fn new(rect: Rect, travel: f32, pointer_area: f32) -> Option<Self> {
        (travel.abs() >= 0.002).then_some(Self {
            rect,
            travel,
            volume: pointer_area * travel.abs(),
        })
    }

    /// Screen-space aperture occupied by the moving pointer.
    pub const fn rect(self) -> Rect {
        self.rect
    }

    /// Signed travel normal to the faceplate.
    pub const fn travel(self) -> f32 {
        self.travel
    }

    /// Absolute swept volume in logical point³.
    pub const fn swept_volume(self) -> f32 {
        self.volume
    }
}

#[derive(Clone)]
struct InstalledPose {
    pointer: Arc<egui::Mesh>,
    shadow: Arc<egui::Mesh>,
}

#[derive(Clone, Default)]
struct PoseCache {
    origin: Option<Pos2>,
    atlas: usize,
    poses: HashMap<(usize, usize), InstalledPose>,
}

impl PoseCache {
    fn prepare(
        &mut self,
        origin: Pos2,
        atlas: usize,
        pose: usize,
        direction: usize,
        poses: &'static [BakedSortPose],
    ) -> InstalledPose {
        if self.origin != Some(origin) || self.atlas != atlas {
            *self = Self {
                origin: Some(origin),
                atlas,
                ..Self::default()
            };
        }
        self.poses
            .entry((pose, direction))
            .or_insert_with(|| InstalledPose {
                pointer: plunger::instantiate(poses[pose].pointers[direction], origin),
                shadow: plunger::instantiate(poses[pose].shadows[direction], origin),
            })
            .clone()
    }
}
