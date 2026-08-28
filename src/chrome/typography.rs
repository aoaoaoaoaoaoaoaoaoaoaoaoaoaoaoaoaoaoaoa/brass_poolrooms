//! Semantic typography for Poolrooms interfaces.

#![deny(missing_docs)]

use egui::{FontFamily, FontId, RichText, TextStyle};

/// One semantic rung in the Poolrooms application type scale.
///
/// The role, rather than a caller-selected point size, owns the metric. Use
/// [`TypeRole::text`] for ordinary egui text and one of the font constructors
/// only when painting text directly. Physical inscriptions forged into a
/// mechanism remain governed by that mechanism's gauge instead.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TypeRole {
    /// Nonessential spatial instrumentation, such as a map or plot annotation.
    ///
    /// Instrument text must never carry an action, fault, instruction, or the
    /// only statement of a user-visible fact.
    Instrument,
    /// Terse metadata, legends, and compact overlines.
    Caption,
    /// Secondary prose and status that remains comfortably readable.
    Supporting,
    /// Ordinary labels, prose, values, and control text.
    Body,
    /// A heading within the current application surface.
    Heading,
    /// The title of an application or substantial transient pane.
    Title,
}

impl TypeRole {
    /// Complete semantic scale from least to most prominent.
    pub const ALL: [Self; 6] = [
        Self::Instrument,
        Self::Caption,
        Self::Supporting,
        Self::Body,
        Self::Heading,
        Self::Title,
    ];

    /// Stable role name for galleries and instrumentation.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Instrument => "INSTRUMENT",
            Self::Caption => "CAPTION",
            Self::Supporting => "SUPPORTING",
            Self::Body => "BODY",
            Self::Heading => "HEADING",
            Self::Title => "TITLE",
        }
    }

    /// Construct text at this role's canonical metric.
    pub fn text(self, text: impl Into<String>) -> RichText {
        RichText::new(text).font(self.proportional())
    }

    /// Construct a proportional font at this role's canonical metric.
    #[allow(
        clippy::disallowed_methods,
        reason = "TypeRole is the sole owner of application font metrics"
    )]
    pub fn proportional(self) -> FontId {
        FontId::new(self.points(), FontFamily::Proportional)
    }

    /// Construct a monospace font at this role's canonical metric.
    #[allow(
        clippy::disallowed_methods,
        reason = "TypeRole is the sole owner of application font metrics"
    )]
    pub fn monospace(self) -> FontId {
        FontId::new(self.points(), FontFamily::Monospace)
    }

    pub(super) const fn points(self) -> f32 {
        match self {
            Self::Instrument => 10.5,
            Self::Caption => 12.0,
            Self::Supporting => 13.0,
            Self::Body => 14.0,
            Self::Heading => 15.0,
            Self::Title => 18.0,
        }
    }
}

pub(super) fn install(style: &mut egui::Style) {
    let _small = style
        .text_styles
        .insert(TextStyle::Small, TypeRole::Caption.proportional());
    let _body = style
        .text_styles
        .insert(TextStyle::Body, TypeRole::Body.proportional());
    let _button = style
        .text_styles
        .insert(TextStyle::Button, TypeRole::Body.proportional());
    let _heading = style
        .text_styles
        .insert(TextStyle::Heading, TypeRole::Title.proportional());
    let _monospace = style
        .text_styles
        .insert(TextStyle::Monospace, TypeRole::Body.monospace());
}
