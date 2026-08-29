#![expect(
    unused_crate_dependencies,
    reason = "the atelier consumes egui and wgpu through the crate's version-locked re-exports"
)]

mod support;

use std::sync::Arc;

use anyhow::Result;
use brass_poolrooms::{
    chrome::{self, NumberInput, ScrewScroll, TypeRole},
    egui::{self, FontData, FontDefinitions, FontFamily, FontId, FontTweak, RichText},
    water::{Surface, Wetness},
};
use egui::epaint::{
    FontColorTransferFunction,
    text::{HintingTarget, SmoothHinting},
};
use support::Exhibit;

const CMU_REGULAR: &[u8] = include_bytes!("../assets/fonts/cmu-typewriter/cmuntt.ttf");
const CMU_EMPHASIS: &[u8] = include_bytes!("../assets/fonts/atelier/cmu-typewriter/cmuntb.otf");
const CM_GRADE_0: &[u8] = include_bytes!(
    "../assets/fonts/atelier/cm-graded/ComputerModernGradedG0Typewriter10Regular.otf"
);
const CM_GRADE_10: &[u8] = include_bytes!(
    "../assets/fonts/atelier/cm-graded/ComputerModernGradedG10Typewriter10Regular.otf"
);
const CM_GRADE_19: &[u8] = include_bytes!(
    "../assets/fonts/atelier/cm-graded/ComputerModernGradedG19Typewriter10Regular.otf"
);
const CM_GRADE_32: &[u8] = include_bytes!(
    "../assets/fonts/atelier/cm-graded/ComputerModernGradedG32Typewriter10Regular.otf"
);
const CM_GRADE_44: &[u8] = include_bytes!(
    "../assets/fonts/atelier/cm-graded/ComputerModernGradedG44Typewriter10Regular.otf"
);
const CM_GRADE_57: &[u8] = include_bytes!(
    "../assets/fonts/atelier/cm-graded/ComputerModernGradedG57Typewriter10Regular.otf"
);
const CM_GRADE_72: &[u8] = include_bytes!(
    "../assets/fonts/atelier/cm-graded/ComputerModernGradedG72Typewriter10Regular.otf"
);
const NOTO_MATH: &[u8] = include_bytes!("../assets/fonts/noto/NotoSansMath-Regular.ttf");
const NOTO_SYMBOLS: &[u8] = include_bytes!("../assets/fonts/noto/NotoSansSymbols2-Regular.ttf");

#[derive(Clone, Copy)]
struct Face {
    slug: &'static str,
    name: &'static str,
    province: &'static str,
    regular: &'static [u8],
    emphasis: &'static [u8],
}

const FACES: &[Face] = &[
    Face {
        slug: "cmu-typewriter",
        name: "CMU TYPEWRITER",
        province: "current production face",
        regular: CMU_REGULAR,
        emphasis: CMU_EMPHASIS,
    },
    Face {
        slug: "cmu-typewriter-light",
        name: "CMU TYPEWRITER LIGHT",
        province: "same CM Unicode family · light / bold",
        regular: include_bytes!("../assets/fonts/atelier/cmu-typewriter/cmunbtl.otf"),
        emphasis: CMU_EMPHASIS,
    },
    Face {
        slug: "cm-graded-0",
        name: "CM GRADED · G0",
        province: "exact CM metrics · grade 0",
        regular: CM_GRADE_0,
        emphasis: CM_GRADE_0,
    },
    Face {
        slug: "cm-graded-10",
        name: "CM GRADED · G10",
        province: "exact CM metrics · grade 10",
        regular: CM_GRADE_10,
        emphasis: CM_GRADE_10,
    },
    Face {
        slug: "cm-graded-19",
        name: "CM GRADED · G19",
        province: "package default · grade 19",
        regular: CM_GRADE_19,
        emphasis: CM_GRADE_19,
    },
    Face {
        slug: "cm-graded-32",
        name: "CM GRADED · G32",
        province: "exact CM metrics · grade 32",
        regular: CM_GRADE_32,
        emphasis: CM_GRADE_32,
    },
    Face {
        slug: "cm-graded-44",
        name: "CM GRADED · G44",
        province: "exact CM metrics · grade 44",
        regular: CM_GRADE_44,
        emphasis: CM_GRADE_44,
    },
    Face {
        slug: "cm-graded-57",
        name: "CM GRADED · G57",
        province: "exact CM metrics · grade 57",
        regular: CM_GRADE_57,
        emphasis: CM_GRADE_57,
    },
    Face {
        slug: "cm-graded-72",
        name: "CM GRADED · G72",
        province: "exact CM metrics · grade 72",
        regular: CM_GRADE_72,
        emphasis: CM_GRADE_72,
    },
    Face {
        slug: "latin-modern-mono-8",
        name: "LATIN MODERN MONO 8",
        province: "small optical master · direct CM",
        regular: include_bytes!("../assets/fonts/atelier/latin-modern/lmmono8-regular.otf"),
        emphasis: include_bytes!("../assets/fonts/atelier/latin-modern/lmmono8-regular.otf"),
    },
    Face {
        slug: "latin-modern-mono-9",
        name: "LATIN MODERN MONO 9",
        province: "small optical master · direct CM",
        regular: include_bytes!("../assets/fonts/atelier/latin-modern/lmmono9-regular.otf"),
        emphasis: include_bytes!("../assets/fonts/atelier/latin-modern/lmmono9-regular.otf"),
    },
    Face {
        slug: "latin-modern-mono-10",
        name: "LATIN MODERN MONO 10",
        province: "direct CM outline descendant",
        regular: include_bytes!("../assets/fonts/atelier/latin-modern/lmmono10-regular.otf"),
        emphasis: include_bytes!("../assets/fonts/atelier/latin-modern/lmmono10-regular.otf"),
    },
    Face {
        slug: "latin-modern-mono-12",
        name: "LATIN MODERN MONO 12",
        province: "larger CM optical master",
        regular: include_bytes!("../assets/fonts/atelier/latin-modern/lmmono12-regular.otf"),
        emphasis: include_bytes!("../assets/fonts/atelier/latin-modern/lmmono12-regular.otf"),
    },
    Face {
        slug: "latin-modern-mono-light-10",
        name: "LATIN MODERN MONO LIGHT",
        province: "CM light / bold pair",
        regular: include_bytes!("../assets/fonts/atelier/latin-modern/lmmonolt10-regular.otf"),
        emphasis: include_bytes!("../assets/fonts/atelier/latin-modern/lmmonolt10-bold.otf"),
    },
    Face {
        slug: "latin-modern-mono-light-condensed-10",
        name: "LATIN MODERN MONO CONDENSED",
        province: "CM light condensed master",
        regular: include_bytes!("../assets/fonts/atelier/latin-modern/lmmonoltcond10-regular.otf"),
        emphasis: include_bytes!("../assets/fonts/atelier/latin-modern/lmmonoltcond10-regular.otf"),
    },
    Face {
        slug: "new-cm-mono-10",
        name: "NEW CM MONO 10",
        province: "expanded CM regular / book",
        regular: include_bytes!(
            "../assets/fonts/atelier/new-computer-modern/NewCMMono10-Regular.otf"
        ),
        emphasis: include_bytes!(
            "../assets/fonts/atelier/new-computer-modern/NewCMMono10-Book.otf"
        ),
    },
    Face {
        slug: "courier-prime",
        name: "COURIER PRIME",
        province: "screenplay Courier redrawing",
        regular: include_bytes!("../assets/fonts/atelier/courier-prime/CourierPrime-Regular.ttf"),
        emphasis: include_bytes!("../assets/fonts/atelier/courier-prime/CourierPrime-Bold.ttf"),
    },
    Face {
        slug: "courier-prime-code",
        name: "COURIER PRIME CODE",
        province: "code-tuned Courier Prime",
        regular: include_bytes!(
            "../assets/fonts/atelier/courier-prime-code/CourierPrimeCode-Regular.ttf"
        ),
        emphasis: include_bytes!(
            "../assets/fonts/atelier/courier-prime-code/CourierPrimeCode-Regular.ttf"
        ),
    },
    Face {
        slug: "tex-gyre-cursor",
        name: "TEX GYRE CURSOR",
        province: "extended Nimbus / Courier",
        regular: include_bytes!(
            "../assets/fonts/atelier/tex-gyre-cursor/texgyrecursor-regular.otf"
        ),
        emphasis: include_bytes!("../assets/fonts/atelier/tex-gyre-cursor/texgyrecursor-bold.otf"),
    },
    Face {
        slug: "nimbus-mono-ps",
        name: "NIMBUS MONO PS",
        province: "URW Courier master",
        regular: include_bytes!("../assets/fonts/atelier/nimbus-mono/NimbusMonoPS-Regular.otf"),
        emphasis: include_bytes!("../assets/fonts/atelier/nimbus-mono/NimbusMonoPS-Bold.otf"),
    },
    Face {
        slug: "liberation-mono",
        name: "LIBERATION MONO",
        province: "Courier New metric branch",
        regular: include_bytes!(
            "../assets/fonts/atelier/liberation-mono/LiberationMono-Regular.ttf"
        ),
        emphasis: include_bytes!("../assets/fonts/atelier/liberation-mono/LiberationMono-Bold.ttf"),
    },
    Face {
        slug: "cousine",
        name: "COUSINE",
        province: "ChromeOS Courier metric branch",
        regular: include_bytes!("../assets/fonts/atelier/cousine/Cousine-Regular.ttf"),
        emphasis: include_bytes!("../assets/fonts/atelier/cousine/Cousine-Bold.ttf"),
    },
];

#[derive(Clone, Copy)]
struct RasterProfile {
    slug: &'static str,
    name: &'static str,
    province: &'static str,
    hinting: bool,
    target: HintingTarget,
    subpixel_binning: bool,
}

const SMOOTH_STABLE: HintingTarget = HintingTarget::Smooth(SmoothHinting {
    light: false,
    symmetric_rendering: true,
    preserve_linear_metrics: true,
});
const SMOOTH_GRID_FIT: HintingTarget = HintingTarget::Smooth(SmoothHinting {
    light: false,
    symmetric_rendering: false,
    preserve_linear_metrics: false,
});
const SMOOTH_ASYMMETRIC: HintingTarget = HintingTarget::Smooth(SmoothHinting {
    light: false,
    symmetric_rendering: false,
    preserve_linear_metrics: true,
});
const SMOOTH_LIGHT: HintingTarget = HintingTarget::Smooth(SmoothHinting {
    light: true,
    symmetric_rendering: true,
    preserve_linear_metrics: true,
});

const PROFILES: &[RasterProfile] = &[
    RasterProfile {
        slug: "production",
        name: "PRODUCTION",
        province: "stable hint · 4 phases",
        hinting: true,
        target: SMOOTH_STABLE,
        subpixel_binning: true,
    },
    RasterProfile {
        slug: "fixed-phase",
        name: "FIXED PHASE",
        province: "stable hint · no bins",
        hinting: true,
        target: SMOOTH_STABLE,
        subpixel_binning: false,
    },
    RasterProfile {
        slug: "grid-fit",
        name: "GRID FIT",
        province: "full x/y fit · no bins",
        hinting: true,
        target: SMOOTH_GRID_FIT,
        subpixel_binning: false,
    },
    RasterProfile {
        slug: "asymmetric",
        name: "ASYMMETRIC",
        province: "x shape fit · stable metrics",
        hinting: true,
        target: SMOOTH_ASYMMETRIC,
        subpixel_binning: false,
    },
    RasterProfile {
        slug: "light-hint",
        name: "LIGHT HINT",
        province: "vertical fit · no bins",
        hinting: true,
        target: SMOOTH_LIGHT,
        subpixel_binning: false,
    },
    RasterProfile {
        slug: "mono-hint",
        name: "MONO HINT",
        province: "hard grid target · no bins",
        hinting: true,
        target: HintingTarget::Mono,
        subpixel_binning: false,
    },
    RasterProfile {
        slug: "unhinted-phases",
        name: "UNHINTED 4×",
        province: "outline faithful · 4 phases",
        hinting: false,
        target: SMOOTH_STABLE,
        subpixel_binning: true,
    },
    RasterProfile {
        slug: "unhinted-fixed",
        name: "UNHINTED 1×",
        province: "outline faithful · no bins",
        hinting: false,
        target: SMOOTH_STABLE,
        subpixel_binning: false,
    },
];

impl RasterProfile {
    fn tweak(self) -> FontTweak {
        FontTweak {
            hinting: Some(self.hinting),
            hinting_target: self.target,
            subpixel_binning: Some(self.subpixel_binning),
            ..FontTweak::default()
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum Weight {
    #[default]
    Regular,
    Emphasis,
}

impl Weight {
    const fn name(self) -> &'static str {
        match self {
            Self::Regular => "REGULAR / TEXT",
            Self::Emphasis => "EMPHASIS / MEDIUM",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum Transfer {
    #[default]
    Dark,
    Gamma65,
    Gamma80,
    Raw,
}

impl Transfer {
    const ALL: [Self; 4] = [Self::Dark, Self::Gamma65, Self::Gamma80, Self::Raw];

    const fn name(self) -> &'static str {
        match self {
            Self::Dark => "DARK SHARP",
            Self::Gamma65 => "GAMMA 0.65",
            Self::Gamma80 => "GAMMA 0.80",
            Self::Raw => "RAW COVERAGE",
        }
    }

    const fn law(self) -> FontColorTransferFunction {
        match self {
            Self::Dark => FontColorTransferFunction::TwoCoverageMinusCoverageSq,
            Self::Gamma65 => FontColorTransferFunction::Gamma(0.65),
            Self::Gamma80 => FontColorTransferFunction::Gamma(0.80),
            Self::Raw => FontColorTransferFunction::Off,
        }
    }
}

#[derive(Clone, Copy)]
struct SpecimenScale {
    points: [f32; TypeRole::ALL.len()],
}

const PROSPECTIVE_SCALE: [f32; TypeRole::ALL.len()] = [12.5, 14.5, 17.0, 17.0, 17.75, 21.5];

impl Default for SpecimenScale {
    fn default() -> Self {
        Self {
            points: PROSPECTIVE_SCALE,
        }
    }
}

impl SpecimenScale {
    fn points(self, role: TypeRole) -> f32 {
        self.points[role_index(role)]
    }

    fn font(self, role: TypeRole, family: FontFamily) -> FontId {
        specimen_font(self.points(role), family)
    }
}

#[derive(Default)]
struct FontRasterAtelier {
    face: usize,
    profile: usize,
    weight: Weight,
    transfer: Transfer,
    wet: bool,
    scale: SpecimenScale,
}

impl Exhibit for FontRasterAtelier {
    const TITLE: &'static str = "Poolrooms · Font Raster Atelier";
    const SIZE: [f64; 2] = [1_280.0, 920.0];

    fn install(&self, ctx: &egui::Context) {
        install_fonts(ctx, self.weight);
        install_transfer(ctx, self.transfer);
    }

    fn ui(&mut self, ui: &mut egui::Ui, water: &mut Surface) {
        water.set_wetness(if self.wet { Wetness::Wet } else { Wetness::Dry });
        install_transfer(ui.ctx(), self.transfer);

        let _panel = egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(chrome::PAGE).inner_margin(22))
            .show(ui, |ui| {
                let _scroll = ScrewScroll::vertical()
                    .id_salt("font-raster-atelier")
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.set_max_width(1_220.0);
                        self.header(ui);
                        self.controls(ui);
                        ui.add_space(10.0);
                        self.target_selectors(ui);
                        ui.add_space(10.0);
                        self.focus_bench(ui, water);
                        ui.add_space(24.0);
                    });
            });
    }
}

impl FontRasterAtelier {
    fn header(&self, ui: &mut egui::Ui) {
        let _title = ui.label(TypeRole::Title.text("FONT RASTER ATELIER"));
        let _purpose = ui.label(TypeRole::Supporting.text(
            "Native egui glyph atlas → production tessellator → WGPU. Judge at actual size.",
        ));
        let _conditions = ui.label(TypeRole::Caption.text(format!(
            "{} PIXELS PER POINT · {} · {}",
            ui.pixels_per_point(),
            self.transfer.name(),
            if self.wet { "PRODUCTION WATER" } else { "DRY" },
        )));
        let _optics = ui.label(TypeRole::Instrument.text(
            "Dry isolates raster law. Production Water admits the live tooltip lift and composite.",
        ));
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        let before_weight = self.weight;
        let before_transfer = self.transfer;
        let _frame = egui::Frame::new()
            .fill(chrome::SURFACE)
            .stroke(egui::Stroke::new(1.0, chrome::EDGE_STRONG))
            .inner_margin(10)
            .show(ui, |ui| {
                let _controls = ui.horizontal_wrapped(|ui| {
                    let _weight_label = ui.label(TypeRole::Caption.text("WEIGHT"));
                    for weight in [Weight::Regular, Weight::Emphasis] {
                        let _response = ui.selectable_value(
                            &mut self.weight,
                            weight,
                            TypeRole::Caption.text(weight.name()),
                        );
                    }
                    let _separator = ui.separator();
                    let _transfer_label = ui.label(TypeRole::Caption.text("ATLAS TRANSFER"));
                    for transfer in Transfer::ALL {
                        let _response = ui.selectable_value(
                            &mut self.transfer,
                            transfer,
                            TypeRole::Caption.text(transfer.name()),
                        );
                    }
                    let _separator = ui.separator();
                    let _response = ui.checkbox(&mut self.wet, "PRODUCTION WATER");
                    let _separator = ui.separator();
                    if ui
                        .button(TypeRole::Caption.text("RESET SEMANTIC SIZES"))
                        .clicked()
                    {
                        self.scale = SpecimenScale::default();
                    }
                });
            });
        if before_weight != self.weight || before_transfer != self.transfer {
            install_fonts(ui.ctx(), self.weight);
            install_transfer(ui.ctx(), self.transfer);
            ui.ctx().request_discard("Font atelier raster law changed");
        }
    }

    fn focus_bench(&mut self, ui: &mut egui::Ui, water: &mut Surface) {
        let face = FACES[self.face];
        let profile = PROFILES[self.profile];
        let family = family(face, profile);
        let scale = &mut self.scale;
        let _frame = egui::Frame::new()
            .fill(chrome::CONTROL)
            .stroke(egui::Stroke::new(1.0, chrome::EDGE))
            .inner_margin(12)
            .show(ui, |ui| {
                let _heading = ui.horizontal(|ui| {
                    let _selection = ui
                        .label(TypeRole::Heading.text(format!("{} × {}", face.name, profile.name)));
                    let _province = ui.label(
                        TypeRole::Caption.text(format!("{} · {}", face.province, profile.province)),
                    );
                });
                ui.add_space(4.0);
                ui.columns(2, |columns| {
                    role_witness(&mut columns[0], family.clone(), scale, water);
                    diagnostic_witness(&mut columns[1], family.clone(), *scale);
                });
                ui.add_space(6.0);
                phase_witness(ui, family.clone(), *scale);
                ui.add_space(4.0);
                let response = ui.label(
                    TypeRole::Body
                        .text("HOVER: production tooltip optical path")
                        .color(chrome::HOT),
                );
                let body = scale.font(TypeRole::Body, family.clone());
                let supporting = scale.font(TypeRole::Supporting, family.clone());
                let _response = response.on_hover_ui(move |ui| {
                    let _primary = ui.label(
                        RichText::new("Hover help: Ctrl+, opens settings · Il1 0OQ 8B").font(body),
                    );
                    let _secondary = ui.label(
                        RichText::new("Fine strokes, punctuation, and warm low-contrast ink.")
                            .font(supporting)
                            .color(chrome::MUTED),
                    );
                });
            });
    }

    fn target_selectors(&mut self, ui: &mut egui::Ui) {
        let _frame = egui::Frame::new()
            .fill(chrome::SURFACE)
            .stroke(egui::Stroke::new(1.0, chrome::EDGE_STRONG))
            .inner_margin(10)
            .show(ui, |ui| {
                let _selectors = ui.horizontal(|ui| {
                    let _face_label = ui.label(TypeRole::Caption.text("FACE"));
                    let _face = egui::ComboBox::from_id_salt("font-raster-face")
                        .width(230.0)
                        .height(620.0)
                        .selected_text(TypeRole::Body.text(FACES[self.face].name))
                        .show_ui(ui, |ui| {
                            for (index, face) in FACES.iter().copied().enumerate() {
                                let _choice = ui.selectable_value(
                                    &mut self.face,
                                    index,
                                    TypeRole::Body.text(face.name),
                                );
                            }
                        });
                    let _separator = ui.separator();
                    let _profile_label = ui.label(TypeRole::Caption.text("RASTER LAW"));
                    let _profile = egui::ComboBox::from_id_salt("font-raster-profile")
                        .width(190.0)
                        .height(260.0)
                        .selected_text(TypeRole::Body.text(PROFILES[self.profile].name))
                        .show_ui(ui, |ui| {
                            for (index, profile) in PROFILES.iter().copied().enumerate() {
                                let _choice = ui.selectable_value(
                                    &mut self.profile,
                                    index,
                                    TypeRole::Body.text(profile.name),
                                );
                            }
                        });
                });
                let _selection = ui.label(TypeRole::Instrument.text(format!(
                    "{} · {}",
                    FACES[self.face].province, PROFILES[self.profile].province,
                )));
            });
    }
}

fn role_witness(
    ui: &mut egui::Ui,
    family: FontFamily,
    scale: &mut SpecimenScale,
    water: &mut Surface,
) {
    let _grid = egui::Grid::new("semantic-size-witness")
        .num_columns(3)
        .spacing(egui::vec2(5.0, 3.0))
        .show(ui, |ui| {
            semantic_header(ui, "SEMANTIC ROLE", 112.0);
            semantic_header(ui, "POINTS", 84.0);
            semantic_header(ui, "WITNESS", 250.0);
            ui.end_row();

            for (index, (role, text)) in [
                (TypeRole::Instrument, "ridge 1,842 m · 03:17"),
                (TypeRole::Caption, "CONFIGURATION FILE"),
                (TypeRole::Supporting, "No matching sessions"),
                (TypeRole::Body, "Edit trail and save changes"),
                (TypeRole::Heading, "TRAIL CREATOR"),
                (TypeRole::Title, "CODEX WRANGLER"),
            ]
            .into_iter()
            .enumerate()
            {
                semantic_header(ui, role.name(), 112.0);
                let register = NumberInput::new(&mut scale.points[index], 8.0..=32.0, 0.25, 2)
                    .register_width(60.0)
                    .show(ui)
                    .on_hover_text("scroll by 0.25 pt · double-click register for exact entry");
                water.number_input(&register);
                let _witness = ui.label(RichText::new(text).font(scale.font(role, family.clone())));
                ui.end_row();
            }
        });
}

fn diagnostic_witness(ui: &mut egui::Ui, family: FontFamily, scale: SpecimenScale) {
    for (role, text, color) in [
        (TypeRole::Body, "Il1 0OQ 5S 2Z rn m wvw", chrome::TEXT),
        (
            TypeRole::Supporting,
            "() [] {} <> /\\ | ! ? : ; . ,",
            chrome::TEXT,
        ),
        (TypeRole::Body, "↶ ↷ ↗ ⚙ ♥ ✓ ✕ ⌫ ⏎", chrome::HOT),
        (TypeRole::Supporting, "∑ ∂ μ π √∞ ≠ ≤ ≥ · ×", chrome::TEXT),
        (
            TypeRole::Supporting,
            "正名 café naïve Ångström",
            chrome::MUTED,
        ),
        (TypeRole::Caption, "0123456789 +42.75 −1032", chrome::MUTED),
    ] {
        let _line = ui.label(
            RichText::new(text)
                .font(scale.font(role, family.clone()))
                .color(color),
        );
    }
}

fn phase_witness(ui: &mut egui::Ui, family: FontFamily, scale: SpecimenScale) {
    let _heading =
        ui.label(TypeRole::Caption.text("PHYSICAL X PHASE · 0.00 / 0.25 / 0.50 / 0.75 PIXEL"));
    let ppp = ui.pixels_per_point();
    let (rect, _response) = ui.allocate_exact_size(egui::vec2(1_150.0, 28.0), egui::Sense::hover());
    let painter = ui.painter().with_clip_rect(rect);
    for (index, phase) in [0.0_f32, 0.25, 0.5, 0.75].into_iter().enumerate() {
        let x = rect.left() + index as f32 * 280.0 + phase / ppp;
        let _bounds = painter.text(
            egui::pos2(x, rect.top()),
            egui::Align2::LEFT_TOP,
            format!("{phase:.2}px · Il1 EDIT 012 ↗"),
            scale.font(TypeRole::Body, family.clone()),
            chrome::TEXT,
        );
    }
}

fn semantic_header(ui: &mut egui::Ui, text: &str, width: f32) {
    let (rect, _response) = ui.allocate_exact_size(egui::vec2(width, 24.0), egui::Sense::hover());
    let _plate = ui.painter().rect(
        rect,
        1.0,
        chrome::SURFACE,
        egui::Stroke::new(1.0, chrome::EDGE_STRONG),
        egui::StrokeKind::Inside,
    );
    let _text = ui.painter().text(
        rect.left_center() + egui::vec2(6.0, 0.0),
        egui::Align2::LEFT_CENTER,
        text,
        TypeRole::Instrument.proportional(),
        chrome::MUTED,
    );
}

const fn role_index(role: TypeRole) -> usize {
    match role {
        TypeRole::Instrument => 0,
        TypeRole::Caption => 1,
        TypeRole::Supporting => 2,
        TypeRole::Body => 3,
        TypeRole::Heading => 4,
        TypeRole::Title => 5,
    }
}

#[allow(
    clippy::disallowed_methods,
    reason = "the atelier is the deliberate numerical escape for type-scale research"
)]
fn specimen_font(points: f32, family: FontFamily) -> FontId {
    FontId::new(points, family)
}

fn family(face: Face, profile: RasterProfile) -> FontFamily {
    FontFamily::Name(format!("atelier:{}:{}", face.slug, profile.slug).into())
}

fn install_fonts(ctx: &egui::Context, weight: Weight) {
    let mut fonts = FontDefinitions::empty();
    for profile in PROFILES {
        let fallback_cmu = insert_face(
            &mut fonts,
            &format!("atelier:fallback-cmu:{}", profile.slug),
            match weight {
                Weight::Regular => CMU_REGULAR,
                Weight::Emphasis => CMU_EMPHASIS,
            },
            *profile,
        );
        let fallback_math = insert_face(
            &mut fonts,
            &format!("atelier:fallback-math:{}", profile.slug),
            NOTO_MATH,
            *profile,
        );
        let fallback_symbols = insert_face(
            &mut fonts,
            &format!("atelier:fallback-symbols:{}", profile.slug),
            NOTO_SYMBOLS,
            *profile,
        );
        for face in FACES {
            let key = insert_face(
                &mut fonts,
                &format!("atelier:face:{}:{}", face.slug, profile.slug),
                match weight {
                    Weight::Regular => face.regular,
                    Weight::Emphasis => face.emphasis,
                },
                *profile,
            );
            let mut stack = Vec::with_capacity(4);
            for candidate in [
                key,
                fallback_math.clone(),
                fallback_symbols.clone(),
                fallback_cmu.clone(),
            ] {
                if !stack.contains(&candidate) {
                    stack.push(candidate);
                }
            }
            let _old = fonts.families.insert(family(*face, *profile), stack);
        }
    }
    let production_stack = vec![
        "atelier:fallback-cmu:production".to_owned(),
        "atelier:fallback-math:production".to_owned(),
        "atelier:fallback-symbols:production".to_owned(),
    ];
    let _proportional = fonts
        .families
        .insert(FontFamily::Proportional, production_stack.clone());
    let _monospace = fonts
        .families
        .insert(FontFamily::Monospace, production_stack);
    ctx.set_fonts(fonts);
}

fn insert_face(
    fonts: &mut FontDefinitions,
    key: &str,
    bytes: &'static [u8],
    profile: RasterProfile,
) -> String {
    let key = key.to_owned();
    let data = FontData::from_static(bytes).tweak(profile.tweak());
    let _old = fonts.font_data.insert(key.clone(), Arc::new(data));
    key
}

fn install_transfer(ctx: &egui::Context, transfer: Transfer) {
    ctx.all_styles_mut(|style| {
        style.visuals.text_options.color_transfer_function = transfer.law();
    });
}

fn main() -> Result<()> {
    support::run(FontRasterAtelier::default())
}
