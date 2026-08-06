//! An editable scalar register driven by a recessed foundry thumbwheel.
//!
//! The actuator is one scalloped oblate solid, forged once in canonical space
//! and baked in both XZ and YZ planes. Runtime scroll travel advances the
//! caller's explicit quantum; double-clicking the register admits exact text
//! entry without changing the surrounding mechanism.

#![deny(missing_docs)]

use std::{
    fmt::Display,
    ops::{Deref, RangeInclusive},
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use egui::{
    Align, Color32, CursorIcon, FontId, Id, Key, Modifiers, Pos2, Rect, Sense, Vec2, WidgetInfo,
    emath::Numeric,
    text::{CCursor, CCursorRange},
};

use super::{
    HOT, foundry,
    plunger::{self, BakedMesh, BakedVertex},
    wheel,
};

const DEFAULT_REGISTER_WIDTH: f32 = 68.0;
const MIN_REGISTER_WIDTH: f32 = 42.0;
const CASING_GAUGE: f32 = 2.0;
const REGISTER_FONT_SIZE: f32 = 13.0;
const MAX_PRECISION: usize = 15;
const ROTOR_STIFFNESS: f32 = 520.0;
const ROTOR_DAMPING: f32 = 25.0;
const ROTOR_STEP: f32 = 1.0 / 240.0;
const ROTOR_LAG_LIMIT: f32 = baked::PITCH * 8.0;
const REFUSAL_KICK: f32 = 7.8;

#[derive(Clone, Copy)]
struct BakedWheelPose {
    phase: f32,
    wheel: BakedMesh,
}

#[derive(Clone, Copy)]
struct BakedWheelPlane {
    aperture: [f32; 2],
    poses: &'static [BakedWheelPose],
}

mod baked {
    use super::{BakedMesh, BakedVertex, BakedWheelPlane, BakedWheelPose};

    include!(concat!(env!("OUT_DIR"), "/number_input_atlas.rs"));
}

/// Plane in which the thumbwheel revolves.
///
/// The remaining axis is its axle. Both variants are rigid transforms of one
/// canonical foundry model; they are projected and illuminated independently.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum WheelPlane {
    /// Horizontal rolling section with a screen-y axle.
    XZ,
    /// Vertical rolling section with a screen-x axle.
    #[default]
    YZ,
}

impl WheelPlane {
    const fn atlas_index(self) -> usize {
        match self {
            Self::XZ => 0,
            Self::YZ => 1,
        }
    }
}

/// Bound that rejected an attempted thumbwheel stroke.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NumberBound {
    /// Inclusive lower limit.
    Minimum,
    /// Inclusive upper limit.
    Maximum,
}

/// One attempted stroke refused by a numerical limit.
///
/// This event is suitable for future contact sound: `excess_detents` retains
/// the magnitude of a fast free-spinning wheel rather than reducing it to a
/// boolean limit hit.
#[derive(Clone, Copy, Debug)]
pub struct NumberRefusal {
    bound: NumberBound,
    excess_detents: f32,
}

impl NumberRefusal {
    /// Limit struck by the attempted motion.
    pub fn bound(self) -> NumberBound {
        self.bound
    }

    /// Absolute rejected travel measured in wheel detents.
    pub fn excess_detents(self) -> f32 {
        self.excess_detents
    }
}

/// A small foundry thumbwheel controlling one bounded numerical primitive.
///
/// `range`, `step`, and `precision` are caller-owned semantics. One wheel
/// detent adds or subtracts `step`; floating values entered through the text
/// override retain their exact offset rather than snapping onto a hidden
/// lattice. Integer primitives require zero decimal places.
///
/// # Example
///
/// ```
/// use dwemer_poolrooms::{chrome::{NumberInput, WheelPlane}, egui};
///
/// fn gain(ui: &mut egui::Ui, value: &mut f32) {
///     let _gain = NumberInput::new(value, -2.0..=2.0, 0.01, 2)
///         .wheel_plane(WheelPlane::YZ)
///         .show(ui);
/// }
/// ```
pub struct NumberInput<'a, N> {
    value: &'a mut N,
    range: RangeInclusive<N>,
    step: N,
    precision: usize,
    plane: WheelPlane,
    register_width: f32,
}

impl<'a, N> NumberInput<'a, N>
where
    N: Numeric + Display + FromStr,
{
    /// Bind a primitive, its inclusive limits, one-detent increment, and
    /// displayed decimal precision.
    ///
    /// # Panics
    ///
    /// Panics when the range is descending or non-finite, `step` is not finite
    /// and positive, precision exceeds fifteen places, or an integer primitive
    /// is assigned nonzero decimal precision.
    pub fn new(value: &'a mut N, range: RangeInclusive<N>, step: N, precision: usize) -> Self {
        validate(*value, &range, step, precision);
        Self {
            value,
            range,
            step,
            precision,
            plane: WheelPlane::default(),
            register_width: DEFAULT_REGISTER_WIDTH,
        }
    }

    /// Select the rigid orientation of the canonical thumbwheel.
    pub const fn wheel_plane(mut self, plane: WheelPlane) -> Self {
        self.plane = plane;
        self
    }

    /// Set the numerical register's exterior width in logical points.
    ///
    /// The wheel retains its fixed small gauge; only the black readout
    /// canister stretches.
    ///
    /// # Panics
    ///
    /// Panics unless `width` is finite and at least 42 points.
    pub fn register_width(mut self, width: f32) -> Self {
        assert!(
            width >= MIN_REGISTER_WIDTH && width.is_finite(),
            "numerical register width must be finite and at least {MIN_REGISTER_WIDTH} points"
        );
        self.register_width = width;
        self
    }

    /// Lay out, actuate, edit, and paint the complete numerical mechanism.
    ///
    /// Hover the wheel and scroll to advance it. Raw line-wheel magnitude is
    /// retained, so a free-spinning wheel can cross many caller quanta in one
    /// frame; point streams bank fractional detents. Double-click the black
    /// register to enter text, then press Enter or leave focus to commit.
    pub fn show(self, ui: &mut egui::Ui) -> NumberInputResponse {
        let Self {
            value,
            range,
            step,
            precision,
            plane,
            register_width,
        } = self;
        validate(*value, &range, step, precision);

        let plane_atlas = baked::PLANES[plane.atlas_index()];
        let desired = Vec2::new(register_width + baked::SOCKET_SIDE, baked::SOCKET_SIDE);
        let (assembly, allocation) = ui.allocate_exact_size(desired, Sense::hover());
        let register =
            Rect::from_min_size(assembly.min, Vec2::new(register_width, assembly.height()));
        let wheel_socket =
            Rect::from_min_max(Pos2::new(register.right(), assembly.top()), assembly.max);
        let register_aperture = register.shrink(CASING_GAUGE);
        let wheel_aperture =
            Rect::from_center_size(wheel_socket.center(), Vec2::from(plane_atlas.aperture));
        let register_id = allocation.id.with("register");
        let wheel_id = allocation.id.with("wheel");
        let editor_id = allocation.id.with("editor");
        let enabled = ui.is_enabled();
        let editing = ui
            .ctx()
            .data_mut(|data| data.get_temp::<String>(editor_id).is_some());

        let register_response = if editing {
            None
        } else {
            Some(ui.interact(register_aperture, register_id, Sense::click()))
        };
        let mut wheel_response = ui.interact(wheel_socket, wheel_id, Sense::click());
        if enabled {
            wheel_response = wheel_response.on_hover_cursor(match plane {
                WheelPlane::XZ => CursorIcon::ResizeHorizontal,
                WheelPlane::YZ => CursorIcon::ResizeVertical,
            });
        }
        if wheel_response.clicked() {
            wheel_response.request_focus();
        }

        let old = *value;
        let clamped = clamp(*value, &range);
        if clamped != *value {
            *value = clamped;
        }
        let mut requested = 0_i32;
        if enabled && !editing && wheel_response.hovered() {
            requested += wheel::precise_notches(ui, wheel_id);
            if requested != 0 {
                wheel_response.request_focus();
            }
        }
        if enabled && !editing && wheel_response.has_focus() {
            requested += ui.input_mut(|input| {
                input.count_and_consume_key(Modifiers::NONE, Key::ArrowUp) as i32
                    - input.count_and_consume_key(Modifiers::NONE, Key::ArrowDown) as i32
            });
        }
        let Step { accepted, refusal } = apply_steps(value, &range, step, requested);

        let motion = rotor_motion(ui, wheel_id, accepted, refusal);
        let mut painter = ui.painter().clone();
        if !enabled {
            painter.set_opacity(1.0);
        }
        paint_canister(
            ui,
            &painter,
            assembly,
            register_aperture,
            wheel_aperture,
            plane,
            motion.angle,
            &wheel_response,
        );

        let mut editor_response = None;
        if editing {
            editor_response = Some(edit_register(
                ui,
                editor_id,
                register_aperture,
                value,
                &range,
                precision,
            ));
        } else {
            let text = format_number(*value, precision);
            let _value = painter.text(
                register_aperture.center(),
                egui::Align2::CENTER_CENTER,
                text,
                FontId::monospace(REGISTER_FONT_SIZE),
                HOT,
            );
            if enabled
                && register_response
                    .as_ref()
                    .is_some_and(egui::Response::double_clicked)
            {
                let _old = ui
                    .ctx()
                    .data_mut(|data| data.insert_temp(editor_id, format_number(*value, precision)));
                ui.ctx()
                    .memory_mut(|memory| memory.request_focus(editor_id));
                ui.ctx().request_repaint();
            }
        }
        paint_index(&painter, register_aperture, wheel_aperture);
        foundry::socket_rim(&painter, register_aperture);
        foundry::socket_rim(&painter, wheel_aperture);

        let mut response = allocation.union(wheel_response);
        if let Some(register_response) = register_response {
            response = response.union(register_response);
        }
        if let Some(editor_response) = editor_response {
            response = response.union(editor_response);
        }
        if *value != old {
            response.mark_changed();
        }
        response.widget_info(|| WidgetInfo::drag_value(enabled, value.to_f64()));
        super::tension(ui, &response);

        NumberInputResponse {
            wake: NumberInputWake::new(wheel_aperture, motion.travel, plane),
            refusal,
            response,
            angle: motion.angle,
            editing: ui
                .ctx()
                .data_mut(|data| data.get_temp::<String>(editor_id).is_some()),
        }
    }
}

fn validate<N: Numeric>(value: N, range: &RangeInclusive<N>, step: N, precision: usize) {
    assert!(
        range.start() <= range.end(),
        "numerical input limits must be ascending"
    );
    assert!(
        value.to_f64().is_finite()
            && range.start().to_f64().is_finite()
            && range.end().to_f64().is_finite(),
        "numerical input value and limits must be finite"
    );
    assert!(
        step.to_f64().is_finite() && step.to_f64() > 0.0,
        "numerical input step must be finite and positive"
    );
    assert!(
        precision <= MAX_PRECISION,
        "numerical input precision cannot exceed {MAX_PRECISION} decimal places"
    );
    assert!(
        !N::INTEGRAL || precision == 0,
        "integer numerical inputs require zero decimal precision"
    );
}

fn clamp<N: Numeric>(value: N, range: &RangeInclusive<N>) -> N {
    if value < *range.start() {
        *range.start()
    } else if value > *range.end() {
        *range.end()
    } else {
        value
    }
}

struct Step {
    accepted: f32,
    refusal: Option<NumberRefusal>,
}

fn apply_steps<N: Numeric>(
    value: &mut N,
    range: &RangeInclusive<N>,
    step: N,
    requested: i32,
) -> Step {
    if requested == 0 {
        return Step {
            accepted: 0.0,
            refusal: None,
        };
    }
    let before = value.to_f64();
    let quantum = step.to_f64();
    let attempted = f64::from(requested);
    let candidate = N::from_f64(before + attempted * quantum);
    let next = clamp(candidate, range);
    *value = next;
    let accepted = (next.to_f64() - before) / quantum;
    let excess = attempted - accepted;
    let refusal = (excess.abs() > 1e-6).then_some(NumberRefusal {
        bound: if excess.is_sign_positive() {
            NumberBound::Maximum
        } else {
            NumberBound::Minimum
        },
        excess_detents: excess.abs() as f32,
    });
    Step {
        accepted: accepted as f32,
        refusal,
    }
}

fn format_number<N: Numeric + Display>(value: N, precision: usize) -> String {
    if N::INTEGRAL {
        value.to_string()
    } else {
        format!("{:.precision$}", value.to_f64())
    }
}

fn parse_number<N: Numeric + FromStr>(text: &str) -> Option<N> {
    text.trim()
        .chars()
        .map(|ch| if ch == '−' { '-' } else { ch })
        .collect::<String>()
        .parse()
        .ok()
}

fn edit_register<N>(
    ui: &mut egui::Ui,
    id: Id,
    aperture: Rect,
    value: &mut N,
    range: &RangeInclusive<N>,
    precision: usize,
) -> egui::Response
where
    N: Numeric + Display + FromStr,
{
    let mut text = ui
        .ctx()
        .data_mut(|data| data.get_temp::<String>(id))
        .unwrap_or_else(|| format_number(*value, precision));
    let mut response = ui.put(
        aperture,
        egui::TextEdit::singleline(&mut text)
            .id(id)
            .font(FontId::monospace(REGISTER_FONT_SIZE))
            .text_color(HOT)
            .background_color(Color32::TRANSPARENT)
            .frame(egui::Frame::NONE)
            .margin(egui::Margin::same(1))
            .horizontal_align(Align::Center)
            .vertical_align(Align::Center)
            .desired_width(aperture.width())
            .min_size(aperture.size())
            .clip_text(true),
    );
    if response.gained_focus() {
        let mut state = egui::TextEdit::load_state(ui.ctx(), id).unwrap_or_default();
        state.cursor.set_char_range(Some(CCursorRange::two(
            CCursor::new(0),
            CCursor::new(text.chars().count()),
        )));
        state.store(ui.ctx(), id);
    }

    let escape = ui.input(|input| input.key_pressed(Key::Escape));
    let commit = response.lost_focus() && !escape;
    if escape {
        ui.ctx().memory_mut(|memory| memory.surrender_focus(id));
        let _discarded = ui.ctx().data_mut(|data| data.remove_temp::<String>(id));
    } else if commit {
        if let Some(parsed) = parse_number(&text) {
            let parsed = clamp(parsed, range);
            if parsed != *value {
                *value = parsed;
                response.mark_changed();
            }
        }
        let _committed = ui.ctx().data_mut(|data| data.remove_temp::<String>(id));
    } else {
        let _old = ui.ctx().data_mut(|data| data.insert_temp(id, text));
    }
    response
}

#[derive(Clone, Copy, Debug, Default)]
struct Rotor {
    angle: f32,
    target: f32,
    velocity: f32,
}

impl Rotor {
    fn drive(&mut self, accepted: f32, refusal: Option<NumberRefusal>) {
        self.target += accepted * baked::PITCH;
        let lag = (self.target - self.angle).clamp(-ROTOR_LAG_LIMIT, ROTOR_LAG_LIMIT);
        self.target = self.angle + lag;
        if let Some(refusal) = refusal {
            let sign = match refusal.bound {
                NumberBound::Minimum => -1.0,
                NumberBound::Maximum => 1.0,
            };
            self.velocity += sign * REFUSAL_KICK * (1.0 + refusal.excess_detents.ln_1p()).min(2.4);
        }
    }

    fn advance(&mut self, dt: f32) -> f32 {
        let before = self.angle;
        let steps = (dt / ROTOR_STEP).ceil() as u32;
        let h = dt / steps.max(1) as f32;
        for _ in 0..steps {
            self.velocity +=
                (-ROTOR_STIFFNESS * (self.angle - self.target) - ROTOR_DAMPING * self.velocity) * h;
            self.angle += self.velocity * h;
        }
        self.angle - before
    }

    fn moving(self) -> bool {
        (self.angle - self.target).abs() > 0.0002 || self.velocity.abs() > 0.004
    }

    fn rebase(&mut self) {
        if self.moving() || self.target.abs() < std::f32::consts::TAU * 4.0 {
            return;
        }
        let turns = (self.target / std::f32::consts::TAU).trunc();
        self.target -= turns * std::f32::consts::TAU;
        self.angle -= turns * std::f32::consts::TAU;
    }
}

struct RotorMotion {
    angle: f32,
    travel: f32,
}

fn rotor_motion(
    ui: &egui::Ui,
    id: Id,
    accepted: f32,
    refusal: Option<NumberRefusal>,
) -> RotorMotion {
    let dt = ui
        .input(|input| input.stable_dt)
        .clamp(1.0 / 240.0, 1.0 / 30.0);
    let (motion, moving) = ui.ctx().data_mut(|data| {
        let key = id.with("torsion");
        let mut rotor = data.get_temp::<Rotor>(key).unwrap_or_default();
        rotor.drive(accepted, refusal);
        let travel = rotor.advance(dt);
        rotor.rebase();
        let moving = rotor.moving();
        let motion = RotorMotion {
            angle: rotor.angle,
            travel,
        };
        let _old = data.insert_temp(key, rotor);
        (motion, moving)
    });
    if moving {
        ui.ctx().request_repaint_after(Duration::from_millis(4));
    }
    motion
}

fn paint_canister(
    ui: &egui::Ui,
    painter: &egui::Painter,
    assembly: Rect,
    register: Rect,
    wheel: Rect,
    plane: WheelPlane,
    angle: f32,
    response: &egui::Response,
) {
    foundry::darkened_sheet(painter, assembly);
    foundry::socket_bed(painter, register);
    foundry::socket_bed(painter, wheel);
    let atlas = baked::PLANES[plane.atlas_index()];
    let phase = angle.rem_euclid(baked::PITCH);
    let pose_index = plunger::pose_index(phase, 0.0, baked::PITCH, atlas.poses.len());
    debug_assert!(
        (atlas.poses[pose_index].phase - phase).abs()
            <= baked::PITCH / (atlas.poses.len() - 1) as f32
    );
    let mesh = ui.ctx().data_mut(|data| {
        data.get_temp_mut_or_default::<WheelCache>(response.id.with("compiled-thumbwheel"))
            .prepare(wheel.center(), plane, pose_index, atlas)
    });
    foundry::paint_compiled(painter, wheel, &mesh);
}

fn paint_index(painter: &egui::Painter, register: Rect, wheel: Rect) {
    let cy = wheel.center().y;
    let base_x = register.right();
    let tip_x = wheel.left() + 2.2;
    let half = 3.3;
    let nose = 0.7;
    foundry::stamp(
        painter,
        vec![
            Pos2::new(base_x, cy - half),
            Pos2::new(base_x, cy + half),
            Pos2::new(tip_x, cy + nose),
            Pos2::new(tip_x, cy - nose),
        ],
        &[[Pos2::new(base_x, cy - half), Pos2::new(tip_x, cy - nose)]],
        &[[Pos2::new(base_x, cy + half), Pos2::new(tip_x, cy + nose)]],
        -0.14,
    );
}

#[derive(Clone, Default)]
struct WheelCache {
    origin: Option<Pos2>,
    plane: Option<WheelPlane>,
    poses: Vec<Option<Arc<egui::Mesh>>>,
}

impl WheelCache {
    fn prepare(
        &mut self,
        origin: Pos2,
        plane: WheelPlane,
        pose: usize,
        atlas: BakedWheelPlane,
    ) -> Arc<egui::Mesh> {
        if self.origin != Some(origin) || self.plane != Some(plane) {
            *self = Self {
                origin: Some(origin),
                plane: Some(plane),
                poses: vec![None; atlas.poses.len()],
            };
        }
        self.poses[pose]
            .get_or_insert_with(|| plunger::instantiate(atlas.poses[pose].wheel, origin))
            .clone()
    }
}

#[must_use = "the response carries egui state, angular water forcing, and limit refusal"]
/// Interaction, rotor motion, edit state, and limit contact from one frame.
pub struct NumberInputResponse {
    response: egui::Response,
    wake: Option<NumberInputWake>,
    refusal: Option<NumberRefusal>,
    angle: f32,
    editing: bool,
}

impl NumberInputResponse {
    /// Angular wheel travel since the preceding frame, if it moved.
    pub fn wake(&self) -> Option<NumberInputWake> {
        self.wake
    }

    /// Limit contact generated by input during this frame.
    pub fn refusal(&self) -> Option<NumberRefusal> {
        self.refusal
    }

    /// Current unwrapped rotor angle in radians.
    pub fn angle(&self) -> f32 {
        self.angle
    }

    /// Whether the numerical register currently owns text focus.
    pub fn editing(&self) -> bool {
        self.editing
    }

    /// Attach a tooltip while retaining the mechanism's physical response.
    pub fn on_hover_text(mut self, text: impl Into<egui::WidgetText>) -> Self {
        self.response = self.response.on_hover_text(text);
        self
    }

    /// Discard physical telemetry and return the ordinary egui response.
    pub fn into_response(self) -> egui::Response {
        self.response
    }
}

impl Deref for NumberInputResponse {
    type Target = egui::Response;

    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

/// Tangential forcing generated by the rotating scalloped wheel.
#[derive(Clone, Copy, Debug)]
pub struct NumberInputWake {
    rect: Rect,
    angular_travel: f32,
    swept_area: f32,
    plane: WheelPlane,
}

impl NumberInputWake {
    fn new(rect: Rect, angular_travel: f32, plane: WheelPlane) -> Option<Self> {
        (angular_travel.abs() >= 0.0002).then_some(Self {
            rect,
            angular_travel,
            swept_area: 2.0 * baked::HALF_DEPTH * baked::RADIUS * angular_travel.abs(),
            plane,
        })
    }

    /// Visible wheel aperture receiving the tangential forcing.
    pub fn rect(self) -> Rect {
        self.rect
    }

    /// Signed wheel rotation during this frame, in radians.
    pub fn angular_travel(self) -> f32 {
        self.angular_travel
    }

    /// First-order surface area swept through the water, in logical point².
    pub fn swept_area(self) -> f32 {
        self.swept_area
    }

    /// Plane selecting the projected direction of tangential motion.
    pub fn plane(self) -> WheelPlane {
        self.plane
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wheel_input(
        screen: Rect,
        center: Pos2,
        unit: egui::MouseWheelUnit,
        y: f32,
    ) -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(screen),
            events: vec![
                egui::Event::PointerMoved(center),
                egui::Event::MouseWheel {
                    unit,
                    delta: Vec2::new(0.0, y),
                    phase: egui::TouchPhase::Move,
                    modifiers: Modifiers::NONE,
                },
            ],
            ..egui::RawInput::default()
        }
    }

    #[test]
    fn integer_and_float_primitives_keep_caller_semantics() {
        let mut integer = 7_i32;
        let whole = apply_steps(&mut integer, &(0..=20), 3, 2);
        assert_eq!(integer, 13);
        assert_eq!(whole.accepted, 2.0);
        assert!(whole.refusal.is_none());
        assert_eq!(format_number(integer, 0), "13");

        let mut float = 0.003_f64;
        let fractional = apply_steps(&mut float, &(-1.0..=1.0), 0.1, 2);
        assert!((float - 0.203).abs() < 1e-12);
        assert_eq!(fractional.accepted, 2.0);
        assert_eq!(format_number(float, 3), "0.203");
    }

    #[test]
    fn partial_last_detent_stops_exactly_at_the_caller_bound() {
        let mut value = 9_i32;
        let step = apply_steps(&mut value, &(0..=10), 3, 2);
        assert!(
            step.refusal.is_some(),
            "upper limit must refuse excess travel"
        );
        let refusal = step.refusal.unwrap_or(NumberRefusal {
            bound: NumberBound::Minimum,
            excess_detents: 0.0,
        });
        assert_eq!(value, 10);
        assert!((step.accepted - 1.0 / 3.0).abs() < 1e-6);
        assert_eq!(refusal.bound(), NumberBound::Maximum);
        assert!((refusal.excess_detents() - 5.0 / 3.0).abs() < 1e-6);

        let reverse = apply_steps(&mut value, &(0..=10), 3, -5);
        assert!(
            reverse.refusal.is_some(),
            "lower limit must refuse excess travel"
        );
        let refusal = reverse.refusal.unwrap_or(NumberRefusal {
            bound: NumberBound::Maximum,
            excess_detents: 0.0,
        });
        assert_eq!(value, 0);
        assert_eq!(refusal.bound(), NumberBound::Minimum);
    }

    #[test]
    fn line_wheel_preserves_fast_detent_magnitude() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(180.0, 40.0));
        let mut value = 0_i32;
        let mut wheel_center = Pos2::ZERO;
        let _prime = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(screen),
                ..egui::RawInput::default()
            },
            |ui| {
                let response = NumberInput::new(&mut value, -100..=100, 2, 0).show(ui);
                wheel_center = Pos2::new(
                    response.rect.right() - baked::SOCKET_SIDE * 0.5,
                    response.rect.center().y,
                );
            },
        );
        let _spin = ctx.run_ui(
            wheel_input(screen, wheel_center, egui::MouseWheelUnit::Line, 7.0),
            |ui| {
                let _response = NumberInput::new(&mut value, -100..=100, 2, 0).show(ui);
            },
        );
        assert_eq!(value, 14);
        assert!(crate::chrome::take_control_wheel(&ctx));
    }

    #[test]
    fn point_wheel_banks_subdetent_motion() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(180.0, 40.0));
        let mut value = 0.0_f32;
        let mut wheel_center = Pos2::ZERO;
        let _prime = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(screen),
                ..egui::RawInput::default()
            },
            |ui| {
                let response = NumberInput::new(&mut value, -1.0..=1.0, 0.125, 3).show(ui);
                wheel_center = Pos2::new(
                    response.rect.right() - baked::SOCKET_SIDE * 0.5,
                    response.rect.center().y,
                );
            },
        );
        for points in [20.0, 30.0] {
            let _spin = ctx.run_ui(
                wheel_input(screen, wheel_center, egui::MouseWheelUnit::Point, points),
                |ui| {
                    let _response = NumberInput::new(&mut value, -1.0..=1.0, 0.125, 3).show(ui);
                },
            );
        }
        assert_eq!(value, 0.125);
    }

    #[test]
    fn claimed_wheel_motion_never_leaks_into_an_enclosing_scroll_area() {
        fn frame(ctx: &egui::Context, value: &mut i32, input: egui::RawInput) -> (Pos2, f32) {
            let mut wheel_center = Pos2::ZERO;
            let mut offset = 0.0;
            let _pass = ctx.run_ui(input, |ui| {
                let scroll = egui::ScrollArea::vertical()
                    .id_salt("number-input-scroll-containment")
                    .show(ui, |ui| {
                        let response = NumberInput::new(value, -9..=9, 1, 0).show(ui);
                        wheel_center = Pos2::new(
                            response.rect.right() - baked::SOCKET_SIDE * 0.5,
                            response.rect.center().y,
                        );
                        ui.add_space(400.0);
                    });
                offset = scroll.state.offset.y;
            });
            (wheel_center, offset)
        }

        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(180.0, 100.0));
        let mut value = 0_i32;
        let (wheel_center, _) = frame(
            &ctx,
            &mut value,
            egui::RawInput {
                screen_rect: Some(screen),
                ..egui::RawInput::default()
            },
        );
        let (_, offset) = frame(
            &ctx,
            &mut value,
            wheel_input(screen, wheel_center, egui::MouseWheelUnit::Line, -1.0),
        );
        assert_eq!(value, -1);
        assert_eq!(offset, 0.0);

        for _ in 0..12 {
            let (_, offset) = frame(
                &ctx,
                &mut value,
                egui::RawInput {
                    screen_rect: Some(screen),
                    predicted_dt: 1.0 / 60.0,
                    ..egui::RawInput::default()
                },
            );
            assert_eq!(offset, 0.0, "smoothed wheel residue escaped containment");
        }
    }

    #[test]
    fn hard_stop_kicks_the_rotor_then_springs_home() {
        let refusal = NumberRefusal {
            bound: NumberBound::Maximum,
            excess_detents: 4.0,
        };
        let mut rotor = Rotor::default();
        rotor.drive(0.0, Some(refusal));
        let first = rotor.advance(1.0 / 60.0);
        assert!(first > 0.0);
        for _ in 0..180 {
            let _travel = rotor.advance(1.0 / 120.0);
        }
        assert!(rotor.angle.abs() < 0.001);
        assert!(rotor.velocity.abs() < 0.01);
    }

    #[test]
    fn one_canonical_atlas_covers_both_rigid_planes() {
        assert_eq!(baked::PLANES.len(), 2);
        assert_eq!(
            baked::PLANES[WheelPlane::XZ.atlas_index()].aperture,
            [14.0, 12.0]
        );
        assert_eq!(
            baked::PLANES[WheelPlane::YZ.atlas_index()].aperture,
            [12.0, 14.0]
        );
        for atlas in baked::PLANES {
            assert_eq!(atlas.poses.len(), baked::POSE_COUNT);
            assert_eq!(atlas.poses[0].phase, 0.0);
            assert_eq!(atlas.poses[baked::POSE_COUNT - 1].phase, baked::PITCH);
            assert!(
                atlas
                    .poses
                    .iter()
                    .all(|pose| !pose.wheel.vertices.is_empty())
            );
        }
    }
}
