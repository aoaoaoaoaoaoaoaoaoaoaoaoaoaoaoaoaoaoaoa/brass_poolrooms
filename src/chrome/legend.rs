//! Physical legends for mnemonic labels and keyboard chords.

#![deny(missing_docs)]

use std::{borrow::Cow, sync::Arc};

use egui::{
    Atom, Button, Color32, FontId, Response, Sense, Stroke, TextStyle, Vec2, WidgetInfo, WidgetType,
};

use super::{CONTROL, EDGE, EDGE_STRONG, HOT, MUTED, TEXT};

const MNEMONIC_UNDERLINE_LIFT: f32 = 2.0;
const INLINE_KEYCAP_FONT_SIZE: f32 = 9.0;
const INLINE_KEYCAP_PADDING: Vec2 = Vec2::new(3.0, 1.0);

/// Text carrying one permanently visible Alt mnemonic underline.
///
/// This type owns only the typographic mark. Dispatch of the corresponding
/// `Alt+key` command belongs to the application command layer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MnemonicText<'a> {
    label: Cow<'a, str>,
    key: char,
    byte: usize,
}

impl<'a> MnemonicText<'a> {
    /// Mark the first case-insensitive occurrence of `key` in `label`.
    ///
    /// # Panics
    ///
    /// Panics unless `key` is ASCII alphanumeric and occurs in `label`.
    pub fn new(label: impl Into<Cow<'a, str>>, key: char) -> Self {
        let label = label.into();
        assert!(
            key.is_ascii_alphanumeric(),
            "mnemonic keys must be ASCII alphanumeric"
        );
        let byte = label
            .char_indices()
            .find_map(|(byte, candidate)| candidate.eq_ignore_ascii_case(&key).then_some(byte));
        assert!(
            byte.is_some(),
            "mnemonic `{key}` does not occur in `{label}`"
        );
        let byte = byte.unwrap_or_default();
        Self { label, key, byte }
    }

    /// Unadorned label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// ASCII key named by the underline.
    pub const fn key(&self) -> char {
        self.key
    }

    /// Build a button-style egui label with only the mnemonic glyph underlined.
    pub fn widget_text(&self, ui: &egui::Ui) -> egui::WidgetText {
        self.widget_text_with_font(ui, TextStyle::Button.resolve(ui.style()))
    }

    /// Build an egui label in `font` with only the mnemonic glyph underlined.
    pub fn widget_text_with_font(&self, ui: &egui::Ui, font: FontId) -> egui::WidgetText {
        let end = self.byte
            + self.label[self.byte..]
                .chars()
                .next()
                .map_or(0, char::len_utf8);
        let ordinary = egui::text::TextFormat {
            font_id: font,
            color: Color32::PLACEHOLDER,
            valign: ui.text_valign(),
            ..Default::default()
        };
        let marked = egui::text::TextFormat {
            underline: Stroke::new(1.0_f32, HOT),
            ..ordinary.clone()
        };
        let mut job = egui::text::LayoutJob::default();
        job.append(&self.label[..self.byte], 0.0, ordinary.clone());
        job.append(&self.label[self.byte..end], 0.0, marked);
        job.append(&self.label[end..], 0.0, ordinary);
        let mut galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
        lift_mnemonic_underline(&mut galley);
        galley.into()
    }
}

fn lift_mnemonic_underline(galley: &mut Arc<egui::Galley>) {
    // egui exposes underline stroke but not underline offset. A text row emits
    // its ornaments after `glyph_vertex_range`, so the one requested underline
    // can be lifted without perturbing glyph geometry or layout.
    let galley = Arc::make_mut(galley);
    let mut bounds = egui::Rect::NOTHING;
    for placed in &mut galley.rows {
        let row = Arc::make_mut(&mut placed.row);
        let ornament = row.visuals.glyph_vertex_range.end..;
        for vertex in &mut row.visuals.mesh.vertices[ornament] {
            vertex.pos.y -= MNEMONIC_UNDERLINE_LIFT;
        }
        row.visuals.mesh_bounds = row.visuals.mesh.calc_bounds();
        bounds = bounds.union(row.visuals.mesh_bounds.translate(placed.pos.to_vec2()));
    }
    galley.mesh_bounds = bounds;
}

/// A compact physical plate naming one key or complete keyboard chord.
#[derive(Clone, Debug)]
pub struct Keycap {
    label: String,
}

impl Keycap {
    /// Forge a noninteractive chord plate.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
        }
    }

    /// Lay out and paint the chord plate.
    pub fn show(self, ui: &mut egui::Ui) -> Response {
        let enabled = ui.is_enabled();
        let ink = if enabled { TEXT } else { MUTED };
        let font = FontId::monospace(11.0);
        let galley = ui.painter().layout_no_wrap(self.label.clone(), font, ink);
        let padding = Vec2::new(6.0, 3.0);
        let (rect, response) =
            ui.allocate_exact_size(galley.size() + 2.0 * padding, Sense::hover());
        let painter = ui.painter();
        let _face = painter.rect_filled(rect, 1.0, CONTROL);
        let _edge = painter.rect_stroke(
            rect,
            1.0,
            Stroke::new(1.0_f32, if enabled { EDGE_STRONG } else { EDGE }),
            egui::StrokeKind::Inside,
        );
        painter.galley(rect.center() - galley.size() * 0.5, galley, ink);
        response
            .widget_info(|| WidgetInfo::labeled(WidgetType::Label, enabled, self.label.clone()));
        response
    }

    /// Embed this key legend at the trailing edge of a button.
    ///
    /// The compact key well remains part of the enclosing button's allocation
    /// and interaction surface. It conveys an accelerator without turning the
    /// command label itself into mutable shortcut notation.
    pub fn show_in(self, ui: &mut egui::Ui, button: Button<'_>) -> Response {
        let enabled = ui.is_enabled();
        let ink = if enabled { HOT } else { MUTED };
        let font = FontId::monospace(INLINE_KEYCAP_FONT_SIZE);
        let galley = ui.painter().layout_no_wrap(self.label, font, ink);
        let size = galley.size() + 2.0 * INLINE_KEYCAP_PADDING;
        let id = ui.next_auto_id().with("inline-keycap");
        let layout = button
            .right_text(Atom::custom(id, size))
            .gap(4.0)
            .atom_ui(ui);
        if let Some(rect) = layout.rect(id) {
            let painter = ui.painter();
            let _well = painter.rect_filled(rect, 1.0, CONTROL);
            let _edge = painter.rect_stroke(
                rect,
                1.0,
                Stroke::new(1.0_f32, EDGE),
                egui::StrokeKind::Inside,
            );
            painter.galley(rect.center() - galley.size() * 0.5, galley, ink);
        }
        layout.response
    }
}
