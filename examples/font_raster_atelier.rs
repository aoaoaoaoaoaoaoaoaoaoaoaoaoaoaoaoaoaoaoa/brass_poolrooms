#![expect(
    unused_crate_dependencies,
    reason = "the atelier consumes egui and wgpu through the crate's version-locked re-exports"
)]

mod support;

use std::sync::Arc;

use anyhow::Result;
use brass_poolrooms::{
    chrome::{self, ScrewScroll, TypeRole},
    egui::{self, Color32, FontData, FontDefinitions, FontFamily, FontTweak, RichText},
    water::{Surface, Wetness},
};
use egui::epaint::{
    FontColorTransferFunction,
    text::{HintingTarget, SmoothHinting},
};
use support::Exhibit;

const CELL_WIDTH: f32 = 158.0;
const CELL_HEIGHT: f32 = 96.0;
const ROW_LABEL_WIDTH: f32 = 150.0;

const CMU_REGULAR: &[u8] = include_bytes!("../assets/fonts/cmu-typewriter/cmuntt.ttf");
const CMU_EMPHASIS: &[u8] = include_bytes!("../assets/fonts/atelier/cmu-typewriter/cmuntb.otf");
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
        slug: "ia-mono",
        name: "IA WRITER MONO",
        province: "strict mono candidate",
        regular: include_bytes!("../assets/fonts/atelier/ia-writer/iAWriterMonoS-Regular.ttf"),
        emphasis: include_bytes!("../assets/fonts/atelier/ia-writer/iAWriterMonoS-Bold.ttf"),
    },
    Face {
        slug: "ia-duo",
        name: "IA WRITER DUO",
        province: "duospaced candidate",
        regular: include_bytes!("../assets/fonts/atelier/ia-writer/iAWriterDuoS-Regular.ttf"),
        emphasis: include_bytes!("../assets/fonts/atelier/ia-writer/iAWriterDuoS-Bold.ttf"),
    },
    Face {
        slug: "ia-quattro",
        name: "IA WRITER QUATTRO",
        province: "four-width candidate",
        regular: include_bytes!("../assets/fonts/atelier/ia-writer/iAWriterQuattroS-Regular.ttf"),
        emphasis: include_bytes!("../assets/fonts/atelier/ia-writer/iAWriterQuattroS-Bold.ttf"),
    },
    Face {
        slug: "plex-mono",
        name: "IBM PLEX MONO",
        province: "text / medium candidate",
        regular: include_bytes!("../assets/fonts/atelier/ibm-plex/IBMPlexMono-Text.ttf"),
        emphasis: include_bytes!("../assets/fonts/atelier/ibm-plex/IBMPlexMono-Medium.ttf"),
    },
    Face {
        slug: "plex-sans",
        name: "IBM PLEX SANS",
        province: "proportional candidate",
        regular: include_bytes!("../assets/fonts/atelier/ibm-plex/IBMPlexSans-Text.ttf"),
        emphasis: include_bytes!("../assets/fonts/atelier/ibm-plex/IBMPlexSans-Medium.ttf"),
    },
    Face {
        slug: "jetbrains-mono",
        name: "JETBRAINS MONO",
        province: "screen mono candidate",
        regular: include_bytes!("../assets/fonts/atelier/jetbrains-mono/JetBrainsMono-Regular.ttf"),
        emphasis: include_bytes!("../assets/fonts/atelier/jetbrains-mono/JetBrainsMono-Medium.ttf"),
    },
    Face {
        slug: "courier-prime",
        name: "COURIER PRIME",
        province: "typewriter candidate",
        regular: include_bytes!("../assets/fonts/atelier/courier-prime/CourierPrime-Regular.ttf"),
        emphasis: include_bytes!("../assets/fonts/atelier/courier-prime/CourierPrime-Bold.ttf"),
    },
    Face {
        slug: "noto-math",
        name: "NOTO SANS MATH",
        province: "production math fallback",
        regular: NOTO_MATH,
        emphasis: NOTO_MATH,
    },
    Face {
        slug: "noto-symbols",
        name: "NOTO SANS SYMBOLS 2",
        province: "production symbol fallback",
        regular: NOTO_SYMBOLS,
        emphasis: NOTO_SYMBOLS,
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

#[derive(Default)]
struct FontRasterAtelier {
    face: usize,
    profile: usize,
    weight: Weight,
    transfer: Transfer,
    wet: bool,
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
                        self.focus_bench(ui);
                        ui.add_space(14.0);
                        self.matrix(ui);
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
                });
            });
        if before_weight != self.weight || before_transfer != self.transfer {
            install_fonts(ui.ctx(), self.weight);
            install_transfer(ui.ctx(), self.transfer);
            ui.ctx().request_discard("Font atelier raster law changed");
        }
    }

    fn focus_bench(&self, ui: &mut egui::Ui) {
        let face = FACES[self.face];
        let profile = PROFILES[self.profile];
        let family = family(face, profile);
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
                    role_witness(&mut columns[0], family.clone());
                    diagnostic_witness(&mut columns[1], family.clone());
                });
                ui.add_space(6.0);
                phase_witness(ui, family.clone());
                ui.add_space(4.0);
                let response = ui.label(
                    TypeRole::Body
                        .text("HOVER: production tooltip optical path")
                        .color(chrome::HOT),
                );
                let _response = response.on_hover_ui(move |ui| {
                    let _primary = ui.label(
                        RichText::new("Hover help: Ctrl+, opens settings · Il1 0OQ 8B")
                            .font(TypeRole::Body.in_family(family.clone())),
                    );
                    let _secondary = ui.label(
                        RichText::new("Fine strokes, punctuation, and warm low-contrast ink.")
                            .font(TypeRole::Supporting.in_family(family))
                            .color(chrome::MUTED),
                    );
                });
            });
    }

    fn matrix(&mut self, ui: &mut egui::Ui) {
        let _heading = ui.label(TypeRole::Heading.text("COMPLETE FACE × RASTER WITNESS"));
        let _instructions = ui.label(TypeRole::Caption.text(
            "Click a specimen for the complete role, pixel-phase, symbol, and tooltip bench above.",
        ));
        let _horizontal = egui::ScrollArea::horizontal()
            .id_salt("font-raster-matrix-x")
            .auto_shrink([false, true])
            .show(ui, |ui| {
                let _header = ui.horizontal(|ui| {
                    let _label = ui.allocate_ui(egui::vec2(ROW_LABEL_WIDTH, 28.0), |ui| {
                        let _face = ui.label(TypeRole::Caption.text("FACE"));
                    });
                    for profile in PROFILES {
                        let _profile = ui.allocate_ui(egui::vec2(CELL_WIDTH, 28.0), |ui| {
                            let _name = ui.label(TypeRole::Caption.text(profile.name));
                        });
                    }
                });
                for (face_index, face) in FACES.iter().copied().enumerate() {
                    let _row = ui.horizontal(|ui| {
                        let _label =
                            ui.allocate_ui(egui::vec2(ROW_LABEL_WIDTH, CELL_HEIGHT), |ui| {
                                let _name = ui.label(TypeRole::Caption.text(face.name));
                                let _province = ui.label(
                                    TypeRole::Instrument
                                        .text(face.province)
                                        .color(chrome::MUTED),
                                );
                            });
                        for (profile_index, profile) in PROFILES.iter().copied().enumerate() {
                            let selected = self.face == face_index && self.profile == profile_index;
                            if matrix_cell(ui, face, profile, selected).clicked() {
                                self.face = face_index;
                                self.profile = profile_index;
                            }
                        }
                    });
                }
            });
    }
}

fn role_witness(ui: &mut egui::Ui, family: FontFamily) {
    for (role, text) in [
        (TypeRole::Instrument, "10.5 · ridge 1,842 m · 03:17"),
        (TypeRole::Caption, "12 · CONFIGURATION FILE"),
        (TypeRole::Supporting, "13 · No matching sessions"),
        (TypeRole::Body, "14 · Edit trail and save changes"),
        (TypeRole::Heading, "15 · TRAIL CREATOR"),
        (TypeRole::Title, "18 · CODEX WRANGLER"),
    ] {
        let _line = ui.label(RichText::new(text).font(role.in_family(family.clone())));
    }
}

fn diagnostic_witness(ui: &mut egui::Ui, family: FontFamily) {
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
                .font(role.in_family(family.clone()))
                .color(color),
        );
    }
}

fn phase_witness(ui: &mut egui::Ui, family: FontFamily) {
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
            TypeRole::Body.in_family(family.clone()),
            chrome::TEXT,
        );
    }
}

fn matrix_cell(
    ui: &mut egui::Ui,
    face: Face,
    profile: RasterProfile,
    selected: bool,
) -> egui::Response {
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(CELL_WIDTH, CELL_HEIGHT), egui::Sense::click());
    let hovered = response.hovered();
    let fill = if selected {
        Color32::from_rgb(44, 36, 25)
    } else if hovered {
        chrome::RAISED
    } else {
        chrome::CONTROL
    };
    let stroke = if selected || hovered {
        egui::Stroke::new(1.0, chrome::HOT)
    } else {
        egui::Stroke::new(1.0, chrome::EDGE)
    };
    let _shape = ui
        .painter()
        .rect(rect, 1.0, fill, stroke, egui::StrokeKind::Inside);
    let family = family(face, profile);
    let _content = ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(rect.shrink2(egui::vec2(6.0, 5.0)))
            .layout(egui::Layout::top_down(egui::Align::LEFT)),
        |ui| {
            ui.spacing_mut().item_spacing.y = 1.0;
            let _body = ui.label(
                RichText::new("Edit trail · 42 km").font(TypeRole::Body.in_family(family.clone())),
            );
            let _hard = ui.label(
                RichText::new("Il1 0OQ rn mw").font(TypeRole::Supporting.in_family(family.clone())),
            );
            let _symbols = ui.label(
                RichText::new("↶ ↷ ↗ ⚙ ♥ ✓ ∑ μ")
                    .font(TypeRole::Caption.in_family(family.clone()))
                    .color(chrome::HOT),
            );
            let _caption = ui.label(
                RichText::new("CFG 03:17 +42.75")
                    .font(TypeRole::Instrument.in_family(family))
                    .color(chrome::MUTED),
            );
        },
    );
    response
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
