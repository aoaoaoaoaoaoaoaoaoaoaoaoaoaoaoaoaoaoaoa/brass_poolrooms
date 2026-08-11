//! A corner-mounted close plunger with a die-sunk crossed trench.
//!
//! The control shares its socket, crown stock, stroke, spring, projection, and
//! bronze with [`super::Monoglyph`]. Its fixed mark is categorically different
//! from a dynamic glyph: the X is a deep three-dimensional negative relief,
//! projected and self-shadowed in the build-time foundry atlas.

#![deny(missing_docs)]

use std::{hash::Hash, ops::Deref};

use egui::{CursorIcon, Rect, Sense, Vec2, WidgetInfo, WidgetType};

use super::plunger::{self, BakedGauge, BakedMesh, BakedPose, BakedVertex, PlungerWake, SpringLaw};
use super::{MechanismSize, foundry};

const SPRING_LAW: SpringLaw = SpringLaw {
    stiffness: 2_400.0,
    damping: 68.0,
    restitution: 0.12,
    floor: baked::POSE_MIN,
    ceiling: baked::POSE_MAX,
};

mod baked {
    use super::{BakedGauge, BakedMesh, BakedPose, BakedVertex};

    include!(concat!(env!("OUT_DIR"), "/corner_close_atlas.rs"));
}

/// A momentary close plunger centered on a pane's top-right corner.
///
/// The pane corner passes exactly through the mechanism's center. Reserve
/// [`Self::headroom`] before laying out a floating pane so its parent `Ui`
/// owns the upward overhang; [`Self::show`] registers the complete square
/// mechanism and extends the parent's right edge for the other half. The
/// selected [`MechanismSize`] governs the complete modelled die, footprint,
/// interaction region, and displaced volume.
///
/// # Example
///
/// ```
/// use dwemer_poolrooms::{chrome::{self, CornerClose}, egui};
///
/// fn popup(ui: &mut egui::Ui) -> bool {
///     let close = CornerClose::new().size(chrome::MechanismSize::Small);
///     ui.add_space(close.headroom());
///     let margin = egui::Margin::symmetric(8, 6);
///     let pane = egui::Frame::new()
///         .fill(chrome::SURFACE)
///         .stroke(egui::Stroke::new(1.0, chrome::EDGE_STRONG))
///         .inner_margin(margin)
///         .show(ui, |ui| {
///             let _header = close.guarded_header(ui, margin, |ui| ui.label("INSPECTION"));
///             let _body = ui.label("full width resumes here");
///         });
///     close.show(ui, pane.response.rect, "inspection-close").clicked()
/// }
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CornerClose {
    size: MechanismSize,
}

impl CornerClose {
    /// Space above a pane occupied by the default large mechanism.
    ///
    /// New variable-gauge layouts should reserve [`Self::headroom`] from the
    /// configured value instead. This constant preserves the original large
    /// closure's layout contract.
    pub const HEADROOM: f32 = MechanismSize::Large.side() * 0.5;

    /// Forge the standard corner closure.
    pub const fn new() -> Self {
        Self {
            size: MechanismSize::Large,
        }
    }

    /// Select a build-time forged square footprint.
    pub const fn size(mut self, size: MechanismSize) -> Self {
        self.size = size;
        self
    }

    /// Space above the pane occupied by this mechanism's upper half.
    pub const fn headroom(self) -> f32 {
        self.size.side() * 0.5
    }

    /// Lay out the pane's first row around this closure's lower-left quarter.
    ///
    /// Egui text galleys wrap inside one constant-width rectangle; they do not
    /// flow around arbitrary floating exclusions. This composition makes the
    /// obstruction an ordinary trailing allocation in the one row it can
    /// intersect. The following pane rows therefore recover their full width.
    /// `pane_margin` must be the enclosing frame's actual inner margin.
    pub fn guarded_header<R>(
        self,
        ui: &mut egui::Ui,
        pane_margin: egui::Margin,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> egui::InnerResponse<R> {
        let intrusion = self.pane_intrusion(pane_margin);
        ui.horizontal(|ui| {
            let item_spacing = ui.spacing().item_spacing;
            ui.spacing_mut().item_spacing.x = 0.0;
            let inner = ui
                .scope(|ui| {
                    ui.spacing_mut().item_spacing = item_spacing;
                    add_contents(ui)
                })
                .inner;
            if intrusion != Vec2::ZERO {
                let _guard = ui.allocate_exact_size(intrusion, Sense::hover());
            }
            inner
        })
    }

    /// Portion of the closure lying inside a frame's content rectangle.
    pub fn pane_intrusion(self, pane_margin: egui::Margin) -> Vec2 {
        let half = self.headroom();
        Vec2::new(
            (half - f32::from(pane_margin.right)).max(0.0),
            (half - f32::from(pane_margin.top)).max(0.0),
        )
    }

    /// Actuate and paint the mechanism on `pane`'s top-right corner.
    ///
    /// `id_salt` must identify the pane stably when sibling panes may appear or
    /// disappear. The response dereferences to [`egui::Response`] and carries
    /// the signed swept volume for `water::Surface::corner_close`.
    pub fn show(self, ui: &mut egui::Ui, pane: Rect, id_salt: impl Hash) -> CornerCloseResponse {
        let atlas = self.size.atlas_index();
        let gauge = baked::GAUGES[atlas];
        let law = foundry::law::momentary_gauge(gauge.side);
        debug_assert_eq!(gauge.side, self.size.side() as u8);
        debug_assert_eq!(gauge.socket_half, law.socket_half);
        debug_assert_eq!(gauge.top_half, law.top_half);
        debug_assert_eq!(gauge.body_half, law.body_half);
        let rect = Rect::from_center_size(pane.right_top(), Vec2::splat(self.size.side()));
        ui.expand_to_include_rect(rect);
        let id = ui.make_persistent_id(("poolrooms-corner-close", id_salt));
        let mut response = ui.interact(rect, id, Sense::click());
        let enabled = ui.is_enabled();
        if enabled {
            response = response.on_hover_cursor(CursorIcon::PointingHand);
        }
        response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, enabled, "Close"));
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
            |_, _, _| {},
        );
        super::tension(ui, &response);

        CornerCloseResponse {
            wake: CornerCloseWake::new(anatomy.button, motion.travel),
            response,
            elevation: motion.position,
            activated,
        }
    }
}

#[must_use = "the response carries both egui state and displaced-water volume"]
/// Interaction state and displaced-water geometry from one [`CornerClose`] frame.
pub struct CornerCloseResponse {
    response: egui::Response,
    wake: Option<CornerCloseWake>,
    elevation: f32,
    activated: bool,
}

impl CornerCloseResponse {
    /// Whether pointer, accessibility, or exact keyboard activation fired it.
    pub const fn clicked(&self) -> bool {
        self.activated
    }

    /// The plunger volume swept since the preceding frame, if it moved.
    pub fn wake(&self) -> Option<CornerCloseWake> {
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

impl Deref for CornerCloseResponse {
    type Target = egui::Response;

    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

/// Signed swept volume from a corner-close plunger.
pub type CornerCloseWake = PlungerWake;

#[cfg(test)]
mod tests {
    use egui::{Pos2, Vec2};

    use super::*;

    #[test]
    fn pane_corner_bisects_the_compulsory_square_footprint() {
        let ctx = egui::Context::default();
        let pane = Rect::from_min_size(Pos2::new(20.0, 30.0), Vec2::new(120.0, 70.0));
        let mut actual = Rect::NOTHING;
        let _frame = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::splat(240.0))),
                ..egui::RawInput::default()
            },
            |ui| actual = CornerClose::new().show(ui, pane, "close").rect,
        );
        assert_eq!(actual.center(), pane.right_top());
        assert_eq!(actual.size(), Vec2::splat(MechanismSize::Large.side()));
        assert_eq!(pane.right() - actual.left(), CornerClose::HEADROOM);
        assert_eq!(pane.top() - actual.top(), CornerClose::HEADROOM);
    }

    #[test]
    fn every_gauge_bisects_the_pane_corner_and_selects_its_own_atlas() {
        let ctx = egui::Context::default();
        let pane = Rect::from_min_size(Pos2::new(20.0, 30.0), Vec2::new(120.0, 70.0));
        for size in MechanismSize::ALL {
            let mut actual = Rect::NOTHING;
            let _frame = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::splat(240.0))),
                    ..egui::RawInput::default()
                },
                |ui| actual = CornerClose::new().size(size).show(ui, pane, size).rect,
            );
            assert_eq!(actual.center(), pane.right_top());
            assert_eq!(actual.size(), Vec2::splat(size.side()));
            assert_eq!(pane.top() - actual.top(), size.side() * 0.5);
            let gauge = baked::GAUGES[size.atlas_index()];
            assert_eq!(f32::from(gauge.side), size.side());
            assert_eq!(gauge.poses.len(), baked::POSE_COUNT);
        }
    }

    #[test]
    fn guarded_header_reserves_only_the_margin_intrusion() {
        let close = CornerClose::new().size(MechanismSize::Small);
        let margin = egui::Margin::symmetric(8, 6);
        assert_eq!(close.pane_intrusion(margin), Vec2::new(2.0, 4.0));

        let ctx = egui::Context::default();
        let mut header = Rect::NOTHING;
        let mut body = Rect::NOTHING;
        let _frame = ctx.run_ui(egui::RawInput::default(), |ui| {
            header = close
                .guarded_header(ui, margin, |ui| {
                    ui.allocate_exact_size(Vec2::new(50.0, 2.0), Sense::hover())
                })
                .response
                .rect;
            body = ui
                .allocate_exact_size(Vec2::new(80.0, 2.0), Sense::hover())
                .0;
        });
        assert_eq!(header.width(), 52.0);
        assert!(header.height() >= close.pane_intrusion(margin).y);
        assert_eq!(body.width(), 80.0);
    }

    #[test]
    fn reserved_headroom_keeps_a_floating_area_stationary_and_complete() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::splat(320.0));
        let origin = Pos2::new(40.0, 36.0);
        let mut prior = None;
        for pass in 0..4 {
            let mut area_rect = Rect::NOTHING;
            let mut close_rect = Rect::NOTHING;
            let _frame = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(screen),
                    ..egui::RawInput::default()
                },
                |_root| {
                    let area = egui::Area::new(egui::Id::new("popup"))
                        .fixed_pos(origin)
                        .show(&ctx, |ui| {
                            ui.add_space(CornerClose::HEADROOM);
                            let pane = egui::Frame::new().show(ui, |ui| {
                                ui.set_min_size(Vec2::new(120.0, 60.0));
                            });
                            close_rect = CornerClose::new()
                                .show(ui, pane.response.rect, "popup-close")
                                .rect;
                        });
                    area_rect = area.response.rect;
                },
            );
            if pass > 0 {
                assert_eq!(area_rect.left_top(), origin);
                assert!(area_rect.contains_rect(close_rect));
                if let Some(prior) = prior {
                    assert_eq!(area_rect, prior);
                }
                prior = Some(area_rect);
            }
        }
    }

    #[test]
    fn atlas_contains_the_modelled_relief_not_a_runtime_glyph() {
        assert_eq!(baked::GAUGES.len(), baked::GAUGE_COUNT);
        for gauge in baked::GAUGES {
            assert_eq!(gauge.poses.len(), baked::POSE_COUNT);
            assert!(gauge.poses[0].button.vertices.len() > 500);
            assert_eq!(gauge.poses[0].elevation, baked::POSE_MIN);
            assert_eq!(
                gauge.poses[baked::POSE_COUNT - 1].elevation,
                baked::POSE_MAX
            );
        }
    }

    #[test]
    fn pointer_pressure_plunges_and_returns_a_wake() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::splat(160.0));
        let pane = Rect::from_min_size(Pos2::new(20.0, 30.0), Vec2::new(80.0, 70.0));
        let center = pane.right_top();
        let input = |events| egui::RawInput {
            screen_rect: Some(screen),
            predicted_dt: 1.0 / 60.0,
            events,
            ..egui::RawInput::default()
        };
        let _prime = ctx.run_ui(input(Vec::new()), |ui| {
            let _close = CornerClose::new().show(ui, pane, "close");
        });
        let mut swept = 0.0;
        let _press = ctx.run_ui(
            input(vec![
                egui::Event::PointerMoved(center),
                egui::Event::PointerButton {
                    pos: center,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ]),
            |ui| {
                let close = CornerClose::new().show(ui, pane, "close");
                swept += close.wake().map_or(0.0, CornerCloseWake::swept_volume);
            },
        );
        for _ in 0..8 {
            let _stroke = ctx.run_ui(input(Vec::new()), |ui| {
                let close = CornerClose::new().show(ui, pane, "close");
                swept += close.wake().map_or(0.0, CornerCloseWake::swept_volume);
            });
        }
        assert!(swept > 1_000.0, "close plunger swept only {swept} point³");
    }
}
