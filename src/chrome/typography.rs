//! Semantic typography for Poolrooms interfaces.

#![deny(missing_docs)]

use egui::{FontFamily, FontId, RichText, TextStyle, WidgetText};

/// One semantic rung in the Poolrooms application type scale.
///
/// The role, rather than a caller-selected point size, owns the metric. Use
/// [`TypeRole::text`] for ordinary egui text and one of the font constructors
/// only when painting text directly. Physical inscriptions forged into a
/// mechanism remain governed by that mechanism's gauge instead.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TypeRole {
    /// Nonessential marks embedded in a map, plot, diagram, or instrument.
    ///
    /// Annotation text must never carry prose, an action, a fault, an
    /// instruction, or the only statement of a user-visible fact.
    Annotation,
    /// Terse attached identifiers, field labels, metadata, and legends.
    Label,
    /// Ordinary prose, controls, values, status, faults, and instructions.
    ///
    /// Substantive hover help remains body text: its transient container does
    /// not make prose less important or less demanding to read.
    Body,
    /// A heading within the current application surface.
    Heading,
    /// The title of an application or substantial transient pane.
    Title,
}

impl TypeRole {
    /// Complete semantic scale from least to most prominent.
    pub const ALL: [Self; 5] = [
        Self::Annotation,
        Self::Label,
        Self::Body,
        Self::Heading,
        Self::Title,
    ];

    /// Stable role name for galleries and instrumentation.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Annotation => "ANNOTATION",
            Self::Label => "LABEL",
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
        self.in_family(FontFamily::Proportional)
    }

    /// Construct a monospace font at this role's canonical metric.
    #[allow(
        clippy::disallowed_methods,
        reason = "TypeRole is the sole owner of application font metrics"
    )]
    pub fn monospace(self) -> FontId {
        self.in_family(FontFamily::Monospace)
    }

    /// Construct a font in a named family at this role's canonical metric.
    ///
    /// This is principally useful to font and rasterization judgment surfaces.
    /// Applications should ordinarily inherit the family installed by
    /// [`crate::chrome::install`].
    #[allow(
        clippy::disallowed_methods,
        reason = "TypeRole remains the sole owner of application font metrics"
    )]
    pub fn in_family(self, family: FontFamily) -> FontId {
        FontId::new(self.points(), family)
    }

    pub(super) const fn points(self) -> f32 {
        match self {
            Self::Annotation => 12.5,
            Self::Label => 14.5,
            Self::Body => 17.0,
            Self::Heading => 17.75,
            Self::Title => 21.5,
        }
    }
}

pub(super) fn label_hover_text(text: impl Into<WidgetText>) -> WidgetText {
    text.into().fallback_text_style(TextStyle::Small)
}

pub(super) fn install(style: &mut egui::Style) {
    let _small = style
        .text_styles
        .insert(TextStyle::Small, TypeRole::Label.proportional());
    let _body = style
        .text_styles
        .insert(TextStyle::Body, TypeRole::Body.proportional());
    let _button = style
        .text_styles
        .insert(TextStyle::Button, TypeRole::Body.proportional());
    let _heading = style
        .text_styles
        .insert(TextStyle::Heading, TypeRole::Heading.proportional());
    let _monospace = style
        .text_styles
        .insert(TextStyle::Monospace, TypeRole::Body.monospace());
}
