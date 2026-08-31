//! Recessed disclosure sections and their physical focus state.

#![deny(missing_docs)]

use std::borrow::Cow;

use egui::{RichText, Sense, Stroke, WidgetInfo, WidgetType};

use super::{EDGE, HOT, RAISED, SURFACE, section_title};

/// A recessed, collapsible Poolrooms section.
///
/// `active` is a physical indication only. [`Section::locked_out`] folds the
/// disclosure and installs the common Lockout Grille over its complete header.
/// Logical panel selection and keyboard traversal belong to the application
/// layer.
#[derive(Clone, Debug)]
pub struct Section {
    title: &'static str,
    default_open: bool,
    active: bool,
    lockout_reason: Option<Cow<'static, str>>,
}

impl Section {
    /// Forge a section with application-owned identity and contents.
    pub const fn new(title: &'static str) -> Self {
        Self {
            title,
            default_open: false,
            active: false,
            lockout_reason: None,
        }
    }

    /// Choose the fold state used before egui has persisted one.
    #[must_use]
    pub const fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }

    /// Illuminate the section as the logical panel currently receiving keys.
    #[must_use]
    pub const fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// Lock the folded header beneath a physical Lockout Grille.
    ///
    /// Lockout closes an open section, prevents every form of activation, and
    /// exposes `reason` when the pointer rests over the grille. Removing the
    /// lockout leaves the section folded; reopening remains an explicit act.
    #[must_use]
    pub fn locked_out(mut self, reason: impl Into<Cow<'static, str>>) -> Self {
        let reason = reason.into();
        assert!(
            !reason.trim().is_empty(),
            "a locked-out section requires a causal explanation"
        );
        self.lockout_reason = Some(reason);
        self
    }

    /// Lay out the disclosure and return its physical interaction witnesses.
    pub fn show(
        self,
        ui: &mut egui::Ui,
        id: impl egui::AsIdSalt,
        add: impl FnOnce(&mut egui::Ui),
    ) -> SectionResponse {
        let Self {
            title,
            default_open,
            active,
            lockout_reason,
        } = self;
        assert!(
            ui.is_enabled(),
            "a Section cannot inherit disabled UI state; use Section::locked_out(reason)"
        );
        let id = ui.make_persistent_id(id);
        let header_id = id.with("header");
        let rect_id = id.with("rect");
        let wake_id = id.with("fold-wake");
        let frame_nr = ui.ctx().cumulative_frame_nr();
        let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(),
            id,
            default_open,
        );
        let mut flux = None;
        let locked_out = lockout_reason.is_some();
        if locked_out && state.is_open() {
            state.set_open(false);
            flux = Some(FoldFlux::Close);
        }
        if locked_out {
            ui.memory_mut(|memory| memory.surrender_focus(header_id));
        }
        let mut header_response = None;
        let mut header_activated = false;
        let frame = egui::Frame::new()
            .fill(SURFACE)
            .stroke(Stroke::new(
                if active && !locked_out {
                    1.5_f32
                } else {
                    1.0_f32
                },
                if active && !locked_out { HOT } else { EDGE },
            ))
            .corner_radius(2)
            .inner_margin(egui::Margin::same(0))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                let header = egui::Frame::new()
                    .fill(RAISED)
                    .stroke(Stroke::new(1.0_f32, EDGE))
                    .inner_margin(egui::Margin::symmetric(8, 5))
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        let glyph = if state.is_open() { "▾" } else { "▸" };
                        let _row = ui.horizontal(|ui| {
                            let _glyph = ui.label(RichText::new(glyph).color(HOT).strong());
                            let _title = ui.label(section_title(title.to_ascii_uppercase()));
                        });
                    });
                let response = if let Some(reason) = lockout_reason.as_ref() {
                    let response = ui
                        .add_enabled_ui(false, |ui| {
                            ui.interact(header.response.rect, header_id, Sense::click())
                        })
                        .inner
                        .on_disabled_hover_text(reason.clone());
                    if response.contains_pointer() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::NotAllowed);
                    }
                    response
                } else {
                    ui.interact(header.response.rect, header_id, Sense::click())
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                };
                response.widget_info(|| {
                    WidgetInfo::selected(
                        WidgetType::CollapsingHeader,
                        response.enabled(),
                        state.is_open(),
                        title,
                    )
                });
                let activated = super::exact_activation(ui, &response);
                if activated {
                    flux = Some(if state.is_open() {
                        FoldFlux::Close
                    } else {
                        FoldFlux::Open
                    });
                    state.toggle(ui);
                }
                if response.has_focus() {
                    let _focus = ui.painter().rect_stroke(
                        header.response.rect.shrink(1.0),
                        1.0,
                        Stroke::new(1.0_f32, HOT.gamma_multiply(0.64)),
                        egui::StrokeKind::Inside,
                    );
                }
                if locked_out {
                    super::lockout_grille::paint(ui.painter(), header.response.rect);
                }
                header_response = Some(response);
                header_activated = activated;
                if state.is_open() {
                    let _body = egui::Frame::new()
                        .fill(SURFACE)
                        .inner_margin(egui::Margin::symmetric(9, 7))
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            add(ui);
                        });
                }
                state.store(ui.ctx());
                header.response
            });
        let rect = frame.response.rect;
        crate::poolroom_anchor!(ui, format!("recess:{title}"), frame.inner.rect);
        if flux.is_some() {
            ui.ctx().request_repaint();
        }
        let wake = ui.ctx().data_mut(|data| {
            let prior = data.get_temp::<egui::Rect>(rect_id);
            let _old = data.insert_temp(rect_id, rect);
            if let Some(flux) = flux {
                let height = match flux {
                    FoldFlux::Open => 0.0,
                    FoldFlux::Close => prior.map_or(rect.height(), |prior| prior.height()),
                };
                let _old = data.insert_temp(
                    wake_id,
                    Some(PendingFoldWake {
                        flux,
                        height,
                        born: frame_nr,
                    }),
                );
                return None;
            }
            let pending = data
                .get_temp::<Option<PendingFoldWake>>(wake_id)
                .flatten()?;
            if pending.born >= frame_nr {
                return None;
            }
            let _cleared = data.remove_temp::<Option<PendingFoldWake>>(wake_id);
            Some(FoldWake {
                rect: pending.rect(rect),
                flux: pending.flux,
            })
        });
        SectionResponse {
            wake,
            response: frame.response,
            header: header_response.unwrap_or(frame.inner),
            activated: header_activated,
            open: state.is_open(),
            locked_out,
        }
    }
}

/// Draw the legacy section surface without an active-panel indication.
///
/// New logical panel managers should use [`Section`] and inspect its
/// [`SectionResponse`].
pub fn section(
    ui: &mut egui::Ui,
    id: impl egui::AsIdSalt,
    title: &'static str,
    default_open: bool,
    add: impl FnOnce(&mut egui::Ui),
) -> Option<FoldWake> {
    Section::new(title)
        .default_open(default_open)
        .show(ui, id, add)
        .wake
}

/// Physical responses emitted by one [`Section`] pass.
#[derive(Debug)]
pub struct SectionResponse {
    /// Delayed water forcing caused by opening or closing the recess.
    pub wake: Option<FoldWake>,
    /// Response covering the complete section.
    pub response: egui::Response,
    /// Disclosure header response; focusable unless locked out.
    pub header: egui::Response,
    /// Whether the disclosure accepted a pointer, accessibility, or exact key activation.
    pub activated: bool,
    /// Whether the disclosure body is open after this pass.
    pub open: bool,
    /// Whether a Lockout Grille physically disables the folded header.
    pub locked_out: bool,
}

#[derive(Clone, Copy, Debug)]
struct PendingFoldWake {
    flux: FoldFlux,
    height: f32,
    born: u64,
}

impl PendingFoldWake {
    fn rect(self, settled: egui::Rect) -> egui::Rect {
        let height = self.height.max(settled.height());
        egui::Rect::from_min_size(settled.min, egui::vec2(settled.width(), height))
    }
}

/// Delayed displacement generated by changing a section's fold state.
#[derive(Clone, Copy, Debug)]
pub struct FoldWake {
    /// Recess geometry swept by the fold.
    pub rect: egui::Rect,
    /// Direction in which the section moved.
    pub flux: FoldFlux,
}

/// Direction of one section fold transition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FoldFlux {
    /// The recess opened.
    Open,
    /// The recess closed.
    Close,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lockout_folds_the_section_and_keeps_it_folded_after_release() {
        let ctx = egui::Context::default();
        crate::chrome::install(&ctx);
        let mut body_runs = 0;

        ctx.run_ui(egui::RawInput::default(), |ui| {
            let section = Section::new("FINDER")
                .default_open(true)
                .show(ui, "finder", |_| body_runs += 1);
            assert!(section.open);
            assert!(!section.locked_out);
            assert!(section.header.enabled());
        })
        .drop_without_applying_deltas();

        ctx.run_ui(egui::RawInput::default(), |ui| {
            let section = Section::new("FINDER")
                .default_open(true)
                .locked_out("Finish the trail edit first.")
                .show(ui, "finder", |_| body_runs += 1);
            assert!(!section.open);
            assert!(section.locked_out);
            assert!(!section.header.enabled());
            assert!(!section.activated);
        })
        .drop_without_applying_deltas();

        ctx.run_ui(egui::RawInput::default(), |ui| {
            let section = Section::new("FINDER")
                .default_open(true)
                .show(ui, "finder", |_| body_runs += 1);
            assert!(!section.open);
            assert!(!section.locked_out);
        })
        .drop_without_applying_deltas();

        assert_eq!(body_runs, 1);
    }
}
