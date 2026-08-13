//! Drag actuators mounted to riveted, crosshatched backplates.
//!
//! Square pulls admit either a sprung folding bail or the same bail rigidly
//! mounted. The dense-tool-row variant is a fixed half-width friction pad.
//! Plates, raised knurling, peened rivets, lugs, round stock, projection,
//! illumination, and cast shadows originate as three-dimensional geometry.
//! Build time compiles each supported footprint and hinge pose into a 2D vector
//! atlas; runtime owns only interaction and the optional sprung hinge coordinate.

#![deny(missing_docs)]

use std::{collections::HashMap, ops::Deref, sync::Arc};

use egui::{CursorIcon, Pos2, Rect, Sense, Stroke, Vec2, WidgetInfo, WidgetType};

use super::{HOT, MechanismSize, foundry};

use super::mechanism::{CouplingPorts, CouplingTarget, sealed};
use super::plunger::{self, BakedMesh, BakedVertex, Motion, SpringLaw};

const SPRING_LAW: SpringLaw = SpringLaw {
    stiffness: 420.0,
    damping: 25.0,
    restitution: 0.08,
    floor: baked::POSE_MIN,
    ceiling: baked::POSE_MAX,
};

#[derive(Clone, Copy)]
struct BakedBailPose {
    angle: f32,
    shadow: BakedMesh,
    bail: BakedMesh,
}

#[derive(Clone, Copy)]
struct BakedBailGauge {
    side: u8,
    plate: BakedMesh,
    floor_shadow: BakedMesh,
    static_shadow: BakedMesh,
    hardware: BakedMesh,
    sweep_per_radian: f32,
    poses: &'static [BakedBailPose],
}

#[derive(Clone, Copy)]
struct BakedFrictionGauge {
    side: u8,
    width: f32,
    plate: BakedMesh,
    floor_shadow: BakedMesh,
    static_shadow: BakedMesh,
    hardware: BakedMesh,
}

mod baked {
    use super::{BakedBailGauge, BakedBailPose, BakedFrictionGauge, BakedMesh, BakedVertex};

    include!(concat!(env!("OUT_DIR"), "/drag_handle_atlas.rs"));
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mechanism {
    FoldingBail,
    RigidBail,
    FrictionPad,
}

/// A forged drag actuator that captures planar drag gestures.
///
/// [`DragHandle::folding_bail`] rotates toward the viewer under pointer
/// pressure and seats with a short, stiff rebound. [`DragHandle::rigid_bail`]
/// retains that square silhouette without a free hinge.
/// [`DragHandle::friction_pad`] is a rigid, half-width knurled grip for dense
/// tool rows. The ordinary `egui::Response` drag methods remain available through
/// [`DragHandleResponse`]'s `Deref` implementation.
///
/// # Example
///
/// ```
/// use dwemer_poolrooms::{chrome::{DragHandle, MechanismSize}, egui};
///
/// fn reorder_grip(ui: &mut egui::Ui) -> egui::Vec2 {
///     DragHandle::new()
///         .size(MechanismSize::Small)
///         .show(ui)
///         .drag_delta()
/// }
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DragHandle {
    size: MechanismSize,
    mechanism: Mechanism,
}

impl DragHandle {
    /// Forge a standard-size rigid bail handle.
    ///
    /// The folding degree of freedom is opt-in through
    /// [`DragHandle::folding_bail`].
    pub const fn new() -> Self {
        Self::rigid_bail()
    }

    /// Forge a square bail whose hinge lifts under pointer pressure and seats
    /// on release.
    pub const fn folding_bail() -> Self {
        Self {
            size: MechanismSize::Large,
            mechanism: Mechanism::FoldingBail,
        }
    }

    /// Forge the square bail as one rigid pull with no hinge degree of freedom.
    pub const fn rigid_bail() -> Self {
        Self {
            size: MechanismSize::Large,
            mechanism: Mechanism::RigidBail,
        }
    }

    /// Forge a rigid friction pad on a half-width compact escutcheon.
    ///
    /// A shallow bevel surrounds raised diagonal knurling and four peened
    /// corner rivets. The pad has no hinge coordinate or relative swept volume.
    ///
    /// Its default footprint is 12×24 points: the same height as a medium
    /// monoglyph, and half its width.
    pub const fn friction_pad() -> Self {
        Self {
            size: MechanismSize::Medium,
            mechanism: Mechanism::FrictionPad,
        }
    }

    /// Select a build-time forged height gauge.
    ///
    /// Bail assemblies remain square. A friction pad's width is exactly half
    /// the selected height.
    pub const fn size(mut self, size: MechanismSize) -> Self {
        self.size = size;
        self
    }

    /// Lay out, actuate, and paint the complete handle assembly.
    ///
    /// Pass the returned response to `water::Surface::drag_handle` during the
    /// same UI pass to couple a folding bail's swept volume into the water
    /// world. Rigid mechanisms correctly report no relative swept volume.
    pub fn show(self, ui: &mut egui::Ui) -> DragHandleResponse {
        match self.mechanism {
            Mechanism::FoldingBail | Mechanism::RigidBail => self.show_bail(ui),
            Mechanism::FrictionPad => self.show_friction_pad(ui),
        }
    }

    fn show_bail(self, ui: &mut egui::Ui) -> DragHandleResponse {
        let folding = self.mechanism == Mechanism::FoldingBail;
        let atlas = self.size.atlas_index();
        let gauge = baked::GAUGES[atlas];
        debug_assert_eq!(gauge.side, self.size.side() as u8);
        let (rect, mut response) =
            ui.allocate_exact_size(Vec2::splat(self.size.side()), Sense::drag());
        let enabled = ui.is_enabled();
        if enabled {
            let cursor = if response.is_pointer_button_down_on() {
                CursorIcon::Grabbing
            } else {
                CursorIcon::Grab
            };
            response = response.on_hover_cursor(cursor);
        }
        response.widget_info(|| WidgetInfo::labeled(WidgetType::Other, enabled, "Drag handle"));

        let motion = if folding {
            bail_motion(ui, &response, enabled)
        } else {
            Motion {
                position: baked::REST,
                travel: 0.0,
            }
        };
        let mut painter = ui.painter().clone();
        if !enabled {
            painter.set_opacity(1.0);
        }
        paint(ui, &painter, rect, atlas, gauge, motion.position, &response);
        super::tension(ui, &response);

        DragHandleResponse {
            wake: if folding {
                DragHandleWake::new(rect, motion.travel, gauge.sweep_per_radian)
            } else {
                None
            },
            response,
            angle: motion.position,
            ports: CouplingPorts::around_with_station_height(
                rect,
                foundry::law::momentary_gauge(gauge.side).socket_half * 2.0,
            ),
        }
    }

    fn show_friction_pad(self, ui: &mut egui::Ui) -> DragHandleResponse {
        let atlas = self.size.atlas_index();
        let gauge = baked::FRICTION_GAUGES[atlas];
        debug_assert_eq!(gauge.side, self.size.side() as u8);
        let desired = Vec2::new(gauge.width, self.size.side());
        let (rect, mut response) = ui.allocate_exact_size(desired, Sense::drag());
        let enabled = ui.is_enabled();
        if enabled {
            let cursor = if response.is_pointer_button_down_on() {
                CursorIcon::Grabbing
            } else {
                CursorIcon::Grab
            };
            response = response.on_hover_cursor(cursor);
        }
        response
            .widget_info(|| WidgetInfo::labeled(WidgetType::Other, enabled, "Friction drag pad"));

        let mut painter = ui.painter().clone();
        if !enabled {
            painter.set_opacity(1.0);
        }
        paint_friction_pad(ui, &painter, rect, atlas, gauge, &response);
        super::tension(ui, &response);

        DragHandleResponse {
            response,
            wake: None,
            angle: 0.0,
            ports: CouplingPorts::around_with_station_height(
                rect,
                foundry::law::momentary_gauge(gauge.side).socket_half * 2.0,
            ),
        }
    }
}

impl Default for DragHandle {
    fn default() -> Self {
        Self::new()
    }
}

fn bail_motion(ui: &egui::Ui, response: &egui::Response, enabled: bool) -> Motion {
    let dt = ui
        .input(|input| input.stable_dt)
        .clamp(1.0 / 240.0, 1.0 / 30.0);
    let struck = enabled
        && response.hovered()
        && ui.input(|input| input.pointer.primary_pressed() && input.pointer.primary_down());
    let held = enabled && response.is_pointer_button_down_on();
    let strike = if struck {
        Some(7.2)
    } else if response.drag_stopped() {
        Some(-3.6)
    } else {
        None
    };
    plunger::motion(
        ui,
        response.id.with("bail"),
        baked::REST,
        if held { baked::LIFT } else { baked::REST },
        strike,
        dt,
        SPRING_LAW,
    )
}

#[must_use = "the response carries both egui drag state and displaced-water volume"]
/// Interaction, hinge pose, and swept-volume geometry from one [`DragHandle`].
pub struct DragHandleResponse {
    response: egui::Response,
    wake: Option<DragHandleWake>,
    angle: f32,
    ports: CouplingPorts,
}

impl DragHandleResponse {
    /// Bail volume swept since the preceding frame, if it moved.
    pub fn wake(&self) -> Option<DragHandleWake> {
        self.wake
    }

    /// Current bail angle away from the faceplate, in radians.
    ///
    /// Rigid bails return their fixed mounting angle; friction pads return
    /// zero because they have no hinge axis.
    pub fn angle(&self) -> f32 {
        self.angle
    }

    /// Attach a tooltip while retaining the mechanism's physical response.
    pub fn on_hover_text(mut self, text: impl Into<egui::WidgetText>) -> Self {
        self.response = self.response.on_hover_text(text);
        self
    }

    /// Discard physical displacement and return the ordinary egui response.
    pub fn into_response(self) -> egui::Response {
        self.response
    }
}

impl Deref for DragHandleResponse {
    type Target = egui::Response;

    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

impl sealed::Sealed for DragHandleResponse {}

impl CouplingTarget for DragHandleResponse {
    fn coupling_ports(&self) -> CouplingPorts {
        self.ports
    }
}

/// Swept volume from a folding bail's angular travel.
#[derive(Clone, Copy, Debug)]
pub struct DragHandleWake {
    rect: Rect,
    angular_travel: f32,
    volume: f32,
}

impl DragHandleWake {
    fn new(rect: Rect, angular_travel: f32, sweep_per_radian: f32) -> Option<Self> {
        (angular_travel.abs() >= 0.0002).then_some(Self {
            rect,
            angular_travel,
            volume: sweep_per_radian * angular_travel.abs(),
        })
    }

    /// Screen-space footprint occupied by the handle assembly.
    pub fn rect(self) -> Rect {
        self.rect
    }

    /// Signed hinge travel in radians. Positive rotates toward the viewer.
    pub fn angular_travel(self) -> f32 {
        self.angular_travel
    }

    /// Absolute first-order volume swept by the round stock, in logical point³.
    pub fn swept_volume(self) -> f32 {
        self.volume
    }
}

fn paint(
    ui: &egui::Ui,
    painter: &egui::Painter,
    rect: Rect,
    atlas: usize,
    gauge: BakedBailGauge,
    angle: f32,
    response: &egui::Response,
) {
    let origin = rect.center();
    let pose = plunger::pose_index(angle, baked::POSE_MIN, baked::POSE_MAX, gauge.poses.len());
    debug_assert!(
        (gauge.poses[pose].angle - angle).abs()
            <= (baked::POSE_MAX - baked::POSE_MIN) / (gauge.poses.len() - 1) as f32
    );
    let rendered = ui.ctx().data_mut(|data| {
        data.get_temp_mut_or_default::<RenderCache>(response.id.with("compiled-bail"))
            .prepare(origin, atlas, pose, gauge)
    });
    let outer = rect.expand(6.0);
    foundry::paint_compiled(painter, outer, &rendered.floor_shadow);
    foundry::paint_compiled(painter, rect, &rendered.plate);
    foundry::paint_compiled(painter, rect, &rendered.static_shadow);
    foundry::paint_compiled(painter, outer, &rendered.hardware);
    foundry::paint_compiled(painter, rect, &rendered.bail_shadow);
    foundry::paint_compiled(painter, outer, &rendered.bail);
    if response.has_focus() {
        let _focus = painter.rect_stroke(
            rect.shrink(0.5),
            1.0,
            Stroke::new(1.0_f32, HOT.gamma_multiply(0.44)),
            egui::StrokeKind::Inside,
        );
    }
}

fn paint_friction_pad(
    ui: &egui::Ui,
    painter: &egui::Painter,
    rect: Rect,
    atlas: usize,
    gauge: BakedFrictionGauge,
    response: &egui::Response,
) {
    let origin = rect.center();
    let rendered = ui.ctx().data_mut(|data| {
        data.get_temp_mut_or_default::<FrictionRenderCache>(
            response.id.with("compiled-friction-pad"),
        )
        .prepare(origin, atlas, gauge)
    });
    let outer = rect.expand(6.0);
    foundry::paint_compiled(painter, outer, &rendered.floor_shadow);
    foundry::paint_compiled(painter, rect, &rendered.plate);
    foundry::paint_compiled(painter, outer, &rendered.static_shadow);
    foundry::paint_compiled(painter, outer, &rendered.hardware);
    if response.has_focus() {
        let _focus = painter.rect_stroke(
            rect.shrink(0.5),
            1.0,
            Stroke::new(1.0_f32, HOT.gamma_multiply(0.44)),
            egui::StrokeKind::Inside,
        );
    }
}

#[derive(Clone)]
struct InstalledPose {
    shadow: Arc<egui::Mesh>,
    bail: Arc<egui::Mesh>,
}

#[derive(Clone, Default)]
struct RenderCache {
    origin: Option<Pos2>,
    atlas: usize,
    floor_shadow: Option<Arc<egui::Mesh>>,
    plate: Option<Arc<egui::Mesh>>,
    static_shadow: Option<Arc<egui::Mesh>>,
    hardware: Option<Arc<egui::Mesh>>,
    poses: HashMap<usize, InstalledPose>,
}

impl RenderCache {
    fn prepare(
        &mut self,
        origin: Pos2,
        atlas: usize,
        pose: usize,
        gauge: BakedBailGauge,
    ) -> Rendered {
        if self.origin != Some(origin) || self.atlas != atlas {
            *self = Self {
                origin: Some(origin),
                atlas,
                ..Self::default()
            };
        }
        let floor_shadow = self
            .floor_shadow
            .get_or_insert_with(|| plunger::instantiate(gauge.floor_shadow, origin))
            .clone();
        let plate = self
            .plate
            .get_or_insert_with(|| plunger::instantiate(gauge.plate, origin))
            .clone();
        let static_shadow = self
            .static_shadow
            .get_or_insert_with(|| plunger::instantiate(gauge.static_shadow, origin))
            .clone();
        let hardware = self
            .hardware
            .get_or_insert_with(|| plunger::instantiate(gauge.hardware, origin))
            .clone();
        let installed = self.poses.entry(pose).or_insert_with(|| {
            let pose = gauge.poses[pose];
            InstalledPose {
                shadow: plunger::instantiate(pose.shadow, origin),
                bail: plunger::instantiate(pose.bail, origin),
            }
        });
        Rendered {
            floor_shadow,
            plate,
            static_shadow,
            hardware,
            bail_shadow: installed.shadow.clone(),
            bail: installed.bail.clone(),
        }
    }
}

struct Rendered {
    floor_shadow: Arc<egui::Mesh>,
    plate: Arc<egui::Mesh>,
    static_shadow: Arc<egui::Mesh>,
    hardware: Arc<egui::Mesh>,
    bail_shadow: Arc<egui::Mesh>,
    bail: Arc<egui::Mesh>,
}

#[derive(Clone, Default)]
struct FrictionRenderCache {
    origin: Option<Pos2>,
    atlas: usize,
    floor_shadow: Option<Arc<egui::Mesh>>,
    plate: Option<Arc<egui::Mesh>>,
    static_shadow: Option<Arc<egui::Mesh>>,
    hardware: Option<Arc<egui::Mesh>>,
}

impl FrictionRenderCache {
    fn prepare(
        &mut self,
        origin: Pos2,
        atlas: usize,
        gauge: BakedFrictionGauge,
    ) -> FrictionRendered {
        if self.origin != Some(origin) || self.atlas != atlas {
            *self = Self {
                origin: Some(origin),
                atlas,
                ..Self::default()
            };
        }
        FrictionRendered {
            floor_shadow: self
                .floor_shadow
                .get_or_insert_with(|| plunger::instantiate(gauge.floor_shadow, origin))
                .clone(),
            plate: self
                .plate
                .get_or_insert_with(|| plunger::instantiate(gauge.plate, origin))
                .clone(),
            static_shadow: self
                .static_shadow
                .get_or_insert_with(|| plunger::instantiate(gauge.static_shadow, origin))
                .clone(),
            hardware: self
                .hardware
                .get_or_insert_with(|| plunger::instantiate(gauge.hardware, origin))
                .clone(),
        }
    }
}

struct FrictionRendered {
    floor_shadow: Arc<egui::Mesh>,
    plate: Arc<egui::Mesh>,
    static_shadow: Arc<egui::Mesh>,
    hardware: Arc<egui::Mesh>,
}
