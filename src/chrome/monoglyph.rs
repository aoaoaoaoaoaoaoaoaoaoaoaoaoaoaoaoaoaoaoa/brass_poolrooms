//! A momentary square plunger whose one etched glyph is its entire label.
//! Crown, bevel, skirt, projection, illumination, and directional shadow are
//! compiled from a three-dimensional foundry model; runtime selects a pose and
//! integrates the stiff return spring.

#![deny(missing_docs)]

use std::ops::Deref;

use egui::{CursorIcon, FontId, Pos2, Rect, Sense, Vec2, WidgetInfo, WidgetType};

use super::{MechanismSize, foundry};

use super::mechanism::{CouplingPorts, CouplingTarget, sealed};
use super::plunger::{self, BakedGauge, BakedMesh, BakedPose, BakedVertex, PlungerWake, SpringLaw};

const ETCH_EM_PER_CROWN: f32 = 13.5 / (8.9 * 2.0);
const ETCH_DEPTH: f32 = 0.72;
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

/// A square, momentary Poolrooms button carrying exactly one etched glyph.
///
/// The `char` constructor makes the one-glyph boundary structural: text labels
/// and rectangular actions cannot accidentally enter this mechanism. Pointer
/// pressure plunges the flat crown into its black socket; release excites a
/// stiff underdamped spring that makes one small return bounce.
/// [`Monoglyph::size`] selects one of the exact gauges admitted by
/// [`MechanismSize`].
///
/// # Example
///
/// ```
/// use dwemer_poolrooms::{chrome::Monoglyph, egui};
///
/// fn decrement(ui: &mut egui::Ui) -> bool {
///     Monoglyph::new('−').show(ui).clicked()
/// }
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Monoglyph {
    glyph: char,
    size: MechanismSize,
}

impl Monoglyph {
    /// Forge a momentary button around one Unicode scalar.
    pub const fn new(glyph: char) -> Self {
        Self {
            glyph,
            size: MechanismSize::Large,
        }
    }

    /// Select a build-time forged square footprint.
    pub const fn size(mut self, size: MechanismSize) -> Self {
        self.size = size;
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
        response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, enabled, self.glyph));

        let motion = plunger::momentary_motion(
            ui,
            &response,
            enabled,
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
}

impl MonoglyphResponse {
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
    elevation: f32,
    top_half: f32,
) {
    let floor_scale = foundry::perspective_scale(elevation - ETCH_DEPTH);
    let font = FontId::monospace(top_half * 2.0 * ETCH_EM_PER_CROWN * floor_scale);
    let galley = painter.layout_no_wrap(glyph.to_string(), font, egui::Color32::PLACEHOLDER);
    let pos = origin - galley.size() * 0.5;
    foundry::bright_cut_etch(painter, clip, pos, galley, elevation, ETCH_DEPTH);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atlas_covers_the_complete_momentary_stroke() {
        for gauge in baked::GAUGES {
            assert_eq!(gauge.poses.len(), baked::POSE_COUNT);
            assert_eq!(gauge.poses[0].elevation, baked::POSE_MIN);
            assert_eq!(
                gauge.poses[baked::POSE_COUNT - 1].elevation,
                baked::POSE_MAX
            );
        }
        const {
            assert!(baked::PRESS < baked::REST);
            assert!(baked::POSE_MIN < baked::PRESS);
            assert!(baked::REST < baked::POSE_MAX);
        }
    }

    #[test]
    fn crown_uses_darkened_detent_register_not_polished_brass() {
        let register_ceiling =
            foundry::law::darkened_bronze_rgb(foundry::law::DARK_TONE_CEILING)[0];
        let register_body = foundry::law::darkened_bronze_rgb(0.60)[0];
        let brightest = baked::GAUGES
            .iter()
            .flat_map(|gauge| gauge.poses)
            .flat_map(|pose| pose.button.vertices)
            .flat_map(|vertex| vertex.color[..3].iter().copied())
            .max()
            .unwrap_or_default();
        assert!(
            brightest <= register_ceiling,
            "monoglyph crown escaped into polished brass at {brightest}"
        );
        assert!(brightest > register_body);
    }

    #[test]
    fn response_footprint_is_compulsorily_square() {
        let ctx = egui::Context::default();
        let mut actual = Vec2::ZERO;
        let _frame = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::splat(80.0))),
                ..egui::RawInput::default()
            },
            |ui| actual = Monoglyph::new('+').show(ui).rect.size(),
        );
        assert_eq!(actual, Vec2::splat(MechanismSize::Large.side()));
    }

    #[test]
    fn every_public_gauge_has_exact_geometry_and_layout() {
        assert_eq!(baked::GAUGES.len(), baked::GAUGE_COUNT);
        for size in MechanismSize::ALL {
            let side = size.side() as u8;
            let gauge = baked::GAUGES[size.atlas_index()];
            assert_eq!(gauge.side, side);
            let ctx = egui::Context::default();
            let mut actual = Vec2::ZERO;
            let _frame = ctx.run_ui(egui::RawInput::default(), |ui| {
                actual = Monoglyph::new('×').size(size).show(ui).rect.size();
            });
            assert_eq!(actual, Vec2::splat(f32::from(side)));
        }
    }

    #[test]
    fn pointer_pressure_plunges_and_release_recoils() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::splat(80.0));
        let mut center = Pos2::ZERO;
        let mut swept = 0.0;
        let input = |events| egui::RawInput {
            screen_rect: Some(screen),
            predicted_dt: 1.0 / 60.0,
            events,
            ..egui::RawInput::default()
        };
        let _prime = ctx.run_ui(input(Vec::new()), |ui| {
            let button = Monoglyph::new('+').show(ui);
            center = button.rect.center();
        });
        let drive = |events| {
            let mut sample = (baked::REST, 0.0);
            let _frame = ctx.run_ui(input(events), |ui| {
                let button = Monoglyph::new('+').show(ui);
                sample = (
                    button.elevation(),
                    button.wake().map_or(0.0, MonoglyphWake::swept_volume),
                );
            });
            sample
        };
        let (_, volume) = drive(vec![
            egui::Event::PointerMoved(center),
            egui::Event::PointerButton {
                pos: center,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ]);
        swept += volume;
        let mut pressed = baked::REST;
        for _ in 0..8 {
            let (z, volume) = drive(Vec::new());
            pressed = z;
            swept += volume;
        }
        assert!(
            pressed < baked::REST - 8.0,
            "button only reached z={pressed}"
        );

        let (z, volume) = drive(vec![egui::Event::PointerButton {
            pos: center,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }]);
        swept += volume;
        let mut apex = z;
        for _ in 0..18 {
            let (z, volume) = drive(Vec::new());
            swept += volume;
            apex = apex.max(z);
        }
        assert!(
            apex > baked::REST + 0.25,
            "return apex only reached z={apex}"
        );
        assert!(swept > 4_000.0, "plunger swept only {swept} point³");
    }
}
