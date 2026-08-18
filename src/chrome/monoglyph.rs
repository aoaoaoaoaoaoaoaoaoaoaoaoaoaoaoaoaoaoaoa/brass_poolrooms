//! A momentary square plunger whose one engraved glyph is its entire label.
//! Crown, bevel, skirt, projection, illumination, and directional shadow are
//! compiled from a three-dimensional foundry model; runtime selects a pose and
//! integrates the stiff return spring.

#![deny(missing_docs)]

use std::ops::Deref;

use egui::{CursorIcon, FontId, Pos2, Rect, Sense, Vec2, WidgetInfo, WidgetType};

use super::{MechanismSize, Symbol, foundry};

use super::mechanism::{CouplingPorts, CouplingTarget, sealed};
use super::plunger::{self, BakedGauge, BakedMesh, BakedPose, BakedVertex, PlungerWake, SpringLaw};

const ETCH_EM_PER_CROWN: f32 = 13.5 / (8.9 * 2.0);
const BRIGHT_CUT_DEPTH: f32 = 0.72;
const FLAT_CUT_DEPTH: f32 = 0.96;
const SPRING_LAW: SpringLaw = SpringLaw {
    stiffness: 2_400.0,
    damping: 68.0,
    restitution: 0.12,
    floor: baked::POSE_MIN,
    ceiling: baked::POSE_MAX,
};

mod baked {
    use super::{BakedGauge, BakedMesh, BakedPose, BakedVertex};

    include!(concat!(env!("OUT_DIR"), "/monoglyph_atlas.rs"));
}

/// Material and cutter treatment applied to a monoglyph's mark.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MonoglyphFinish {
    /// A shallow action cut dominated by its illuminated fresh-bronze wall.
    #[default]
    BrightCut,
    /// A steep, flat-bottomed engraving whose floor is soot black.
    Void,
    /// A steep, flat-bottomed engraving filled with rough blood-ochre paint.
    Danger,
    /// A steep, flat-bottomed engraving filled with rough deep-pink paint.
    Love,
}

impl MonoglyphFinish {
    /// Complete finish register in stable gallery order.
    pub const ALL: [Self; 4] = [Self::BrightCut, Self::Void, Self::Danger, Self::Love];

    /// Stable material name for galleries and instrumentation.
    pub const fn name(self) -> &'static str {
        match self {
            Self::BrightCut => "BRIGHT CUT",
            Self::Void => "VOID",
            Self::Danger => "DANGER",
            Self::Love => "LOVE",
        }
    }

    const fn depth(self) -> f32 {
        match self {
            Self::BrightCut => BRIGHT_CUT_DEPTH,
            Self::Void | Self::Danger | Self::Love => FLAT_CUT_DEPTH,
        }
    }
}

/// A square, momentary Poolrooms button carrying exactly one engraved glyph.
///
/// The `char` constructor makes the one-glyph boundary structural: text labels
/// and rectangular actions cannot accidentally enter this mechanism.
/// [`Monoglyph::symbol`] selects common action marks from the typed Poolrooms
/// armory so their Unicode scalar and S/M/L typography cannot drift between
/// applications. Pointer pressure plunges the flat crown into its black
/// socket; release excites a stiff underdamped spring that makes one small
/// return bounce.
/// [`Monoglyph::size`] selects one of the exact gauges admitted by
/// [`MechanismSize`].
///
/// # Example
///
/// ```
/// use brass_poolrooms::{chrome::{Monoglyph, Symbol}, egui};
///
/// fn decrement(ui: &mut egui::Ui) -> bool {
///     Monoglyph::symbol(Symbol::Decrement).show(ui).clicked()
/// }
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Monoglyph {
    glyph: char,
    size: MechanismSize,
    finish: MonoglyphFinish,
    symbol: Option<Symbol>,
}

impl Monoglyph {
    /// Forge a momentary button around one Unicode scalar.
    pub const fn new(glyph: char) -> Self {
        Self {
            glyph,
            size: MechanismSize::Large,
            finish: MonoglyphFinish::BrightCut,
            symbol: None,
        }
    }

    /// Forge one canonical action mark from the shared symbology armory.
    ///
    /// The selected [`MechanismSize`] remains the sole typographic gauge:
    /// equal symbols at equal sizes therefore have identical glyph, font,
    /// crown, relief, and motion. The symbol's semantic finish default is
    /// selected from the armory's closed lookup table.
    pub const fn symbol(symbol: Symbol) -> Self {
        Self {
            glyph: symbol.glyph(),
            size: MechanismSize::Large,
            finish: symbol.default_finish(),
            symbol: Some(symbol),
        }
    }

    /// Select a build-time forged square footprint.
    pub const fn size(mut self, size: MechanismSize) -> Self {
        self.size = size;
        self
    }

    /// Override the raw-glyph or semantic-symbol finish.
    ///
    /// This is deliberately applied after [`Monoglyph::symbol`] resolves the
    /// armory default, so a destructive symbol may be rendered in another
    /// material when its local meaning demands it.
    pub const fn finish(mut self, finish: MonoglyphFinish) -> Self {
        self.finish = finish;
        self
    }

    /// Lay out, actuate, and paint the complete square mechanism.
    ///
    /// The response dereferences to [`egui::Response`] and carries the signed
    /// volume swept by the button during this frame. Pass it to
    /// `water::Surface::monoglyph` during the same UI pass to couple the plunge
    /// and return stroke into the active water world.
    pub fn show(self, ui: &mut egui::Ui) -> MonoglyphResponse {
        let atlas = self.size.atlas_index();
        let gauge = baked::GAUGES[atlas];
        let law = foundry::law::momentary_gauge(gauge.side);
        debug_assert_eq!(gauge.side, self.size.side() as u8);
        debug_assert_eq!(gauge.socket_half, law.socket_half);
        debug_assert_eq!(gauge.top_half, law.top_half);
        debug_assert_eq!(gauge.body_half, law.body_half);
        let (rect, mut response) =
            ui.allocate_exact_size(Vec2::splat(self.size.side()), Sense::click());
        let enabled = ui.is_enabled();
        if enabled {
            response = response.on_hover_cursor(CursorIcon::PointingHand);
        }
        response.widget_info(|| {
            WidgetInfo::labeled(
                WidgetType::Button,
                enabled,
                self.symbol
                    .map_or_else(|| self.glyph.to_string(), |symbol| symbol.name().to_owned()),
            )
        });
        let activated = super::exact_activation(ui, &response);

        let motion = plunger::momentary_motion(
            ui,
            &response,
            enabled,
            activated,
            baked::REST,
            baked::PRESS,
            SPRING_LAW,
        );
        let anatomy = plunger::MomentaryAnatomy::new(
            rect,
            self.size.side(),
            gauge.socket_half,
            gauge.body_half,
        );
        let mut painter = ui.painter().clone();
        if !enabled {
            painter.set_opacity(1.0);
        }
        plunger::paint_momentary(
            ui,
            &painter,
            anatomy,
            motion.position,
            &response,
            atlas,
            gauge.poses,
            baked::POSE_MIN,
            baked::POSE_MAX,
            |painter, aperture, origin| {
                etch(
                    painter,
                    aperture,
                    origin,
                    self.glyph,
                    self.finish,
                    motion.position,
                    gauge.top_half,
                );
            },
        );
        super::tension(ui, &response);

        MonoglyphResponse {
            wake: MonoglyphWake::new(anatomy.button, motion.travel),
            response,
            elevation: motion.position,
            ports: CouplingPorts::around(anatomy.socket),
            activated,
        }
    }
}

#[must_use = "the response carries both egui state and displaced-water volume"]
/// Interaction state and displaced-water geometry from one [`Monoglyph`] frame.
pub struct MonoglyphResponse {
    response: egui::Response,
    wake: Option<MonoglyphWake>,
    elevation: f32,
    ports: CouplingPorts,
    activated: bool,
}

impl MonoglyphResponse {
    /// Whether pointer, accessibility, or exact keyboard activation fired it.
    pub const fn clicked(&self) -> bool {
        self.activated
    }

    /// The plunger volume swept since the preceding frame, if it moved.
    pub fn wake(&self) -> Option<MonoglyphWake> {
        self.wake
    }

    /// Current crown elevation normal to the faceplate, in logical points.
    pub fn elevation(&self) -> f32 {
        self.elevation
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

impl Deref for MonoglyphResponse {
    type Target = egui::Response;

    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

impl sealed::Sealed for MonoglyphResponse {}

impl CouplingTarget for MonoglyphResponse {
    fn coupling_ports(&self) -> CouplingPorts {
        self.ports
    }
}

/// Signed swept volume from a monoglyph plunger.
pub type MonoglyphWake = PlungerWake;

fn etch(
    painter: &egui::Painter,
    clip: Rect,
    origin: Pos2,
    glyph: char,
    finish: MonoglyphFinish,
    elevation: f32,
    top_half: f32,
) {
    let depth = finish.depth();
    let floor_scale = foundry::perspective_scale(elevation - depth);
    let font = FontId::monospace(top_half * 2.0 * ETCH_EM_PER_CROWN * floor_scale);
    let galley = painter.layout_no_wrap(glyph.to_string(), font, egui::Color32::PLACEHOLDER);
    let pos = origin - galley.size() * 0.5;
    match finish {
        MonoglyphFinish::BrightCut => {
            foundry::bright_cut_etch(painter, clip, pos, galley, elevation, depth);
        }
        MonoglyphFinish::Void => {
            foundry::flat_cut_etch(
                painter,
                clip,
                pos,
                galley,
                elevation,
                depth,
                foundry::EngravingFloor::Void,
            );
        }
        MonoglyphFinish::Danger => {
            foundry::flat_cut_etch(
                painter,
                clip,
                pos,
                galley,
                elevation,
                depth,
                foundry::EngravingFloor::Danger(glyph as u32),
            );
        }
        MonoglyphFinish::Love => {
            foundry::flat_cut_etch(
                painter,
                clip,
                pos,
                galley,
                elevation,
                depth,
                foundry::EngravingFloor::Love(glyph as u32),
            );
        }
    }
}
