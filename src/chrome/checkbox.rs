//! A latching square plunger under a sprung protective grille. The plunger's
//! two boolean states are literal z-stops: unchecked stands proud, checked
//! seats down in its aperture. Pointer pressure drives it below either latch;
//! release changes the latch and a stiff underdamped spring closes the motion.
//!
//! Disabled controls do not change their state indication. A welded-wire cage
//! occupies the hand volume above the mechanism while leaving its elevation
//! visible. Crown, skirt, wire, welds, and frame are physical triangle meshes;
//! the build-time foundry compiler performs projection, visibility,
//! illumination, and directional shadow casting once, then runtime replays its
//! 2D vector pose atlas.
//!
//! Optional descriptions inhabit a casing-height bronze plaque with 45° edge
//! facets and two cylindrical ties. Its parameterized plate geometry and
//! dynamic flat-bottomed text cut are projected under the same camera and
//! illuminant.

#![deny(missing_docs)]

use std::{collections::HashMap, ops::Deref, sync::Arc};

use egui::{
    CursorIcon, Pos2, Rect, Sense, Stroke, TextStyle, TextWrapMode, Vec2, WidgetInfo, WidgetText,
    WidgetType,
};

use super::{COUPLING_SPACING, HOT, foundry};

use super::mechanism::{CouplingPorts, CouplingTarget, MechanismSize, sealed};
use super::plunger::{
    self, BakedMesh, BakedPose, BakedShadow, BakedVertex, PlungerWake, SpringLaw,
};

#[derive(Clone, Copy)]
struct BakedCheckboxGauge {
    side: u8,
    control_height: f32,
    assembly_side: f32,
    socket_half: f32,
    body_half: f32,
    latch_up: f32,
    latch_down: f32,
    pose_min: f32,
    pose_max: f32,
    wire_count: u8,
    guard: BakedMesh,
    guard_floor_shadow: BakedMesh,
    guard_crown_shadow: BakedShadow,
    poses: &'static [BakedPose],
}

fn spring_law(gauge: BakedCheckboxGauge) -> SpringLaw {
    SpringLaw {
        stiffness: 1_700.0,
        damping: 42.0,
        restitution: 0.16,
        floor: gauge.pose_min,
        ceiling: gauge.pose_max,
    }
}

mod baked {
    use super::{BakedCheckboxGauge, BakedMesh, BakedPose, BakedShadow, BakedVertex};

    include!(concat!(env!("OUT_DIR"), "/checkbox_atlas.rs"));
}

/// A mechanically latching Poolrooms boolean control.
///
/// The unchecked crown stands proud of its aperture; the checked crown rests
/// on the lower latch. Pressing drives either state toward a shared overtravel
/// stop, and release excites the spring around the newly selected latch.
/// A nonempty label is cut into a casing-height bronze plaque joined to the
/// mechanism by two cylindrical ties. [`Checkbox::label_side`] places that
/// plaque on either side. Disabling the surrounding `egui::Ui` installs the
/// physical wire guard while preserving the state geometry and foundry
/// luminance beneath it; the guard, rather than egui's conventional opacity
/// fade, is the disabled affordance. [`Checkbox::size`] selects an independent
/// build-time forge. Compact guards retain the large guard's wire, frame, and
/// weld stock, removing lattice lines instead of shrinking them into
/// alias-prone filaments.
///
/// # Example
///
/// ```
/// use dwemer_poolrooms::{chrome::{Checkbox, LabelSide}, egui};
///
/// fn controls(ui: &mut egui::Ui, armed: &mut bool) {
///     let checkbox = Checkbox::new(armed, "ARM PUMPS")
///         .label_side(LabelSide::Left)
///         .show(ui);
///     if checkbox.changed() {
///         // `armed` changed latch.
///     }
/// }
/// ```
pub struct Checkbox<'a> {
    checked: &'a mut bool,
    label: Option<WidgetText>,
    label_side: LabelSide,
    size: MechanismSize,
}

/// Side of a mechanism occupied by its etched identification plaque.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LabelSide {
    /// Place the plaque to the left of the mechanism.
    Left,
    /// Place the plaque to the right of the mechanism.
    #[default]
    Right,
}

impl<'a> Checkbox<'a> {
    /// Construct a latching plunger with a right-hand etched plaque.
    ///
    /// An empty label elides the plaque and its ties entirely.
    pub fn new(checked: &'a mut bool, label: impl Into<WidgetText>) -> Self {
        Self {
            checked,
            label: Some(label.into()),
            label_side: LabelSide::Right,
            size: MechanismSize::Large,
        }
    }

    /// Construct an unlabelled latching plunger.
    pub fn without_text(checked: &'a mut bool) -> Self {
        Self {
            checked,
            label: None,
            label_side: LabelSide::Right,
            size: MechanismSize::Large,
        }
    }

    /// Place the etched plaque to the left or right of the plunger casing.
    ///
    /// This has no visible effect on an unlabelled checkbox.
    pub fn label_side(mut self, side: LabelSide) -> Self {
        self.label_side = side;
        self
    }

    /// Select a build-time forged plunger and protective-guard gauge.
    ///
    /// The nominal 20-, 24-, or 32-point gauge governs the plunger. Its
    /// protective guard requires a proportionally larger allocation; fixed
    /// wire stock and progressively coarser lattices keep every size crisp.
    pub const fn size(mut self, size: MechanismSize) -> Self {
        self.size = size;
        self
    }

    /// Lay out, interact with, and paint the complete mechanism.
    ///
    /// The response dereferences to `egui::Response` and carries the signed
    /// volume swept by the plunger during this frame. Pass it to
    /// `water::Surface::checkbox` during the same UI pass to couple that motion
    /// into the active water world.
    pub fn show(self, ui: &mut egui::Ui) -> CheckboxResponse {
        let Self {
            checked,
            label,
            label_side,
            size,
        } = self;
        let atlas = size.atlas_index();
        let gauge = baked::GAUGES[atlas];
        debug_assert_eq!(gauge.side, size.side() as u8);
        debug_assert_eq!(usize::from(gauge.wire_count), atlas + 2);
        let label_text = label.as_ref().map_or("", WidgetText::text).to_owned();
        let plaque = label.and_then(|label| {
            let galley = label.into_galley(
                ui,
                Some(TextWrapMode::Extend),
                f32::INFINITY,
                TextStyle::Button,
            );
            (!galley.is_empty()).then(|| foundry::Plaque::new(galley, gauge.socket_half * 2.0))
        });
        let desired = footprint(gauge, plaque.as_ref().map(foundry::Plaque::size));
        let (rect, mut response) = ui.allocate_exact_size(desired, Sense::click());
        let enabled = ui.is_enabled();
        if enabled {
            response = response.on_hover_cursor(CursorIcon::PointingHand);
        }
        let activated = super::exact_activation(ui, &response);
        if activated {
            *checked = !*checked;
            response.mark_changed();
        }
        response.widget_info(|| {
            WidgetInfo::selected(WidgetType::Checkbox, enabled, *checked, label_text.clone())
        });

        let anatomy = Anatomy::new(
            rect,
            label_side,
            plaque.as_ref().map(foundry::Plaque::size),
            gauge,
        );
        let held = enabled && response.is_pointer_button_down_on();
        let target = if held {
            gauge.pose_min
        } else if *checked {
            gauge.latch_down
        } else {
            gauge.latch_up
        };
        let dt = ui
            .input(|input| input.stable_dt)
            .clamp(1.0 / 240.0, 1.0 / 30.0);
        let seed = if *checked {
            gauge.latch_down
        } else {
            gauge.latch_up
        };
        let scale = f32::from(gauge.side) / MechanismSize::Large.side();
        let motion = plunger::motion(
            ui,
            response.id,
            seed,
            target,
            activated.then_some(-32.0 * scale),
            dt,
            spring_law(gauge),
        );
        let mut painter = ui.painter().clone();
        if !enabled {
            // The grille is the disabled affordance. Egui's inherited opacity
            // would counterfeit a second, nonphysical state change beneath it.
            painter.set_opacity(1.0);
        }
        paint(
            ui,
            &painter,
            anatomy,
            plaque.as_ref(),
            motion.position,
            enabled,
            &response,
            atlas,
            gauge,
        );
        let wake = CheckboxWake::new(anatomy.button, motion.travel);
        CheckboxResponse {
            response,
            wake,
            elevation: motion.position,
            ports: anatomy.coupling_ports(),
            activated,
        }
    }
}

fn footprint(gauge: BakedCheckboxGauge, plaque_size: Option<Vec2>) -> Vec2 {
    plaque_size.map_or(
        Vec2::new(gauge.assembly_side, gauge.control_height),
        |plaque| {
            Vec2::new(
                gauge.assembly_side * 0.5 + gauge.socket_half + COUPLING_SPACING + plaque.x,
                gauge.control_height.max(plaque.y),
            )
        },
    )
}

#[must_use = "the response carries both egui state and displaced-water volume"]
/// Interaction state and displaced-water geometry from one [`Checkbox`] frame.
pub struct CheckboxResponse {
    response: egui::Response,
    wake: Option<CheckboxWake>,
    elevation: f32,
    ports: CouplingPorts,
    activated: bool,
}

impl CheckboxResponse {
    /// Whether pointer, accessibility, or exact keyboard activation toggled it.
    pub const fn clicked(&self) -> bool {
        self.activated
    }

    /// The plunger volume swept since the preceding frame, if it moved.
    pub fn wake(&self) -> Option<CheckboxWake> {
        self.wake
    }

    /// Current crown elevation normal to the faceplate, in logical points.
    /// Positive values stand toward the viewer; negative values lie within
    /// the recess.
    pub fn elevation(&self) -> f32 {
        self.elevation
    }

    /// Discard physical displacement and return the ordinary egui response.
    pub fn into_response(self) -> egui::Response {
        self.response
    }
}

impl Deref for CheckboxResponse {
    type Target = egui::Response;

    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

impl sealed::Sealed for CheckboxResponse {}

impl CouplingTarget for CheckboxResponse {
    fn coupling_ports(&self) -> CouplingPorts {
        self.ports
    }
}

/// Signed swept volume from the checkbox plunger.
pub type CheckboxWake = PlungerWake;

#[derive(Clone, Copy)]
struct Anatomy {
    assembly: Rect,
    socket: Rect,
    button: Rect,
    plaque: Option<Rect>,
}

impl Anatomy {
    fn new(
        rect: Rect,
        side: LabelSide,
        plaque_size: Option<Vec2>,
        gauge: BakedCheckboxGauge,
    ) -> Self {
        let assembly_x = match (side, plaque_size) {
            (_, None) => rect.center().x,
            (LabelSide::Left, Some(_)) => rect.right() - gauge.assembly_side * 0.5,
            (LabelSide::Right, Some(_)) => rect.left() + gauge.assembly_side * 0.5,
        };
        let assembly = Rect::from_center_size(
            Pos2::new(assembly_x, rect.center().y),
            Vec2::splat(gauge.assembly_side),
        );
        let socket =
            Rect::from_center_size(assembly.center(), Vec2::splat(gauge.socket_half * 2.0));
        let button = Rect::from_center_size(assembly.center(), Vec2::splat(gauge.body_half * 2.0));
        let plaque = plaque_size.map(|size| {
            let x = match side {
                LabelSide::Left => socket.left() - COUPLING_SPACING - size.x * 0.5,
                LabelSide::Right => socket.right() + COUPLING_SPACING + size.x * 0.5,
            };
            Rect::from_center_size(Pos2::new(x, rect.center().y), size)
        });
        Self {
            assembly,
            socket,
            button,
            plaque,
        }
    }

    fn coupling_ports(self) -> CouplingPorts {
        match self.plaque {
            Some(plaque) if plaque.center().x < self.socket.center().x => {
                CouplingPorts::spanning(plaque, self.socket)
            }
            Some(plaque) => CouplingPorts::spanning(self.socket, plaque),
            None => CouplingPorts::around(self.socket),
        }
    }
}

fn paint(
    ui: &egui::Ui,
    painter: &egui::Painter,
    anatomy: Anatomy,
    plaque: Option<&foundry::Plaque>,
    elevation: f32,
    enabled: bool,
    response: &egui::Response,
    atlas: usize,
    gauge: BakedCheckboxGauge,
) {
    let origin = anatomy.socket.center();
    let clip = anatomy.assembly.expand(2.0);
    let pose = plunger::pose_index(elevation, gauge.pose_min, gauge.pose_max, baked::POSE_COUNT);
    let rendered = ui.ctx().data_mut(|data| {
        data.get_temp_mut_or_default::<RenderCache>(response.id.with("compiled-foundry"))
            .prepare(origin, atlas, gauge, pose, !enabled)
    });

    if let Some(plaque_rect) = anatomy.plaque {
        let ports = if plaque_rect.center().x < anatomy.socket.center().x {
            (
                CouplingPorts::around(plaque_rect),
                CouplingPorts::around(anatomy.socket),
            )
        } else {
            (
                CouplingPorts::around(anatomy.socket),
                CouplingPorts::around(plaque_rect),
            )
        };
        let _ties = painter.add(foundry::tie_pair(ports.0.right, ports.1.left));
    }
    foundry::socket_bed(painter, anatomy.socket);
    if let Some(shadow) = &rendered.guard_floor_shadow {
        foundry::paint_compiled(painter, clip, shadow);
    }
    foundry::paint_compiled(painter, anatomy.socket.shrink(1.0), &rendered.button_shadow);
    foundry::paint_compiled(
        painter,
        anatomy.socket.shrink(foundry::RIM_WIDTH),
        &rendered.button,
    );
    foundry::socket_rim(painter, anatomy.socket);

    if let (Some(shadow), Some(guard)) = (&rendered.guard_crown_shadow, &rendered.guard) {
        let crown_clip = Rect::from_center_size(origin, Vec2::splat(gauge.body_half * 2.5));
        foundry::paint_compiled(painter, crown_clip, shadow);
        foundry::paint_compiled(painter, clip, guard);
    }
    if let (Some(plaque), Some(rect)) = (plaque, anatomy.plaque) {
        plaque.paint(painter, rect.center());
    }
    if response.has_focus() {
        let _focus = painter.rect_stroke(
            anatomy.assembly.shrink(0.5),
            1.0,
            Stroke::new(1.0_f32, HOT.gamma_multiply(0.44)),
            egui::StrokeKind::Inside,
        );
    }
}

#[derive(Clone)]
struct InstalledPose {
    button: Arc<egui::Mesh>,
    button_shadow: Arc<egui::Mesh>,
    guard_crown_shadow: Option<Arc<egui::Mesh>>,
}

#[derive(Clone, Default)]
struct RenderCache {
    origin: Option<Pos2>,
    atlas: Option<usize>,
    poses: HashMap<usize, InstalledPose>,
    guard: Option<Arc<egui::Mesh>>,
    guard_floor_shadow: Option<Arc<egui::Mesh>>,
}

impl RenderCache {
    fn prepare(
        &mut self,
        origin: Pos2,
        atlas: usize,
        gauge: BakedCheckboxGauge,
        pose_index: usize,
        guarded: bool,
    ) -> Rendered {
        if self.origin != Some(origin) || self.atlas != Some(atlas) {
            *self = Self {
                origin: Some(origin),
                atlas: Some(atlas),
                ..Self::default()
            };
        }
        if guarded && self.guard.is_none() {
            self.guard = Some(plunger::instantiate(gauge.guard, origin));
            self.guard_floor_shadow = Some(plunger::instantiate(gauge.guard_floor_shadow, origin));
        }
        let pose = gauge.poses[pose_index];
        let installed = self
            .poses
            .entry(pose_index)
            .or_insert_with(|| InstalledPose {
                button: plunger::instantiate(pose.button, origin),
                button_shadow: plunger::instantiate(pose.shadow, origin),
                guard_crown_shadow: None,
            });
        if guarded && installed.guard_crown_shadow.is_none() {
            installed.guard_crown_shadow = Some(plunger::instantiate_shadow(
                gauge.guard_crown_shadow,
                origin,
                pose.elevation,
                baked::SHADOW_EYE_Z,
                baked::SHADOW_SLOPE,
            ));
        }
        Rendered {
            button: installed.button.clone(),
            button_shadow: installed.button_shadow.clone(),
            guard_crown_shadow: installed.guard_crown_shadow.clone(),
            guard: self.guard.clone(),
            guard_floor_shadow: self.guard_floor_shadow.clone(),
        }
    }
}

struct Rendered {
    button: Arc<egui::Mesh>,
    button_shadow: Arc<egui::Mesh>,
    guard_crown_shadow: Option<Arc<egui::Mesh>>,
    guard: Option<Arc<egui::Mesh>>,
    guard_floor_shadow: Option<Arc<egui::Mesh>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spring_converges_on_both_latches_after_contact() {
        for gauge in baked::GAUGES {
            for (from, target) in [
                (gauge.latch_up, gauge.latch_down),
                (gauge.latch_down, gauge.latch_up),
            ] {
                let mut spring = plunger::Spring::at(from);
                for _ in 0..180 {
                    spring.advance(target, 1.0 / 120.0, spring_law(gauge));
                }
                assert!((spring.position() - target).abs() < 0.002);
                assert!(spring.velocity().abs() < 0.02);
            }
        }
    }

    #[test]
    fn stiff_spring_crosses_and_recoils_within_one_eighth_second() {
        for gauge in baked::GAUGES {
            let mut spring = plunger::Spring::at(gauge.pose_min);
            let mut first_crossing = None;
            let mut recoil = false;
            for step in 1..=60 {
                spring.advance(gauge.latch_down, 1.0 / 240.0, spring_law(gauge));
                if spring.position() > gauge.latch_down {
                    let _crossing = first_crossing.get_or_insert(step);
                } else if first_crossing.is_some() {
                    recoil = true;
                    break;
                }
            }
            assert!(first_crossing.is_some_and(|step| step <= 30));
            assert!(recoil);
        }
    }

    #[test]
    fn build_time_atlas_covers_the_complete_travel() {
        assert_eq!(baked::GAUGES.len(), baked::GAUGE_COUNT);
        for (atlas, gauge) in baked::GAUGES.into_iter().enumerate() {
            assert_eq!(gauge.poses.len(), baked::POSE_COUNT);
            assert_eq!(gauge.poses[0].elevation, gauge.pose_min);
            assert_eq!(gauge.poses[baked::POSE_COUNT - 1].elevation, gauge.pose_max);
            assert!(
                gauge
                    .poses
                    .windows(2)
                    .all(|poses| poses[0].elevation < poses[1].elevation)
            );
            assert!(!gauge.guard.vertices.is_empty());
            assert!(!gauge.guard_crown_shadow.mesh.vertices.is_empty());
            assert_eq!(usize::from(gauge.wire_count), atlas + 2);
        }
    }

    #[test]
    fn perspective_separates_the_two_latched_silhouettes() {
        let diameter = |mesh: BakedMesh| {
            let (lo, hi) = mesh
                .vertices
                .iter()
                .map(|vertex| vertex.position[0])
                .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), x| {
                    (lo.min(x), hi.max(x))
                });
            hi - lo
        };
        let pose = |gauge: BakedCheckboxGauge, elevation| {
            plunger::pose_index(elevation, gauge.pose_min, gauge.pose_max, baked::POSE_COUNT)
        };
        for gauge in baked::GAUGES {
            let raised = diameter(gauge.poses[pose(gauge, gauge.latch_up)].button);
            let recessed = diameter(gauge.poses[pose(gauge, gauge.latch_down)].button);
            let minimum = 4.0 * f32::from(gauge.side) / MechanismSize::Large.side();
            assert!(
                raised - recessed > minimum,
                "gauge {} latched silhouettes differ by only {} points",
                gauge.side,
                raised - recessed
            );
        }
    }

    #[test]
    fn latch_stroke_and_swept_volume_share_logical_point_units() {
        let large = baked::GAUGES[MechanismSize::Large.atlas_index()];
        let stroke = large.latch_up - large.latch_down;
        let area = (large.body_half * 2.0).powi(2);
        assert!((stroke - 31.05).abs() < 0.001);
        assert!((area * stroke - 16_141.0).abs() < 1.0);
        for gauge in baked::GAUGES {
            let scale = f32::from(gauge.side) / f32::from(large.side);
            let volume = (gauge.body_half * 2.0).powi(2) * (gauge.latch_up - gauge.latch_down);
            assert!((volume / (area * stroke) - scale.powi(3)).abs() < 1e-5);
        }
    }

    #[test]
    fn every_public_gauge_has_exact_geometry_and_layout() {
        for size in MechanismSize::ALL {
            let gauge = baked::GAUGES[size.atlas_index()];
            assert_eq!(gauge.side, size.side() as u8);
            let ctx = egui::Context::default();
            let mut actual = Vec2::ZERO;
            let mut checked = false;
            let _frame = ctx.run_ui(egui::RawInput::default(), |ui| {
                actual = Checkbox::without_text(&mut checked)
                    .size(size)
                    .show(ui)
                    .rect
                    .size();
            });
            assert_eq!(actual, Vec2::new(gauge.assembly_side, gauge.control_height));
        }
    }

    #[test]
    fn small_pointer_press_changes_latch_and_reports_swept_volume() {
        let ctx = egui::Context::default();
        let gauge = baked::GAUGES[MechanismSize::Small.atlas_index()];
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(64.0, gauge.control_height));
        let mut checked = false;
        let mut swept = 0.0;
        let mut center = Pos2::ZERO;
        let _prime = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(screen),
                ..egui::RawInput::default()
            },
            |ui| {
                let checkbox = Checkbox::without_text(&mut checked)
                    .size(MechanismSize::Small)
                    .show(ui);
                center = checkbox.rect.center();
            },
        );
        for pressed in [true, false] {
            let _frame = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(screen),
                    events: vec![
                        egui::Event::PointerMoved(center),
                        egui::Event::PointerButton {
                            pos: center,
                            button: egui::PointerButton::Primary,
                            pressed,
                            modifiers: egui::Modifiers::NONE,
                        },
                    ],
                    ..egui::RawInput::default()
                },
                |ui| {
                    let checkbox = Checkbox::without_text(&mut checked)
                        .size(MechanismSize::Small)
                        .show(ui);
                    swept += checkbox.wake().map_or(0.0, CheckboxWake::swept_volume);
                },
            );
        }
        assert!(checked);
        assert!(swept > 120.0, "small plunger swept only {swept} point³");
    }

    #[test]
    fn plaque_height_gap_and_side_are_exact() {
        for gauge in baked::GAUGES {
            let plaque_size = Vec2::new(92.0, gauge.socket_half * 2.0);
            let rect = Rect::from_min_size(Pos2::ZERO, footprint(gauge, Some(plaque_size)));
            for side in [LabelSide::Left, LabelSide::Right] {
                let anatomy = Anatomy::new(rect, side, Some(plaque_size), gauge);
                let plaque = anatomy.plaque.expect("labelled geometry has a plaque");
                assert_eq!(plaque.height(), anatomy.socket.height());
                let gap = match side {
                    LabelSide::Left => anatomy.socket.left() - plaque.right(),
                    LabelSide::Right => plaque.left() - anatomy.socket.right(),
                };
                assert!((gap - COUPLING_SPACING).abs() < 1e-4);
            }
        }
    }

    #[test]
    fn plaque_is_part_of_the_toggle_hit_target() {
        let ctx = egui::Context::default();
        let gauge = baked::GAUGES[MechanismSize::Large.atlas_index()];
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(240.0, gauge.control_height));
        let mut checked = false;
        let mut plaque_point = Pos2::ZERO;
        let _prime = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(screen),
                ..egui::RawInput::default()
            },
            |ui| {
                let checkbox = Checkbox::new(&mut checked, "ARM PUMPS").show(ui);
                plaque_point = Pos2::new(checkbox.rect.right() - 4.0, checkbox.rect.center().y);
            },
        );
        for pressed in [true, false] {
            let _frame = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(screen),
                    events: vec![
                        egui::Event::PointerMoved(plaque_point),
                        egui::Event::PointerButton {
                            pos: plaque_point,
                            button: egui::PointerButton::Primary,
                            pressed,
                            modifiers: egui::Modifiers::NONE,
                        },
                    ],
                    ..egui::RawInput::default()
                },
                |ui| {
                    let _checkbox = Checkbox::new(&mut checked, "ARM PUMPS").show(ui);
                },
            );
        }
        assert!(checked);
    }
}
