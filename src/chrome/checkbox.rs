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

#![deny(missing_docs)]

use std::{collections::HashMap, ops::Deref, sync::Arc};

use egui::{
    Color32, CursorIcon, Pos2, Rect, Sense, Stroke, TextStyle, TextWrapMode, Vec2, WidgetInfo,
    WidgetText, WidgetType,
};

use super::{HOT, TEXT, foundry};

const CONTROL_H: f32 = 42.0;
const ASSEMBLY_W: f32 = 38.0;
const SOCKET_HALF: f32 = 14.8;
const SPRING_K: f32 = 1_700.0;
const SPRING_C: f32 = 42.0;
const CONTACT_RESTITUTION: f32 = 0.16;
const INTEGRATOR_STEP: f32 = 1.0 / 240.0;

#[derive(Clone, Copy)]
struct BakedVertex {
    position: [f32; 2],
    color: [u8; 4],
}

#[derive(Clone, Copy)]
struct BakedMesh {
    vertices: &'static [BakedVertex],
    indices: &'static [u32],
}

#[derive(Clone, Copy)]
struct BakedShadow {
    mesh: BakedMesh,
}

#[derive(Clone, Copy)]
struct BakedPose {
    elevation: f32,
    button: BakedMesh,
    shadow: BakedMesh,
}

mod baked {
    use super::{BakedMesh, BakedPose, BakedShadow, BakedVertex};

    include!(concat!(env!("OUT_DIR"), "/checkbox_atlas.rs"));
}

/// A mechanically latching Poolrooms boolean control.
///
/// The unchecked crown stands proud of its aperture; the checked crown rests
/// on the lower latch. Pressing drives either state toward a shared overtravel
/// stop, and release excites the spring around the newly selected latch.
/// Disabling the surrounding `egui::Ui` installs the physical wire guard while
/// preserving the state geometry and foundry luminance beneath it; the guard,
/// rather than egui's conventional opacity fade, is the disabled affordance.
///
/// # Example
///
/// ```
/// use dwemer_poolrooms::{chrome::Checkbox, egui};
///
/// fn controls(ui: &mut egui::Ui, armed: &mut bool) {
///     let checkbox = Checkbox::new(armed, "ARM PUMPS").show(ui);
///     if checkbox.changed() {
///         // `armed` changed latch.
///     }
/// }
/// ```
pub struct Checkbox<'a> {
    checked: &'a mut bool,
    label: WidgetText,
}

impl<'a> Checkbox<'a> {
    /// Construct a labelled latching plunger.
    pub fn new(checked: &'a mut bool, label: impl Into<WidgetText>) -> Self {
        Self {
            checked,
            label: label.into(),
        }
    }

    /// Construct an unlabelled latching plunger.
    pub fn without_text(checked: &'a mut bool) -> Self {
        Self::new(checked, WidgetText::default())
    }

    /// Lay out, interact with, and paint the complete mechanism.
    ///
    /// The response dereferences to `egui::Response` and carries the signed
    /// volume swept by the plunger during this frame. Pass it to
    /// `water::Surface::checkbox` during the same UI pass to couple that motion
    /// into the active water world.
    pub fn show(self, ui: &mut egui::Ui) -> CheckboxResponse {
        let Self { checked, label } = self;
        let label_text = label.text().to_owned();
        let galley = label.into_galley(
            ui,
            Some(TextWrapMode::Extend),
            f32::INFINITY,
            TextStyle::Button,
        );
        let gap = if galley.is_empty() {
            0.0
        } else {
            ui.spacing().item_spacing.x
        };
        let desired = Vec2::new(
            ASSEMBLY_W + gap + galley.size().x,
            CONTROL_H.max(galley.size().y),
        );
        let (rect, mut response) = ui.allocate_exact_size(desired, Sense::click());
        let enabled = ui.is_enabled();
        if enabled {
            response = response.on_hover_cursor(CursorIcon::PointingHand);
        }
        if response.clicked() {
            *checked = !*checked;
            response.mark_changed();
        }
        response.widget_info(|| {
            WidgetInfo::selected(WidgetType::Checkbox, enabled, *checked, label_text.clone())
        });

        let anatomy = Anatomy::new(rect);
        let held = enabled && response.is_pointer_button_down_on();
        let target = if held {
            baked::POSE_MIN
        } else if *checked {
            baked::LATCH_DOWN
        } else {
            baked::LATCH_UP
        };
        let dt = ui
            .input(|input| input.stable_dt)
            .clamp(INTEGRATOR_STEP, 1.0 / 30.0);
        let motion = plunger_motion(ui, response.id, target, held, response.clicked(), dt);
        let mut painter = ui.painter().clone();
        if !enabled {
            // The grille is the disabled affordance. Egui's inherited opacity
            // would counterfeit a second, nonphysical state change beneath it.
            painter.set_opacity(1.0);
        }
        paint(ui, &painter, anatomy, motion.elevation, enabled, &response);

        if !galley.is_empty() {
            let label_pos = Pos2::new(
                anatomy.assembly.right() + gap,
                rect.center().y - galley.size().y * 0.5,
            );
            painter.galley(label_pos, galley, TEXT);
        }
        let wake = CheckboxWake::new(anatomy.button, motion.travel);
        CheckboxResponse {
            response,
            wake,
            elevation: motion.elevation,
        }
    }
}

#[must_use = "the response carries both egui state and displaced-water volume"]
/// Interaction state and displaced-water geometry from one [`Checkbox`] frame.
pub struct CheckboxResponse {
    response: egui::Response,
    wake: Option<CheckboxWake>,
    elevation: f32,
}

impl CheckboxResponse {
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

/// Signed swept volume from the checkbox plunger.
#[derive(Clone, Copy, Debug)]
pub struct CheckboxWake {
    rect: Rect,
    travel: f32,
    volume: f32,
}

impl CheckboxWake {
    fn new(rect: Rect, travel: f32) -> Option<Self> {
        (travel.abs() >= 0.002).then_some(Self {
            rect,
            travel,
            volume: rect.area() * travel.abs(),
        })
    }

    /// Screen-space footprint occupied by the moving plunger.
    pub fn rect(self) -> Rect {
        self.rect
    }

    /// Signed travel normal to the faceplate. Positive is toward the viewer.
    pub fn travel(self) -> f32 {
        self.travel
    }

    /// Absolute swept volume in logical point³.
    pub fn swept_volume(self) -> f32 {
        self.volume
    }
}

#[derive(Clone, Copy)]
struct Anatomy {
    assembly: Rect,
    socket: Rect,
    button: Rect,
}

impl Anatomy {
    fn new(rect: Rect) -> Self {
        let assembly = Rect::from_center_size(
            Pos2::new(rect.left() + ASSEMBLY_W * 0.5, rect.center().y),
            Vec2::splat(ASSEMBLY_W),
        );
        let socket = Rect::from_center_size(assembly.center(), Vec2::splat(SOCKET_HALF * 2.0));
        let button = Rect::from_center_size(assembly.center(), Vec2::splat(baked::BODY_HALF * 2.0));
        Self {
            assembly,
            socket,
            button,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Spring {
    elevation: f32,
    velocity: f32,
    held: bool,
}

impl Spring {
    fn at(elevation: f32) -> Self {
        Self {
            elevation,
            velocity: 0.0,
            held: false,
        }
    }

    fn advance(&mut self, target: f32, dt: f32) {
        let steps = (dt / INTEGRATOR_STEP).ceil() as u32;
        let h = dt / steps.max(1) as f32;
        for _ in 0..steps {
            self.velocity += (-SPRING_K * (self.elevation - target) - SPRING_C * self.velocity) * h;
            self.elevation += self.velocity * h;
            if self.elevation < baked::POSE_MIN {
                self.elevation = baked::POSE_MIN;
                self.velocity = self.velocity.abs() * CONTACT_RESTITUTION;
            } else if self.elevation > baked::POSE_MAX {
                self.elevation = baked::POSE_MAX;
                self.velocity = -self.velocity.abs() * CONTACT_RESTITUTION;
            }
        }
    }

    fn moving(self, target: f32) -> bool {
        (self.elevation - target).abs() > 0.001 || self.velocity.abs() > 0.01
    }
}

#[derive(Clone, Copy)]
struct Motion {
    elevation: f32,
    travel: f32,
}

fn plunger_motion(
    ui: &egui::Ui,
    id: egui::Id,
    target: f32,
    held: bool,
    clicked: bool,
    dt: f32,
) -> Motion {
    let key = id.with("plunger-spring");
    let (motion, moving) = ui.ctx().data_mut(|data| {
        let mut spring = data
            .get_temp::<Spring>(key)
            .unwrap_or_else(|| Spring::at(target));
        let before = spring.elevation;
        if clicked && !spring.held {
            spring.velocity = spring.velocity.min(-32.0);
        }
        spring.held = held;
        spring.advance(target, dt);
        let moving = spring.moving(target);
        let motion = Motion {
            elevation: spring.elevation,
            travel: spring.elevation - before,
        };
        let _old = data.insert_temp(key, spring);
        (motion, moving)
    });
    if moving {
        ui.ctx().request_repaint();
    }
    motion
}

fn paint(
    ui: &egui::Ui,
    painter: &egui::Painter,
    anatomy: Anatomy,
    elevation: f32,
    enabled: bool,
    response: &egui::Response,
) {
    let origin = anatomy.socket.center();
    let clip = anatomy.assembly.expand(2.0);
    let pose = pose_index(elevation);
    let rendered = ui.ctx().data_mut(|data| {
        data.get_temp_mut_or_default::<RenderCache>(response.id.with("compiled-foundry"))
            .prepare(origin, pose, !enabled)
    });

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
        let crown_clip = Rect::from_center_size(origin, Vec2::splat(baked::BODY_HALF * 2.5));
        foundry::paint_compiled(painter, crown_clip, shadow);
        foundry::paint_compiled(painter, clip, guard);
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

fn pose_index(elevation: f32) -> usize {
    let t = ((elevation - baked::POSE_MIN) / (baked::POSE_MAX - baked::POSE_MIN)).clamp(0.0, 1.0);
    ((t * (baked::POSE_COUNT - 1) as f32).round() as usize).min(baked::POSE_COUNT - 1)
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
    poses: HashMap<usize, InstalledPose>,
    guard: Option<Arc<egui::Mesh>>,
    guard_floor_shadow: Option<Arc<egui::Mesh>>,
}

impl RenderCache {
    fn prepare(&mut self, origin: Pos2, pose_index: usize, guarded: bool) -> Rendered {
        if self.origin != Some(origin) {
            *self = Self {
                origin: Some(origin),
                ..Self::default()
            };
        }
        if guarded && self.guard.is_none() {
            self.guard = Some(instantiate(baked::GUARD, origin));
            self.guard_floor_shadow = Some(instantiate(baked::GUARD_FLOOR_SHADOW, origin));
        }
        let pose = baked::POSES[pose_index];
        let installed = self
            .poses
            .entry(pose_index)
            .or_insert_with(|| InstalledPose {
                button: instantiate(pose.button, origin),
                button_shadow: instantiate(pose.shadow, origin),
                guard_crown_shadow: None,
            });
        if guarded && installed.guard_crown_shadow.is_none() {
            installed.guard_crown_shadow = Some(instantiate_shadow(
                baked::GUARD_CROWN_SHADOW_SOURCE,
                origin,
                pose.elevation,
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

fn instantiate(baked: BakedMesh, origin: Pos2) -> Arc<egui::Mesh> {
    let mut mesh = egui::Mesh::default();
    mesh.vertices.reserve(baked.vertices.len());
    mesh.indices.reserve(baked.indices.len());
    for vertex in baked.vertices {
        let [x, y] = vertex.position;
        let [r, g, b, a] = vertex.color;
        mesh.colored_vertex(
            origin + Vec2::new(x, y),
            Color32::from_rgba_unmultiplied(r, g, b, a),
        );
    }
    mesh.indices.extend_from_slice(baked.indices);
    Arc::new(mesh)
}

fn instantiate_shadow(shadow: BakedShadow, origin: Pos2, receiver_z: f32) -> Arc<egui::Mesh> {
    let scale = baked::SHADOW_EYE_Z / (baked::SHADOW_EYE_Z - receiver_z);
    let mut mesh = egui::Mesh::default();
    mesh.vertices.reserve(shadow.mesh.vertices.len());
    mesh.indices.reserve(shadow.mesh.indices.len());
    for vertex in shadow.mesh.vertices {
        let [x, y_plus_slope_z] = vertex.position;
        let [r, g, b, a] = vertex.color;
        mesh.colored_vertex(
            origin
                + Vec2::new(
                    x * scale,
                    (y_plus_slope_z - baked::SHADOW_SLOPE * receiver_z) * scale,
                ),
            Color32::from_rgba_unmultiplied(r, g, b, a),
        );
    }
    mesh.indices.extend_from_slice(shadow.mesh.indices);
    Arc::new(mesh)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spring_converges_on_both_latches_after_contact() {
        for (from, target) in [
            (baked::LATCH_UP, baked::LATCH_DOWN),
            (baked::LATCH_DOWN, baked::LATCH_UP),
        ] {
            let mut spring = Spring::at(from);
            for _ in 0..180 {
                spring.advance(target, 1.0 / 120.0);
            }
            assert!((spring.elevation - target).abs() < 0.002);
            assert!(spring.velocity.abs() < 0.02);
        }
    }

    #[test]
    fn stiff_spring_crosses_and_recoils_within_one_eighth_second() {
        let mut spring = Spring::at(baked::POSE_MIN);
        let mut first_crossing = None;
        let mut recoil = false;
        for step in 1..=60 {
            spring.advance(baked::LATCH_DOWN, INTEGRATOR_STEP);
            if spring.elevation > baked::LATCH_DOWN {
                let _crossing = first_crossing.get_or_insert(step);
            } else if first_crossing.is_some() {
                recoil = true;
                break;
            }
        }
        assert!(first_crossing.is_some_and(|step| step <= 30));
        assert!(recoil);
    }

    #[test]
    fn build_time_atlas_covers_the_complete_travel() {
        assert_eq!(baked::POSES.len(), baked::POSE_COUNT);
        assert_eq!(baked::POSES[0].elevation, baked::POSE_MIN);
        assert_eq!(
            baked::POSES[baked::POSE_COUNT - 1].elevation,
            baked::POSE_MAX
        );
        assert!(
            baked::POSES
                .windows(2)
                .all(|poses| poses[0].elevation < poses[1].elevation)
        );
        assert!(!baked::GUARD.vertices.is_empty());
        assert!(!baked::GUARD_CROWN_SHADOW_SOURCE.mesh.vertices.is_empty());
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
        let raised = diameter(baked::POSES[pose_index(baked::LATCH_UP)].button);
        let recessed = diameter(baked::POSES[pose_index(baked::LATCH_DOWN)].button);
        assert!(
            raised - recessed > 6.0,
            "latched silhouettes differ by only {} points",
            raised - recessed
        );
    }

    #[test]
    fn latch_stroke_and_swept_volume_share_logical_point_units() {
        let stroke = baked::LATCH_UP - baked::LATCH_DOWN;
        let area = (baked::BODY_HALF * 2.0).powi(2);
        assert!((stroke - 31.05).abs() < 0.001);
        assert!((area * stroke - 16_141.0).abs() < 1.0);
    }

    #[test]
    fn pointer_press_changes_latch_and_reports_swept_volume() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(64.0, CONTROL_H));
        let mut checked = false;
        let mut swept = 0.0;
        let _prime = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(screen),
                ..egui::RawInput::default()
            },
            |ui| {
                let _checkbox = Checkbox::without_text(&mut checked).show(ui);
            },
        );
        for pressed in [true, false] {
            let _frame = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(screen),
                    events: vec![
                        egui::Event::PointerMoved(screen.center()),
                        egui::Event::PointerButton {
                            pos: screen.center(),
                            button: egui::PointerButton::Primary,
                            pressed,
                            modifiers: egui::Modifiers::NONE,
                        },
                    ],
                    ..egui::RawInput::default()
                },
                |ui| {
                    let checkbox = Checkbox::without_text(&mut checked).show(ui);
                    swept += checkbox.wake().map_or(0.0, CheckboxWake::swept_volume);
                },
            );
        }
        assert!(checked);
        assert!(swept > 500.0, "plunger swept only {swept} point³");
    }
}
