//! Date bounds as an austere analog tape-transport — a dreamlike, submerged
//! poolroom fitting. One to three black recesses are sunk into a dim tiled faceplate;
//! down in each runs a strip of near-black magnetic tape that lies flat across
//! a reading head and vanishes over a conveyor roller at either end (a
//! cassette/conveyor profile, seen through a perspective eye), the labels
//! *printed* on the tape and bending over the rollers exactly as in a 3D scene.
//! Only the rollers' capped, spindled ends show past the ribbon, their welded
//! notch turning in lockstep with the tape; a brass index arrow welded to each
//! recess wall points at the value. One physical model — a lightly underdamped
//! spring on a continuous roll offset — drives wheel, drag, and the elastic
//! recoil when the year reel is shoved past its stops; spinning a reel also
//! drags the surrounding water through `Surface::date_spool`.

#![deny(missing_docs)]

use std::ops::{BitOr, BitOrAssign, Deref, RangeInclusive};

use super::{
    self as chrome,
    foundry::{self, StockAxis, bronze},
    wheel,
};

const H: f32 = 104.0;
const RECESS_TILES: f32 = 4.0; // each recess is this many tiles wide
const MIN_RECESS_TILES: f32 = 2.0;
const OUTER_TILES: f32 = 2.0;

// --- The tape path, as a 3D scene --------------------------------------------
// Not a circular dial: a cassette ribbon seen edge-on. The middle of the strip
// lies on a gentle arc (radius R_FLAT — labels nearly full size, readable),
// and past the surface angle BETA_F it wraps a *tight* roller (radius R_ROLL)
// that races the angle to BETA_MAX and curls out of sight, bunching labels.
// All lengths are in aperture half-heights, so the profile scales with the
// reel; a perspective eye CAM_D back folds width as the tape recedes.
const R_FLAT: f32 = 3.8; // flat-run radius (nearly straight) / aperture half-height
const R_ROLL: f32 = 0.50; // roller radius — a tight pulley the tape wraps 180°
const BETA_F: f32 = 0.19; // surface angle where the flat hands off to the roller
// The tape wraps the roller and disappears: we render the front of the wrap,
// curling to edge-on, then the black recess swallows the far side.
const BETA_MAX: f32 = 1.46;
// The ribbon's *print* dissolves over the last stretch of curl before the apex:
// past readability the perspective fold barely shrinks a glyph (fold ≈ 0.83 even
// at the rim), so without this every label beyond the rim collapses to near-full
// width at the apex and consecutive ones pile into a ghost just below the roller.
const CURL_FADE: f32 = 0.55; // radians of curl over which the print fades to nothing
const CAM_D: f32 = 2.4; // eye distance
const PITCH: f32 = 0.46; // label spacing along the tape
const ROLL_GAIN: f32 = PITCH / R_ROLL; // roller turn per pitch of tape (no slip)

// --- Shared foundry light + tape response -----------------------------------
// Direction and half-vector belong to `foundry`; tape, rollers, indices, and
// rails therefore inhabit one light field rather than individually tuned ones.
const SHINE: f32 = 26.0;
const AMBIENT: f32 = 0.17; // floor the curl rides on — lift it clear of the abyss
const DIFFUSE: f32 = 0.34;
const GLOSS: f32 = 0.15;

// Real magnetic tape: a near-black, near-neutral body, with only the faint
// violet iridescence its sheen flashes back — the dark ribbon in the recess.
const ALBEDO: [f32; 3] = [16.0, 14.0, 18.0];
const SPARK: [f32; 3] = [80.0, 58.0, 112.0];
// --- The metal ---------------------------------------------------------------
// Bronze, cylinder sections, and recess tooling all come from `foundry`. The
// faceplate itself remains the dim poolroom mosaic rather than metal.
const TILE_BASE: [f32; 3] = [34.0, 28.0, 20.0]; // pool-tile body (baseline dark)
const TILE_LIFT: [f32; 3] = [17.0, 14.0, 9.0]; // per-tile brightness scatter
const TILE_CAST: f32 = 5.0; // per-tile warm/cool tint scatter (±, r vs b)
const GROUT: egui::Color32 = egui::Color32::from_rgb(17, 13, 8); // tile seam
const TILE: f32 = 15.6; // pool-tile pitch (px)
const CAP_W: f32 = 3.0; // roller end-cap width — the cylinder end seen edge-on
const TAPE_THK: f32 = 1.0; // tape thickness: gap from the tape's top to the cap's

// --- Spring + stops ----------------------------------------------------------
const SPRING_K: f32 = 560.0; // stiffness (ω≈23.7 rad/s, ~0.26s settle)
const SPRING_C: f32 = 24.0; // damping (ζ≈0.5 → a lively overshoot)
const COMMIT: f32 = 0.5; // roll past this and a new label has reached the head
const WALL_STRETCH: f32 = 0.62; // how far the tape gives past a hard stop (pitches)
const WALL_KICK: f32 = 7.0; // recoil velocity slammed back off the stop
const HAZARD_GAP: f32 = 0.72; // blank tape (pitches) between last year and hatch
// Thread/unthread: the cassette rises from or recedes into its socket along z.
const LIFT_K: f32 = 1100.0;
const LIFT_C: f32 = 28.0;

const MONTHS: [&str; 12] = [
    "JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC",
];

/// Boundary refinement between an application's date type and the transport's
/// proleptic-Gregorian mechanism. `from_ymd` receives only valid civil dates.
pub trait GregorianDay: Copy + Eq {
    /// Decompose a valid civil date as `(year, month, day)`.
    ///
    /// Months are numbered `1..=12`; days are numbered from one and must exist
    /// in the returned proleptic-Gregorian month.
    fn ymd(self) -> (i32, u32, u32);

    /// Construct the application date from a valid proleptic-Gregorian date.
    ///
    /// The transport never supplies an invalid month or day.
    fn from_ymd(year: i32, month: u32, day: u32) -> Self;
}

/// A nonempty selection of date reels, always installed in year-month-day order.
///
/// The primitive constants compose with `|`; combined constants cover the
/// usual banks directly.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DateReels(u8);

impl DateReels {
    const YEAR_BIT: u8 = 1 << 0;
    const MONTH_BIT: u8 = 1 << 1;
    const DAY_BIT: u8 = 1 << 2;

    /// A year reel with hard stops at the transport's configured year span.
    pub const YEAR: Self = Self(Self::YEAR_BIT);
    /// A circular month reel.
    pub const MONTH: Self = Self(Self::MONTH_BIT);
    /// A circular day reel whose modulus follows the current month and year.
    pub const DAY: Self = Self(Self::DAY_BIT);
    /// Year and month reels.
    pub const YEAR_MONTH: Self = Self(Self::YEAR_BIT | Self::MONTH_BIT);
    /// Year and day reels.
    pub const YEAR_DAY: Self = Self(Self::YEAR_BIT | Self::DAY_BIT);
    /// Month and day reels, retaining the bound date's hidden year as context.
    pub const MONTH_DAY: Self = Self(Self::MONTH_BIT | Self::DAY_BIT);
    /// The complete year-month-day transport.
    pub const ALL: Self = Self(Self::YEAR_BIT | Self::MONTH_BIT | Self::DAY_BIT);

    /// Whether this bank contains every reel in `other`.
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Number of physical reel apertures in this bank.
    pub const fn reel_count(self) -> usize {
        self.0.count_ones() as usize
    }

    /// Rigid minimum faceplate width in logical egui points.
    ///
    /// Every reel retains a two-tile aperture, adjacent reels retain a one-tile
    /// gutter, and the bank retains one full tile of casing at either end.
    pub const fn minimum_width(self) -> f32 {
        (self.reel_count() as f32 * MIN_RECESS_TILES + (self.reel_count() - 1) as f32 + OUTER_TILES)
            * TILE
    }

    fn contains_reel(self, reel: Reel) -> bool {
        self.0 & reel.bit() != 0
    }

    fn iter(self) -> impl Iterator<Item = Reel> {
        Reel::ALL
            .into_iter()
            .filter(move |reel| self.contains_reel(*reel))
    }
}

impl Default for DateReels {
    fn default() -> Self {
        Self::ALL
    }
}

impl BitOr for DateReels {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for DateReels {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// Signed screen-y travel emitted by one moving tape reel.
#[derive(Clone, Copy, Debug)]
pub struct DateWake {
    rect: egui::Rect,
    travel: f32,
}

impl DateWake {
    fn new(rect: egui::Rect, travel: f32) -> Self {
        Self { rect, travel }
    }

    /// Reel aperture that displaced the water.
    pub fn rect(self) -> egui::Rect {
        self.rect
    }

    /// Signed screen-y travel of the tape head.
    pub fn travel(self) -> f32 {
        self.travel
    }
}

/// A rigid bank of one to three Gregorian tape reels.
///
/// The month and day reels wrap, while the year reel recoils from the
/// configured hard stops. [`DateSpool`] owns no optional value or toggle:
/// applications compose that policy outside the mechanism and project it onto
/// the transport with [`DateSpool::loaded`].
///
/// Reels accept pointer drags and vertical wheel/trackpad motion. The `id`
/// passed to [`DateSpool::show`] must remain stable and unique because it owns
/// the reels' persistent spring state.
///
/// # Example
///
/// ```
/// use brass_poolrooms::{chrome::{DateReels, DateSpool, GregorianDay}, egui};
///
/// # #[derive(Clone, Copy, Eq, PartialEq)]
/// # struct Day(i32, u32, u32);
/// # impl GregorianDay for Day {
/// #     fn ymd(self) -> (i32, u32, u32) { (self.0, self.1, self.2) }
/// #     fn from_ymd(y: i32, m: u32, d: u32) -> Self { Self(y, m, d) }
/// # }
/// fn controls(ui: &mut egui::Ui, value: &mut Day) {
///     let spool = DateSpool::new(value, 2005..=2027)
///         .reels(DateReels::MONTH_DAY)
///         .width(180.0)
///         .label("DATE")
///         .show(ui, "departure-date");
///
///     if spool.changed() {
///         // A visible reel moved to another civil date.
///     }
/// }
/// ```
///
/// An application that owns an `Option<D>` decides whether the date exists,
/// composes its own toggle, and passes that state to [`DateSpool::loaded`].
/// The transport animates the cassette but never synthesizes or clears values.
pub struct DateSpool<'a, D: GregorianDay> {
    value: &'a mut D,
    years: RangeInclusive<i32>,
    label: Option<&'a str>,
    reels: DateReels,
    width: Option<f32>,
    loaded: bool,
}

impl<'a, D: GregorianDay> DateSpool<'a, D> {
    /// Construct a date transport.
    ///
    /// `years` is the inclusive hard-stop range of the year reel. The complete
    /// three-reel bank is installed unless [`DateSpool::reels`] narrows it.
    pub fn new(value: &'a mut D, years: RangeInclusive<i32>) -> Self {
        Self {
            value,
            years,
            label: None,
            reels: DateReels::ALL,
            width: None,
            loaded: true,
        }
    }

    /// Add a caption above the transport.
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Install exactly this nonempty bank of reels.
    pub fn reels(mut self, reels: DateReels) -> Self {
        self.reels = reels;
        self
    }

    /// Project whether the caller currently has a date onto the mechanism.
    ///
    /// A loaded transport is interactive and carries printed tape across its
    /// reading heads. An unloaded transport rejects reel input and springs its
    /// cassette backward into the socket until the apertures are empty. This
    /// controls physical presentation only; `value` remains untouched.
    pub const fn loaded(mut self, loaded: bool) -> Self {
        self.loaded = loaded;
        self
    }

    /// Request a faceplate width in logical egui points.
    ///
    /// The allocation is capped by the available UI width and floored at
    /// [`DateReels::minimum_width`]. If the parent is narrower than that rigid
    /// envelope, the transport truthfully claims its minimum rather than
    /// deforming its apertures, gutters, or casing.
    ///
    /// # Panics
    ///
    /// Panics if `width` is non-finite or non-positive.
    pub fn width(mut self, width: f32) -> Self {
        assert!(
            width.is_finite() && width > 0.0,
            "date-spool width must be finite and positive"
        );
        self.width = Some(width);
        self
    }

    /// Lay out, interact with, and paint the transport.
    ///
    /// `id` must be stable and unique within the egui context.
    ///
    /// # Panics
    ///
    /// Panics if the year range is descending.
    pub fn show(self, ui: &mut egui::Ui, id: impl egui::AsIdSalt) -> DateSpoolResponse {
        date_spool(
            ui,
            id,
            self.label,
            self.value,
            self.years,
            self.reels,
            self.width,
            self.loaded,
        )
    }
}

#[must_use = "the response carries change state and displaced-water wakes"]
/// Value-change state and displaced-water geometry from one [`DateSpool`] frame.
pub struct DateSpoolResponse {
    response: egui::Response,
    wakes: [Option<DateWake>; 2],
}

impl DateSpoolResponse {
    /// Whether the bound date changed during this UI pass.
    pub fn changed(&self) -> bool {
        self.response.changed()
    }

    /// Iterate over tape motion emitted during this UI pass.
    pub fn wakes(&self) -> impl Iterator<Item = DateWake> + '_ {
        self.wakes.iter().copied().flatten()
    }

    /// Discard the water wakes and return the ordinary egui response.
    pub fn into_response(self) -> egui::Response {
        self.response
    }
}

impl Deref for DateSpoolResponse {
    type Target = egui::Response;

    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum Reel {
    #[default]
    Year,
    Month,
    Day,
}

impl Reel {
    const ALL: [Reel; 3] = [Reel::Year, Reel::Month, Reel::Day];

    const fn bit(self) -> u8 {
        match self {
            Self::Year => DateReels::YEAR_BIT,
            Self::Month => DateReels::MONTH_BIT,
            Self::Day => DateReels::DAY_BIT,
        }
    }

    const fn index(self) -> usize {
        match self {
            Self::Year => 0,
            Self::Month => 1,
            Self::Day => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Parts {
    year: i32,
    month: u32,
    day: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct YearSpan {
    lo: i32,
    hi: i32,
}

impl YearSpan {
    fn refine(years: RangeInclusive<i32>) -> Self {
        let lo = *years.start();
        let hi = *years.end();
        assert!(lo <= hi, "date-spool year span must be ascending");
        Self { lo, hi }
    }
}

/// One reel's living state: the visual roll offset (in label pitches) of the
/// committed value away from the head, and the spring velocity carrying it
/// home. `turns` accumulates committed steps so the rollers can spin in
/// lockstep with the tape (visual tape position = `turns - roll`).
#[derive(Clone, Copy, Debug, Default)]
struct Drum {
    roll: f32,
    vel: f32,
    turns: f32,
}

impl Drum {
    /// Relax the roll toward the detent (or toward a clamped stop), returning
    /// whether the reel is still in motion and wants another frame.
    fn relax(&mut self, dt: f32, floor: f32, ceil: f32) -> bool {
        self.vel += (-SPRING_K * self.roll - SPRING_C * self.vel) * dt;
        self.roll += self.vel * dt;
        if self.roll < floor {
            self.roll = floor;
            self.vel = self.vel.max(0.0);
        } else if self.roll > ceil {
            self.roll = ceil;
            self.vel = self.vel.min(0.0);
        }
        self.roll.abs() > 5e-4 || self.vel.abs() > 5e-4
    }
}

fn date_spool<D: GregorianDay>(
    ui: &mut egui::Ui,
    id: impl egui::AsIdSalt,
    label: Option<&str>,
    value: &mut D,
    years: RangeInclusive<i32>,
    reels: DateReels,
    width: Option<f32>,
    loaded: bool,
) -> DateSpoolResponse {
    let years = YearSpan::refine(years);
    let id = ui.make_persistent_id(id);
    let before = *value;
    let mut turn = ui
        .vertical(|ui| {
            if let Some(label) = label {
                let _label = ui.label(chrome::muted(label));
            }
            let available = ui.available_width();
            let width = width
                .unwrap_or(available)
                .min(available)
                .max(reels.minimum_width());
            chronometer(ui, id, value, years, reels, width, loaded)
        })
        .inner;
    if *value != before {
        turn.response.mark_changed();
    }
    DateSpoolResponse {
        response: turn.response,
        wakes: turn.wakes,
    }
}

struct Turn {
    response: egui::Response,
    wakes: [Option<DateWake>; 2],
}

/// Take the shared flag indicating that crafted chrome consumed wheel motion.
///
/// This date-spool-specific spelling remains for source compatibility; prefer
/// [`super::take_control_wheel`] for new code. It observes wheel-enabled rails
/// and date reels alike. A second call before another control consumes motion
/// returns `false`.
pub fn take_date_spool_wheel(ctx: &egui::Context) -> bool {
    wheel::take_control_wheel(ctx)
}

fn chronometer<D: GregorianDay>(
    ui: &mut egui::Ui,
    id: egui::Id,
    value: &mut D,
    years: YearSpan,
    visible: DateReels,
    width: f32,
    loaded: bool,
) -> Turn {
    let operable = loaded && ui.is_enabled();
    let sense = if operable {
        egui::Sense::click_and_drag()
    } else {
        egui::Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, H), sense);
    let mut parts: Parts = value.ymd().into();
    if visible.contains(DateReels::YEAR) {
        parts.year = parts.year.clamp(years.lo, years.hi);
    }
    parts.clamp_day();
    *value = D::from_ymd(parts.year, parts.month, parts.day);
    let reels = ReelLayout::new(rect, visible);
    let dt = ui
        .input(|input| input.stable_dt)
        .clamp(1.0 / 240.0, 1.0 / 30.0);

    let mut pulse = None;
    let mut couple = None;

    let hovered = operable
        .then(|| ui.ctx().pointer_latest_pos().and_then(|pos| reels.at(pos)))
        .flatten();
    let drag = operable
        .then(|| pump_drag(ui, id, &response, hovered, reels))
        .flatten();
    // Apply at most one source of intent this frame: a held drag wins. The
    // pulse direction is the screen-y the tape head travels — the gesture
    // direction — so the water gets shoved off that side. A visible day reel
    // clamped by the turn rides along as `couple`, carrying its own shove.
    if let Some((reel, slip)) = drag {
        if slip.abs() > 1e-4 {
            let (_, day) = turn_reel(ui, id, &mut parts, reel, Slip::Drag(slip), years, visible);
            pulse = Some(DateWake::new(reels.required(reel), slip.signum()));
            couple = day.map(|dir| DateWake::new(reels.required(Reel::Day), dir));
        }
    } else if let Some(reel) = hovered.filter(|_| response.hovered()) {
        // Own every scrap of scroll over a reel — not only the notch frame but
        // egui's whole smoothing tail — so the panel under us never drifts and
        // no stray delta bounces the date back.
        let steps = wheel::notches(ui, id.with((reel as u8, "wheel")));
        if steps != 0 {
            let (_, day) = turn_reel(ui, id, &mut parts, reel, Slip::Notch(steps), years, visible);
            pulse = Some(DateWake::new(
                reels.required(reel),
                -(steps.signum() as f32),
            ));
            couple = day.map(|dir| DateWake::new(reels.required(Reel::Day), dir));
        }
    }
    *value = D::from_ymd(parts.year, parts.month, parts.day);

    // Hidden drums cannot retain a ghost transient if a caller changes banks.
    let held = loaded.then(|| drag_capture(ui, id)).flatten();
    let mut moving = false;
    for reel in Reel::ALL {
        if !visible.contains_reel(reel) {
            with_drum(ui, id, reel, |drum| *drum = Drum::default());
            continue;
        }
        if held == Some(reel) {
            continue;
        }
        let (floor, ceil) = stretch_bounds(reel, parts, years);
        moving |= with_drum(ui, id, reel, |drum| drum.relax(dt, floor, ceil));
    }
    if moving {
        ui.ctx().request_repaint();
    }

    let lift = cassette_lift(ui, id, loaded, dt);
    paint(ui, id, rect, reels, parts, years, lift, loaded);

    Turn {
        response,
        wakes: [pulse, couple],
    }
}

enum Slip {
    Notch(i32),
    Drag(f32),
}

/// Fold an intent into a reel: spin the committed value across detents,
/// catching the tape up with roll so the new value rolls into the head, and
/// recoiling off a hard stop the value refuses to cross.
fn turn_reel(
    ui: &egui::Ui,
    id: egui::Id,
    parts: &mut Parts,
    reel: Reel,
    slip: Slip,
    years: YearSpan,
    visible: DateReels,
) -> (bool, Option<f32>) {
    let before = *parts;
    let day_before = parts.day;
    with_drum(ui, id, reel, |drum| match slip {
        Slip::Notch(steps) => {
            for _ in 0..steps.unsigned_abs() {
                let _moved = commit(parts, drum, reel, steps.signum(), years);
            }
        }
        Slip::Drag(slip) => {
            drum.roll += slip;
            while drum.roll > COMMIT {
                if !commit(parts, drum, reel, -1, years) {
                    break;
                }
            }
            while drum.roll < -COMMIT {
                if !commit(parts, drum, reel, 1, years) {
                    break;
                }
            }
        }
    });
    // Mechanical day coupling: a month/year turn that forced the day to clamp
    // rolls the day reel down to its new value (banking turns for its rollers, so
    // it spins too) instead of teleporting. The returned sign is the screen-y its
    // head travels, for the water it shoves.
    let couple = (reel != Reel::Day && parts.day != day_before && visible.contains_reel(Reel::Day))
        .then(|| {
            let delta = parts.day as f32 - day_before as f32;
            with_drum(ui, id, Reel::Day, |day| {
                day.roll += delta;
                day.turns += delta;
            });
            -delta.signum()
        });
    (*parts != before, couple)
}

/// One detent of travel in `dir` (+1 = next/larger). Returns whether the value
/// actually moved; a refused move leaves a stretched, recoiling tape instead.
fn commit(parts: &mut Parts, drum: &mut Drum, reel: Reel, dir: i32, years: YearSpan) -> bool {
    if parts.spin(reel, dir, years) {
        // The new value is at the head; let the old one still sit there and
        // spring the fresh label up into place. The tape advanced one pitch, so
        // bank a step for the rollers to turn through.
        drum.roll += dir as f32;
        drum.turns += dir as f32;
        true
    } else {
        drum.roll = (drum.roll - dir as f32 * WALL_STRETCH).clamp(-WALL_STRETCH, WALL_STRETCH);
        drum.vel = dir as f32 * WALL_KICK;
        false
    }
}

/// The roll interval a reel may occupy: months/days are endless, the year reel
/// hits a rubber wall a fraction past its first/last admissible value.
fn stretch_bounds(reel: Reel, parts: Parts, years: YearSpan) -> (f32, f32) {
    if reel != Reel::Year {
        return (f32::NEG_INFINITY, f32::INFINITY);
    }
    let floor = if parts.year <= years.lo {
        -WALL_STRETCH
    } else {
        f32::NEG_INFINITY
    };
    let ceil = if parts.year >= years.hi {
        WALL_STRETCH
    } else {
        f32::INFINITY
    };
    (floor, ceil)
}

// --- Input plumbing ----------------------------------------------------------

/// Continuous drag of a captured reel: returns the reel and the roll slip (in
/// pitches) to fold in this frame. The capture survives until the drag ends.
fn pump_drag(
    ui: &egui::Ui,
    id: egui::Id,
    response: &egui::Response,
    hovered: Option<Reel>,
    reels: ReelLayout,
) -> Option<(Reel, f32)> {
    let key = id.with("drag");
    let stopped =
        response.drag_stopped() || (!response.dragged() && !response.is_pointer_button_down_on());
    let dragged = response.dragged();
    let travel = response.drag_delta().y;
    ui.ctx().data_mut(|data| {
        if stopped {
            let _ = data.remove_temp::<Reel>(key);
            return None;
        }
        if !dragged {
            return None;
        }
        // The reel the drag first grabbed stays captured until the button lifts,
        // so a sloppy vertical drag can't hop between adjacent reels.
        let reel = match data.get_temp::<Reel>(key).or(hovered) {
            Some(reel) => reel,
            None => return None,
        };
        let _ = data.insert_temp(key, reel);
        // Drag travels in pixels; one pitch is one center-spaced label.
        let pitch = Spool::new(reels.required(reel)).pitch();
        let slip = travel / pitch.max(1.0);
        Some((reel, slip))
    })
}

fn drag_capture(ui: &egui::Ui, id: egui::Id) -> Option<Reel> {
    ui.ctx().data(|data| data.get_temp::<Reel>(id.with("drag")))
}

fn with_drum<R>(ui: &egui::Ui, id: egui::Id, reel: Reel, edit: impl FnOnce(&mut Drum) -> R) -> R {
    let key = id.with((reel as u8, "drum"));
    ui.ctx().data_mut(|data| {
        let mut drum = data.get_temp::<Drum>(key).unwrap_or_default();
        let out = edit(&mut drum);
        let _ = data.insert_temp(key, drum);
        out
    })
}

// --- The tape path -----------------------------------------------------------

/// One sampled point on the tape: its screen height `y`, the surface-normal
/// angle `beta` (0 = facing the eye) that drives shading, and the perspective
/// `fold` that narrows width as the strip recedes over a roller.
#[derive(Clone, Copy)]
struct Sample {
    y: f32,
    beta: f32,
    fold: f32,
}

/// The tape transport: a gentle flat run that wraps a tight roller at each end.
/// Distances ride in aperture half-heights so the whole profile scales.
#[derive(Clone, Copy)]
struct Spool {
    cx: f32,
    cy: f32,
    a: f32,
}

impl Spool {
    fn new(window: egui::Rect) -> Self {
        Self {
            cx: window.center().x,
            cy: window.center().y,
            a: window.height() * 0.5,
        }
    }

    /// Carry a signed arc-length `t` (px from the head) onto the cassette curve:
    /// the gentle `R_FLAT` arc until the surface tilts to `BETA_F`, then the
    /// tight `R_ROLL` roller racing to `BETA_MAX`. Height and width both fold
    /// through the perspective eye as the tape recedes.
    fn sample(self, t: f32) -> Sample {
        let (flat_r, roll_r, eye) = (R_FLAT * self.a, R_ROLL * self.a, CAM_D * self.a);
        let sgn = if t < 0.0 { -1.0 } else { 1.0 };
        let arc = t.abs();
        let flat_arc = flat_r * BETA_F;
        let (beta, height, depth) = if arc <= flat_arc {
            let beta = arc / flat_r;
            (beta, flat_r * beta.sin(), flat_r * (1.0 - beta.cos()))
        } else {
            let beta = (BETA_F + (arc - flat_arc) / roll_r).min(BETA_MAX);
            let height = flat_r * BETA_F.sin() + roll_r * (beta.sin() - BETA_F.sin());
            let depth = flat_r * (1.0 - BETA_F.cos()) + roll_r * (BETA_F.cos() - beta.cos());
            (beta, height, depth)
        };
        let fold = eye / (eye + depth);
        Sample {
            y: self.cy + sgn * height * fold,
            beta: sgn * beta,
            fold,
        }
    }

    /// Arc-length between adjacent labels (their spacing along the flat run).
    fn pitch(self) -> f32 {
        self.a * PITCH
    }

    /// Arc-length past which the tape has curled edge-on; labels beyond cull.
    fn rim(self) -> f32 {
        self.a * (R_FLAT * BETA_F + R_ROLL * (BETA_MAX - BETA_F))
    }
}

/// Diffuse and specular response of the tape surface at normal angle `beta`.
fn lumen(beta: f32) -> (f32, f32) {
    let (s, c) = (beta.sin(), beta.cos());
    foundry::yz_lumen(s, c, SHINE)
}

fn tape_rgb(beta: f32, gain: f32) -> egui::Color32 {
    let (diff, spec) = lumen(beta);
    let body = AMBIENT + DIFFUSE * diff;
    let glint = GLOSS * spec * (0.35 + 0.65 * gain);
    let chan = |albedo: f32, spark: f32| (albedo * body * gain + spark * glint).min(255.0) as u8;
    egui::Color32::from_rgb(
        chan(ALBEDO[0], SPARK[0]),
        chan(ALBEDO[1], SPARK[1]),
        chan(ALBEDO[2], SPARK[2]),
    )
}

/// Ink shaded by the same light: printed labels dim toward the rollers and
/// catch fire as they cross the gloss band on the flat run.
fn ink_shade(beta: f32, base: egui::Color32) -> egui::Color32 {
    let (diff, spec) = lumen(beta);
    let lit = (0.46 + 0.72 * diff).min(1.25);
    let glow = spec * 0.18;
    let chan = |c: u8, spark: f32| (f32::from(c) * lit + spark * glow).min(255.0) as u8;
    egui::Color32::from_rgb(
        chan(base.r(), SPARK[0]),
        chan(base.g(), SPARK[1]),
        chan(base.b(), SPARK[2]),
    )
}

// --- Painting: one austere brass control, blue tape sunk into it -------------

fn paint(
    ui: &egui::Ui,
    id: egui::Id,
    rect: egui::Rect,
    reels: ReelLayout,
    parts: Parts,
    years: YearSpan,
    lift: f32,
    loaded: bool,
) {
    let painter = ui.painter();
    facia(painter, rect);
    for (reel, window) in reels.iter() {
        let (roll, turns) = with_drum(ui, id, reel, |drum| (drum.roll, drum.turns));
        draw_reel(
            painter, window, reel, parts, years, roll, turns, lift, loaded,
        );
    }
}

/// A stiff z-axis spring threading or withdrawing the cassette bank.
fn cassette_lift(ui: &egui::Ui, id: egui::Id, loaded: bool, dt: f32) -> f32 {
    #[derive(Clone, Copy)]
    struct Lift {
        position: f32,
        velocity: f32,
    }

    let key = id.with("cassette-lift");
    let (position, moving) = ui.ctx().data_mut(|data| {
        let target = f32::from(loaded);
        let mut lift = data.get_temp::<Lift>(key).unwrap_or(Lift {
            position: target,
            velocity: 0.0,
        });
        lift.velocity += (-LIFT_K * (lift.position - target) - LIFT_C * lift.velocity) * dt;
        lift.position += lift.velocity * dt;
        if lift.position < 0.0 {
            lift.position = 0.0;
            lift.velocity = lift.velocity.max(0.0);
        }
        let moving = (lift.position - target).abs() > 1e-3 || lift.velocity.abs() > 1e-3;
        let _old = data.insert_temp(key, lift);
        (lift.position, moving)
    });
    if moving {
        ui.ctx().request_repaint();
    }
    position
}

/// The faceplate: the dim poolroom tilework the reels are sunk into — baseline
/// dark, scattered tile-to-tile in both brightness and a faint warm/cool cast,
/// set off by darker grout and ringed by the same machined lip as the recesses.
fn facia(painter: &egui::Painter, rect: egui::Rect) {
    let _grout = painter.rect_filled(rect, 3.0, GROUT);
    let clip = painter.with_clip_rect(rect);
    let cols = (rect.width() / TILE).ceil() as i32;
    let rows = (rect.height() / TILE).ceil() as i32;
    for cy in 0..rows {
        for cx in 0..cols {
            let min = egui::pos2(
                rect.left() + cx as f32 * TILE,
                rect.top() + cy as f32 * TILE,
            );
            let tile = egui::Rect::from_min_size(min, egui::Vec2::splat(TILE)).intersect(rect);
            let (n, m) = tile_hash(cx, cy);
            let cast = (m - 0.5) * 2.0 * TILE_CAST; // + = warmer (more r, less b)
            let chan = |base: f32, lift: f32, c: f32| (base + lift * n + c).clamp(0.0, 255.0) as u8;
            let color = egui::Color32::from_rgb(
                chan(TILE_BASE[0], TILE_LIFT[0], cast),
                chan(TILE_BASE[1], TILE_LIFT[1], 0.0),
                chan(TILE_BASE[2], TILE_LIFT[2], -cast),
            );
            let _t = clip.add(egui::Shape::rect_filled(tile.shrink(0.7), 1.0, color));
        }
    }
    let _frame = painter.rect_stroke(
        rect,
        3.0,
        egui::Stroke::new(1.5_f32, bronze(0.13)),
        egui::StrokeKind::Inside,
    );
}

fn tile_hash(cx: i32, cy: i32) -> (f32, f32) {
    let h = (cx.wrapping_mul(73_856_093) ^ cy.wrapping_mul(19_349_663)) as u32;
    let n = f32::from((h >> 19) as u16) / 8192.0;
    let m = f32::from((h >> 3) as u16 & 0x1fff) / 8192.0;
    (n, m)
}

fn draw_reel(
    painter: &egui::Painter,
    window: egui::Rect,
    reel: Reel,
    parts: Parts,
    years: YearSpan,
    roll: f32,
    turns: f32,
    lift: f32,
    loaded: bool,
) {
    foundry::socket_bed(painter, window);
    if lift > 0.02 {
        // The transport recedes in z, preserving every rigid proportion rather
        // than sliding down-screen or dissolving as generic disabled chrome.
        let scale = (0.2 + 0.8 * lift).min(1.0);
        let content = egui::Rect::from_center_size(window.center(), window.size() * scale);
        let spool = Spool::new(content);
        let tape_w = (window.width() - 12.0) * scale;
        let clip = painter.with_clip_rect(window);
        let gain = if loaded { 1.0 } else { 0.5 };
        let _tape = clip.add(tape_mesh(spool, tape_w, gain));
        tape_edges(&clip, spool, tape_w, gain);

        if loaded {
            let cull = spool.rim() + spool.pitch();
            let (mut top_stop, mut bot_stop) = (None, None);
            for lane in -6..=6 {
                let t = (lane as f32 + roll) * spool.pitch();
                if t.abs() > cull {
                    continue;
                }
                match label(reel, parts, years, lane) {
                    Some(text) => print_label(&clip, spool, tape_w, t, &text),
                    // Remember the stop nearest the head on each side; the
                    // hazard occupies one continuous band to the curl.
                    None if t < 0.0 => {
                        top_stop = Some(top_stop.map_or(lane, |l: i32| l.max(lane)));
                    }
                    None => bot_stop = Some(bot_stop.map_or(lane, |l: i32| l.min(lane))),
                }
            }
            if let Some(lane) = top_stop {
                hazard(&clip, spool, tape_w, lane, roll, -1.0);
            }
            if let Some(lane) = bot_stop {
                hazard(&clip, spool, tape_w, lane, roll, 1.0);
            }
        }

        // The roller grips the tape with no slip, so its surface tracks the
        // ribbon's travel — `roll - turns`, not its negative.
        let phase = (roll - turns) * ROLL_GAIN;
        roller(&clip, spool, content, window, true, phase);
        roller(&clip, spool, content, window, false, phase);

        // The last third of withdrawal falls behind the socket darkness.
        let veil = (((0.4 - lift) / 0.4).clamp(0.0, 1.0) * 255.0) as u8;
        if veil > 0 {
            let _veil = clip.rect_filled(window, 0.0, egui::Color32::from_black_alpha(veil));
        }
    }

    index_arrow(painter, window, loaded);
    foundry::socket_rim(painter, window);
}

/// The two ends of one conveyor roller. The roller is a cylinder lying along
/// screen-x with the tape wrapped over its front, so each end reads edge-on as
/// a **thin vertical sliver** — the cap — poking past the ribbon, on a spindle
/// pinned to the recess wall. The cap's top sits one tape-thickness past where
/// the ribbon goes vertical. The welded longitudinal seam projects onto the
/// sliver as a mark that circles with the drum — riding up and down (`r·sinθ`)
/// across the near face and hidden once it rounds to the far side (`cosθ < 0`).
fn roller(
    painter: &egui::Painter,
    spool: Spool,
    content: egui::Rect,
    window: egui::Rect,
    top: bool,
    phase: f32,
) {
    let rim = spool.rim();
    let apex = spool.sample(if top { -rim } else { rim }).y;
    let scale = content.height() / window.height();
    let cap_h = foundry::CONTROL_STOCK_DIAMETER * scale;
    let r = cap_h * 0.5;
    let cap_cy = if top {
        apex + TAPE_THK + r
    } else {
        apex - TAPE_THK - r
    };
    for side in [-1.0_f32, 1.0] {
        let edge = if side < 0.0 {
            content.left()
        } else {
            content.right()
        };
        let cap_x = edge - side * (CAP_W * 0.5 + 2.4);
        // The spindle journals into the real recess wall, so it lengthens as the
        // transport rests inset and shortens as the lever pops up to fill.
        let wall_x = if side < 0.0 {
            window.left()
        } else {
            window.right()
        };
        let _spindle = painter.add(egui::Shape::line_segment(
            [
                egui::pos2(wall_x, cap_cy),
                egui::pos2(cap_x + side * CAP_W * 0.5, cap_cy),
            ],
            egui::Stroke::new(1.2_f32, bronze(0.4)),
        ));
        // The same untapered cylindrical stock used by the rail carriage,
        // rotated onto screen-x. Global light, alloy, and section are shared.
        foundry::cylinder(
            painter,
            egui::Rect::from_center_size(egui::pos2(cap_x, cap_cy), egui::vec2(CAP_W, cap_h)),
            StockAxis::ScreenX,
        );
        // the welded seam, edge-on: a mark riding up and down with the rotation
        // (`r·sinθ`), occluded once it rounds to the drum's far side (`cosθ < 0`)
        // and fading as it grazes the silhouette, so it circles instead of
        // sliding back retrograde.
        let face = phase.cos();
        if face > 0.0 {
            let ny = cap_cy + (r - 0.6) * phase.sin();
            let fade = ((face / 0.3).min(1.0) * 255.0) as u8;
            let _notch = painter.add(egui::Shape::line_segment(
                [
                    egui::pos2(cap_x - CAP_W * 0.5, ny),
                    egui::pos2(cap_x + CAP_W * 0.5, ny),
                ],
                egui::Stroke::new(
                    1.0_f32,
                    egui::Color32::from_rgba_unmultiplied(3, 3, 4, fade),
                ),
            ));
        }
    }
}

/// The reading head: a bronze index arrow hanging over the black recess at the
/// centre line, pointing in at the value. Its broad base is welded to the right
/// recess wall — anchored to the housing, not floating in the void.
fn index_arrow(painter: &egui::Painter, window: egui::Rect, loaded: bool) {
    let cy = window.center().y;
    let base_h = 5.4;
    let tip_h = base_h * 0.2; // a snub, sawed-off nose — tip ≈ 0.2× the base
    let base_x = window.right();
    let tip_x = window.right() - 9.0;
    let lit = if loaded { 0.0 } else { -0.2 };
    // a foot welding the base into the recess wall.
    let _foot = painter.rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(base_x - 0.5, cy - base_h - 1.0),
            egui::pos2(base_x + 1.8, cy + base_h + 1.0),
        ),
        0.5,
        bronze(0.42 + lit),
    );
    let wedge = vec![
        egui::pos2(base_x, cy - base_h),
        egui::pos2(base_x, cy + base_h),
        egui::pos2(tip_x, cy + tip_h),
        egui::pos2(tip_x, cy - tip_h),
    ];
    foundry::stamp(
        painter,
        wedge,
        &[[
            egui::pos2(base_x, cy - base_h),
            egui::pos2(tip_x, cy - tip_h),
        ]],
        &[[
            egui::pos2(base_x, cy + base_h),
            egui::pos2(tip_x, cy + tip_h),
        ]],
        lit,
    );
}

/// The glossy ribbon, as a per-scanline-shaded mesh whose width and height fold
/// with perspective as the tape lies flat then curls over its rollers.
fn tape_mesh(spool: Spool, tape_w: f32, gain: f32) -> egui::Shape {
    const ROWS: usize = 44;
    let half = tape_w * 0.5;
    let rim = spool.rim();
    let mut mesh = egui::Mesh::default();
    for row in 0..=ROWS {
        let t = rim * (2.0 * row as f32 / ROWS as f32 - 1.0);
        let s = spool.sample(t);
        let color = tape_rgb(s.beta, gain);
        mesh.colored_vertex(egui::pos2(spool.cx - half * s.fold, s.y), color);
        mesh.colored_vertex(egui::pos2(spool.cx + half * s.fold, s.y), color);
        if row > 0 {
            let base = (row as u32 - 1) * 2;
            mesh.add_triangle(base, base + 1, base + 2);
            mesh.add_triangle(base + 1, base + 3, base + 2);
        }
    }
    egui::Shape::mesh(mesh)
}

/// The two recessed tape edges, darkened and curving with the surface.
fn tape_edges(painter: &egui::Painter, spool: Spool, tape_w: f32, gain: f32) {
    const ROWS: usize = 28;
    let half = tape_w * 0.5;
    let rim = spool.rim();
    for side in [-1.0_f32, 1.0] {
        let pts: Vec<egui::Pos2> = (0..=ROWS)
            .map(|row| {
                let s = spool.sample(rim * (2.0 * row as f32 / ROWS as f32 - 1.0));
                egui::pos2(spool.cx + side * half * s.fold, s.y)
            })
            .collect();
        let edge = egui::Color32::from_rgb(
            (16.0 * gain) as u8,
            (22.0 * gain) as u8,
            (34.0 * gain) as u8,
        );
        let _edge = painter.add(egui::Shape::line(pts, egui::Stroke::new(1.0_f32, edge)));
    }
}

/// Lay a label flat at the head, then carry it onto the tape at arc-length
/// `t_center` and bend every glyph along the profile, shaded by the same light
/// — printed-on-tape, not floated-over-it.
fn print_label(painter: &egui::Painter, spool: Spool, tape_w: f32, t_center: f32, text: &str) {
    let Some(mut shape) = lay_out(painter, spool, tape_w, text, chrome::HOT) else {
        return;
    };
    warp(spool, &mut shape, t_center, true);
    let _glyphs = painter.add(shape);
}

fn lay_out(
    painter: &egui::Painter,
    spool: Spool,
    tape_w: f32,
    text: &str,
    color: egui::Color32,
) -> Option<egui::epaint::TextShape> {
    let size = (tape_w * 0.32).clamp(12.0, 18.0);
    painter.fonts_mut(|fonts| {
        match egui::Shape::text(
            fonts,
            egui::pos2(spool.cx, spool.cy),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::new(size, egui::FontFamily::Monospace),
            color,
        ) {
            egui::Shape::Text(text) => Some(text),
            _ => None,
        }
    })
}

/// Project a flat-laid galley onto the tape at arc-length `t_center`: each
/// vertex's flat height becomes an arc offset along the profile, folding
/// through perspective and dimming by the surface light.
fn warp(spool: Spool, shape: &mut egui::epaint::TextShape, t_center: f32, shade: bool) {
    let galley = std::sync::Arc::make_mut(&mut shape.galley);
    galley.mesh_bounds = egui::Rect::NOTHING;
    galley.rect = egui::Rect::NOTHING;
    for placed in &mut galley.rows {
        let row_pos = placed.pos;
        let row = std::sync::Arc::make_mut(&mut placed.row);
        let mut bounds = egui::Rect::NOTHING;
        for vertex in &mut row.visuals.mesh.vertices {
            let flat = shape.pos + row_pos.to_vec2() + vertex.pos.to_vec2();
            let s = spool.sample(t_center + (flat.y - spool.cy));
            let world = egui::pos2(spool.cx + (flat.x - spool.cx) * s.fold, s.y);
            vertex.pos = world - shape.pos.to_vec2() - row_pos.to_vec2();
            if shade {
                // Dissolve the print as it rounds the curl, so a label vanishes
                // into the roller instead of collapsing onto the apex and piling
                // into a ghost beneath it.
                let fade = ((BETA_MAX - s.beta.abs()) / CURL_FADE).clamp(0.0, 1.0);
                let lit = ink_shade(s.beta, vertex.color);
                let dim = |c: u8| (f32::from(c) * fade) as u8;
                vertex.color = egui::Color32::from_rgba_premultiplied(
                    dim(lit.r()),
                    dim(lit.g()),
                    dim(lit.b()),
                    dim(lit.a()),
                );
            }
            bounds.extend_with(vertex.pos);
        }
        if !bounds.is_positive() {
            continue;
        }
        row.visuals.mesh_bounds = bounds;
        galley
            .mesh_bounds
            .extend_with(row_pos + bounds.min.to_vec2());
        galley
            .mesh_bounds
            .extend_with(row_pos + bounds.max.to_vec2());
    }
    galley.rect = galley.mesh_bounds;
}

/// The end of tape: hazard hatching *printed on the ribbon* past the last
/// admissible year. The hatch is a flat diagonal field (`across + arc = c`), but
/// each stripe is sampled along the arc and projected through the same
/// perspective fold as the printed labels — so it bows and crowds over the
/// rollers like a true 3-D print, and tapers to the tape's real silhouette
/// instead of standing as a flat screen-space grate. Bounded to the invalid
/// arc-run [gap, curl] so it can't float past the edge-on rim, and slid by the
/// roll so the whole field rides with the tape. `toward_rim` is the screen-y
/// sign of the stop (−1 above the head, +1 below).
fn hazard(
    painter: &egui::Painter,
    spool: Spool,
    tape_w: f32,
    lane: i32,
    roll: f32,
    toward_rim: f32,
) {
    let pitch = spool.pitch();
    let last_valid = lane as f32 - toward_rim; // the printed year the hatch trails
    let t_edge = (last_valid + roll + toward_rim * HAZARD_GAP) * pitch;
    let t_rim = toward_rim * spool.rim();
    let (t_lo, t_hi) = (t_edge.min(t_rim), t_edge.max(t_rim));
    if t_hi - t_lo < 1.0 {
        return;
    }
    let half = tape_w * 0.5;
    let period = (spool.a * 0.175).max(5.0);
    let ink = egui::Color32::from_rgba_unmultiplied(214, 162, 78, 184);
    let c0 = roll * pitch; // rides the field with the tape
    let k_lo = ((t_lo - half - c0) / period).floor() as i32;
    let k_hi = ((t_hi + half - c0) / period).ceil() as i32;
    for k in k_lo..=k_hi {
        let c = c0 + k as f32 * period; // this stripe: across + arc = c
        let a_start = t_lo.max(c - half);
        let a_end = t_hi.min(c + half);
        if a_end - a_start < 1.0 {
            continue;
        }
        let steps = ((a_end - a_start) / 3.0).ceil().max(1.0) as usize;
        let pts: Vec<egui::Pos2> = (0..=steps)
            .map(|i| {
                let a = a_start + (a_end - a_start) * i as f32 / steps as f32;
                let s = spool.sample(a);
                egui::pos2(spool.cx + (c - a) * s.fold, s.y)
            })
            .collect();
        let _stripe = painter.add(egui::Shape::line(pts, egui::Stroke::new(1.4_f32, ink)));
    }
}

// --- Geometry helpers --------------------------------------------------------

#[derive(Clone, Copy, Debug)]
struct ReelLayout {
    visible: DateReels,
    windows: [egui::Rect; 3],
}

impl ReelLayout {
    fn new(rect: egui::Rect, visible: DateReels) -> Self {
        let count = visible.reel_count() as f32;
        let gutters = count - 1.0;
        let fitting_tiles = ((rect.width() / TILE - OUTER_TILES - gutters) / count).floor();
        let aperture_tiles = fitting_tiles.clamp(MIN_RECESS_TILES, RECESS_TILES);
        let aperture_width = aperture_tiles * TILE;
        let span = count * aperture_width + gutters * TILE;
        let free_tiles = (rect.width() - span) / TILE;
        let left_tiles = (free_tiles * 0.5 + 1e-4).floor().max(1.0);
        let mut cursor = rect.left() + left_tiles * TILE;
        let inner = rect.shrink(3.0);
        let mut windows = [egui::Rect::NOTHING; 3];

        for reel in visible.iter() {
            windows[reel.index()] = egui::Rect::from_min_size(
                egui::pos2(cursor, inner.top()),
                egui::vec2(aperture_width, inner.height()),
            );
            cursor += aperture_width + TILE;
        }

        Self { visible, windows }
    }

    fn iter(self) -> impl Iterator<Item = (Reel, egui::Rect)> {
        self.visible
            .iter()
            .map(move |reel| (reel, self.windows[reel.index()]))
    }

    fn required(self, reel: Reel) -> egui::Rect {
        assert!(
            self.visible.contains_reel(reel),
            "a wake cannot originate from an absent date reel"
        );
        self.windows[reel.index()]
    }

    fn at(self, pos: egui::Pos2) -> Option<Reel> {
        self.iter()
            .find_map(|(reel, rect)| rect.expand(3.0).contains(pos).then_some(reel))
    }
}

// --- Value mapping -----------------------------------------------------------

/// The label printed at `offset` lanes from the head, or `None` where the year
/// reel runs off its admissible range (a stop, not a number).
fn label(reel: Reel, parts: Parts, years: YearSpan, offset: i32) -> Option<String> {
    match reel {
        Reel::Year => {
            let year = parts.year + offset;
            (years.lo..=years.hi)
                .contains(&year)
                .then(|| format!("{year:04}"))
        }
        Reel::Month => Some(MONTHS[wrap(parts.month as i32 - 1 + offset, 12) as usize].to_owned()),
        Reel::Day => {
            let days = days_in_month(parts.year, parts.month) as i32;
            Some(format!(
                "{:02}",
                wrap(parts.day as i32 - 1 + offset, days) + 1
            ))
        }
    }
}

impl Parts {
    fn spin(&mut self, reel: Reel, dir: i32, years: YearSpan) -> bool {
        match reel {
            Reel::Year => {
                let next = self.year + dir;
                if !(years.lo..=years.hi).contains(&next) {
                    return false;
                }
                self.year = next;
                self.clamp_day();
            }
            Reel::Month => {
                self.month = (wrap(self.month as i32 - 1 + dir, 12) + 1) as u32;
                self.clamp_day();
            }
            Reel::Day => {
                let days = days_in_month(self.year, self.month) as i32;
                self.day = (wrap(self.day as i32 - 1 + dir, days) + 1) as u32;
            }
        }
        true
    }

    fn clamp_day(&mut self) {
        self.day = self.day.min(days_in_month(self.year, self.month));
    }
}

impl From<(i32, u32, u32)> for Parts {
    fn from((year, month, day): (i32, u32, u32)) -> Self {
        Self { year, month, day }
    }
}

fn leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn wrap(value: i32, modulus: i32) -> i32 {
    value.rem_euclid(modulus.max(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    const YEARS: YearSpan = YearSpan { lo: 2005, hi: 2026 };

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Day(i32, u32, u32);

    impl GregorianDay for Day {
        fn ymd(self) -> (i32, u32, u32) {
            (self.0, self.1, self.2)
        }

        fn from_ymd(year: i32, month: u32, day: u32) -> Self {
            Self(year, month, day)
        }
    }

    #[test]
    fn hidden_year_remains_the_month_day_banks_leap_context() {
        let ctx = egui::Context::default();
        let mut day = Day(1904, 2, 29);
        ctx.run_ui(egui::RawInput::default(), |ui| {
            let _spool = DateSpool::new(&mut day, 2005..=2027)
                .reels(DateReels::MONTH_DAY)
                .width(DateReels::MONTH_DAY.minimum_width())
                .show(ui, "hidden-year");
        })
        .drop_without_applying_deltas();

        assert_eq!(day.ymd(), (1904, 2, 29));
    }

    #[test]
    fn circular_month_clamps_day() {
        let mut parts = Parts {
            year: 2024,
            month: 3,
            day: 31,
        };
        assert!(parts.spin(Reel::Month, -1, YEARS));
        assert_eq!((parts.year, parts.month, parts.day), (2024, 2, 29));
    }

    #[test]
    fn circular_day_wraps_inside_month() {
        let mut parts = Parts {
            year: 2025,
            month: 4,
            day: 30,
        };
        assert!(parts.spin(Reel::Day, 1, YEARS));
        assert_eq!(parts.day, 1);
    }
}
