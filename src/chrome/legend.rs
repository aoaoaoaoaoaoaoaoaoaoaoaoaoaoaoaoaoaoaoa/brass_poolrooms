//! Physical legends for mnemonic labels and keyboard chords.

#![deny(missing_docs)]

use std::borrow::Cow;

use egui::{Color32, FontId, Sense, Stroke, TextStyle, Vec2, WidgetInfo, WidgetType};

use super::{CONTROL, EDGE, EDGE_STRONG, HOT, MUTED, TEXT};

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
        let end = self.byte
            + self.label[self.byte..]
                .chars()
                .next()
                .map_or(0, char::len_utf8);
        let ordinary = egui::text::TextFormat {
            font_id: TextStyle::Button.resolve(ui.style()),
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
        job.into()
    }
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
    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mnemonic_marks_exactly_one_glyph() {
        let ctx = egui::Context::default();
        let mut marked = 0;
        let _output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let text = MnemonicText::new("Open archive", 'A').widget_text(ui);
            let egui::WidgetText::LayoutJob(job) = text else {
                unreachable!("mnemonic text must retain its sectioned layout")
            };
            marked = job
                .sections
                .iter()
                .filter(|section| !section.format.underline.is_empty())
                .count();
        });
        assert_eq!(marked, 1);
    }
}
