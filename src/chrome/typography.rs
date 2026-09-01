//! Semantic typography for Poolrooms interfaces.

#![deny(missing_docs)]

use egui::{Context, FontFamily, FontId, RichText, Style, TextStyle, WidgetText};

/// User-selected preset over the canonical Poolrooms type scale.
///
/// Scaling is applied while font identifiers and glyph atlases are constructed,
/// before text layout or rasterization. The three named percentages are stable
/// user-facing tiers backed by tabulated optical metrics, not a framebuffer
/// transform or an arithmetic progression.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FontScale {
    /// Compact desktop metrics exposed as the 100% tier.
    #[default]
    Standard,
    /// The established reference metrics exposed as the 125% tier.
    Large,
    /// A 125% enlargement over the reference metrics, exposed as the 150%
    /// tier and supported layout ceiling.
    ExtraLarge,
}

impl FontScale {
    /// Complete scale menu from least to most enlarged.
    pub const ALL: [Self; 3] = [Self::Standard, Self::Large, Self::ExtraLarge];

    /// Stable user-facing option label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Standard => "STANDARD · 100%",
            Self::Large => "LARGE · 125%",
            Self::ExtraLarge => "EXTRA LARGE · 150%",
        }
    }

    /// Stable user-facing tier number for persistence-neutral projections.
    pub const fn percentage(self) -> u16 {
        match self {
            Self::Standard => 100,
            Self::Large => 125,
            Self::ExtraLarge => 150,
        }
    }

    const fn reference_factor(self) -> f32 {
        match self {
            Self::Standard => 0.8,
            Self::Large => 1.0,
            Self::ExtraLarge => 1.25,
        }
    }
}

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
        RichText::new(text).text_style(self.style())
    }

    /// Resolve a proportional font through the current semantic scale.
    pub fn proportional(self, style: &Style) -> FontId {
        self.in_family(style, FontFamily::Proportional)
    }

    /// Resolve a monospace font through the current semantic scale.
    pub fn monospace(self, style: &Style) -> FontId {
        self.in_family(style, FontFamily::Monospace)
    }

    /// Construct a font in a named family at this role's canonical metric.
    ///
    /// This is principally useful to font and rasterization judgment surfaces.
    /// Applications should ordinarily inherit the family installed by
    /// [`crate::chrome::install`].
    pub fn in_family(self, style: &Style, family: FontFamily) -> FontId {
        let mut font = self.style().resolve(style);
        font.family = family;
        font
    }

    /// Named egui text style owned by this semantic role.
    pub fn style(self) -> TextStyle {
        TextStyle::Name(self.style_name().into())
    }

    const fn style_name(self) -> &'static str {
        match self {
            Self::Annotation => "poolrooms.annotation",
            Self::Label => "poolrooms.label",
            Self::Body => "poolrooms.body",
            Self::Heading => "poolrooms.heading",
            Self::Title => "poolrooms.title",
        }
    }

    const fn reference_points(self) -> f32 {
        match self {
            Self::Annotation => 12.5,
            Self::Label => 14.5,
            Self::Body => 17.0,
            Self::Heading => 17.75,
            Self::Title => 21.5,
        }
    }
}

/// Resolve a deliberately numerical spatial inscription through the active
/// font scale.
///
/// Prefer [`TypeRole`] for application text. This escape exists for map,
/// plot, diagram, and mechanism typography whose nominal metric is derived
/// from physical geometry rather than information hierarchy.
#[allow(
    clippy::disallowed_methods,
    reason = "spatial_font is the governed numerical escape and applies the active font scale"
)]
pub fn spatial_font(ctx: &Context, nominal_points: f32, family: FontFamily) -> FontId {
    let style = ctx.style_of(ctx.theme());
    spatial_font_in(&style, nominal_points, family)
}

/// Resolve a numerical spatial inscription against an explicit local style.
///
/// This is the local-style counterpart to [`spatial_font`].
#[allow(
    clippy::disallowed_methods,
    reason = "spatial_font_in is the governed numerical escape and applies the active font scale"
)]
pub fn spatial_font_in(style: &Style, nominal_points: f32, family: FontFamily) -> FontId {
    FontId::new(
        nominal_points * active_scale(style).reference_factor(),
        family,
    )
}

pub(super) fn label_hover_text(text: impl Into<WidgetText>) -> WidgetText {
    text.into().fallback_text_style(TextStyle::Small)
}

pub(super) fn install(style: &mut Style, scale: FontScale) {
    for role in TypeRole::ALL {
        let _semantic = style.text_styles.insert(
            role.style(),
            spatial_font_at(scale, role.reference_points(), FontFamily::Proportional),
        );
    }
    let _small = style
        .text_styles
        .insert(TextStyle::Small, TypeRole::Label.proportional(style));
    let _body = style
        .text_styles
        .insert(TextStyle::Body, TypeRole::Body.proportional(style));
    let _button = style
        .text_styles
        .insert(TextStyle::Button, TypeRole::Body.proportional(style));
    let _heading = style
        .text_styles
        .insert(TextStyle::Heading, TypeRole::Heading.proportional(style));
    let _monospace = style
        .text_styles
        .insert(TextStyle::Monospace, TypeRole::Body.monospace(style));
}

pub(super) fn active_scale(style: &Style) -> FontScale {
    let body = TypeRole::Body.style().resolve(style).size;
    FontScale::ALL
        .into_iter()
        .min_by(|left, right| {
            let left = (body - TypeRole::Body.reference_points() * left.reference_factor()).abs();
            let right = (body - TypeRole::Body.reference_points() * right.reference_factor()).abs();
            left.total_cmp(&right)
        })
        .unwrap_or_default()
}

#[allow(
    clippy::disallowed_methods,
    reason = "semantic installation is the sole owner of scaled application metrics"
)]
fn spatial_font_at(scale: FontScale, points: f32, family: FontFamily) -> FontId {
    FontId::new(points * scale.reference_factor(), family)
}
