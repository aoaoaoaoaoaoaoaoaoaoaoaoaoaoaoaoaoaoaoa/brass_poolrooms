//! Semantic typography for Poolrooms interfaces.

#![deny(missing_docs)]

use egui::{Context, FontFamily, FontId, RichText, Style, TextStyle, WidgetText};

/// User-selected preset over the canonical Poolrooms type scale.
///
/// Scaling is applied while font identifiers and glyph atlases are constructed,
/// before text layout or rasterization. The three named percentages are stable
/// user-facing approximate tiers backed by tabulated optical metrics, not a
/// framebuffer transform or an arithmetic progression.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FontScale {
    /// Compact desktop metrics exposed as the 100% tier.
    #[default]
    Standard,
    /// Enlarged desktop metrics exposed as the 125% tier.
    Large,
    /// Maximum supported desktop metrics exposed as the 150% tier.
    ExtraLarge,
}

impl FontScale {
    /// Complete scale menu from least to most enlarged.
    pub const ALL: [Self; 3] = [Self::Standard, Self::Large, Self::ExtraLarge];

    /// Stable user-facing option label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Standard => "STANDARD · ≈100%",
            Self::Large => "LARGE · ≈125%",
            Self::ExtraLarge => "EXTRA LARGE · ≈150%",
        }
    }

    /// Stable nominal tier number for persistence-neutral projections.
    pub const fn percentage(self) -> u16 {
        match self {
            Self::Standard => 100,
            Self::Large => 125,
            Self::ExtraLarge => 150,
        }
    }

    const fn spatial_factor(self) -> f32 {
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

    /// Exact production metric for this role and user-facing scale tier.
    ///
    /// This is hidden from ordinary documentation because applications must
    /// choose semantic roles rather than numerical point sizes. It remains
    /// available to renderer judgment surfaces that must reproduce production
    /// without a private copy of the scale table.
    #[doc(hidden)]
    pub const fn production_points(self, scale: FontScale) -> f32 {
        match (scale, self) {
            (FontScale::Standard, Self::Annotation) => 10.10,
            (FontScale::Standard, Self::Label) => 12.05,
            (FontScale::Standard, Self::Body) => 12.40,
            (FontScale::Standard, Self::Heading) => 14.30,
            (FontScale::Standard, Self::Title) => 17.15,
            (FontScale::Large, Self::Annotation) => 12.30,
            (FontScale::Large, Self::Label) => 13.30,
            (FontScale::Large, Self::Body) => 16.60,
            (FontScale::Large, Self::Heading) => 17.70,
            (FontScale::Large, Self::Title) => 21.70,
            (FontScale::ExtraLarge, Self::Annotation) => 15.625,
            (FontScale::ExtraLarge, Self::Label) => 18.125,
            (FontScale::ExtraLarge, Self::Body) => 21.25,
            (FontScale::ExtraLarge, Self::Heading) => 22.1875,
            (FontScale::ExtraLarge, Self::Title) => 26.875,
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
    spatial_font_at(active_scale(style), nominal_points, family)
}

pub(super) fn label_hover_text(text: impl Into<WidgetText>) -> WidgetText {
    text.into().fallback_text_style(TextStyle::Small)
}

pub(super) fn install(style: &mut Style, scale: FontScale) {
    for role in TypeRole::ALL {
        let _semantic = style
            .text_styles
            .insert(role.style(), semantic_font_at(role, scale));
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
            let left = (body - TypeRole::Body.production_points(*left)).abs();
            let right = (body - TypeRole::Body.production_points(*right)).abs();
            left.total_cmp(&right)
        })
        .unwrap_or_default()
}

#[allow(
    clippy::disallowed_methods,
    reason = "semantic installation is the sole owner of scaled application metrics"
)]
fn spatial_font_at(scale: FontScale, points: f32, family: FontFamily) -> FontId {
    FontId::new(points * scale.spatial_factor(), family)
}

#[allow(
    clippy::disallowed_methods,
    reason = "semantic installation is the sole owner of tabulated application metrics"
)]
fn semantic_font_at(role: TypeRole, scale: FontScale) -> FontId {
    FontId::new(role.production_points(scale), FontFamily::Proportional)
}
