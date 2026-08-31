//! Native Foundry judgment surfaces. This module is development apparatus,
//! feature-gated out of production dependencies, and rendered by the same
//! egui/WGPU path as shipped mechanisms.

use std::sync::Arc;

use crate::{
    chrome::{
        self, DieTopology, ForgedMesh, ForgedVertex, MechanismSize, Monoglyph, MonoglyphFinish,
        StudyEtch, StudyEtchPalette, Symbol,
    },
    egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Vec2},
};

type BakedVertex = ForgedVertex;
type BakedMesh = ForgedMesh;
type BakedColor = [u8; 4];

#[derive(Clone, Copy)]
struct BakedStudyCell {
    button: BakedMesh,
    plate: BakedMesh,
}

#[derive(Clone, Copy)]
struct BakedOpticsCell {
    buttons: [BakedMesh; 3],
    sockets: [BakedMesh; 3],
    plate: BakedMesh,
    sphere: BakedMesh,
    cylinder: BakedMesh,
    screw: BakedMesh,
}

mod material_atlas {
    use super::{BakedMesh, BakedStudyCell, BakedVertex};

    include!(concat!(env!("OUT_DIR"), "/material_study_atlas.rs"));
}

mod optics_atlas {
    use super::{BakedColor, BakedMesh, BakedOpticsCell, BakedVertex};

    include!(concat!(env!("OUT_DIR"), "/optics_atelier_atlas.rs"));
}

const SOOT_WIDTHS: [u8; 4] = [4, 6, 8, 10];
const SOOT_SRGB: [u8; 4] = [0, 10, 26, 46];
const DIFFICULT_SYMBOLS: [Symbol; 7] = [
    Symbol::Settings,
    Symbol::Save,
    Symbol::Rename,
    Symbol::Delete,
    Symbol::Heart,
    Symbol::Export,
    Symbol::Visibility,
];

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum Bench {
    #[default]
    Forge,
    Topologies,
    Soot,
    Legacy,
    Mandate,
}

impl Bench {
    const ALL: [Self; 5] = [
        Self::Forge,
        Self::Topologies,
        Self::Soot,
        Self::Legacy,
        Self::Mandate,
    ];

    const fn name(self) -> &'static str {
        match self {
            Self::Forge => "JUDGMENT SURFACE",
            Self::Topologies => "DIE TOPOLOGIES",
            Self::Soot => "SOOT MATRIX",
            Self::Legacy => "PRODUCTION CONTROL",
            Self::Mandate => "WORKING MANDATE",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BrassCharge {
    Current,
    Warm,
    Yellow,
    Pale,
}

impl BrassCharge {
    const ALL: [Self; 4] = [Self::Current, Self::Warm, Self::Yellow, Self::Pale];

    const fn name(self) -> &'static str {
        match self {
            Self::Current => "CURRENT",
            Self::Warm => "WARM",
            Self::Yellow => "YELLOW",
            Self::Pale => "PALE",
        }
    }

    const fn atlas_index(self) -> usize {
        match self {
            Self::Current => 0,
            Self::Warm => 1,
            Self::Yellow => 2,
            Self::Pale => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ToolMark {
    Isotropic,
    Horizontal,
    Vertical,
}

impl ToolMark {
    const ALL: [Self; 3] = [Self::Isotropic, Self::Horizontal, Self::Vertical];

    const fn name(self) -> &'static str {
        match self {
            Self::Isotropic => "ISOTROPIC",
            Self::Horizontal => "HORIZONTAL",
            Self::Vertical => "VERTICAL",
        }
    }

    const fn atlas_index(self) -> usize {
        match self {
            Self::Isotropic => 0,
            Self::Horizontal => 1,
            Self::Vertical => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LightRoom {
    Black,
    Overhead,
    Furnace,
}

impl LightRoom {
    const ALL: [Self; 3] = [Self::Black, Self::Overhead, Self::Furnace];

    const fn name(self) -> &'static str {
        match self {
            Self::Black => "BLACK",
            Self::Overhead => "OVERHEAD CARD",
            Self::Furnace => "FURNACE LINE",
        }
    }

    const fn atlas_index(self) -> usize {
        match self {
            Self::Black => 0,
            Self::Overhead => 1,
            Self::Furnace => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum FinishLaw {
    #[default]
    Semantic,
    BrightCut,
    Void,
    Danger,
    Love,
}

impl FinishLaw {
    const ALL: [Self; 5] = [
        Self::Semantic,
        Self::BrightCut,
        Self::Void,
        Self::Danger,
        Self::Love,
    ];

    const fn name(self) -> &'static str {
        match self {
            Self::Semantic => "SEMANTIC",
            Self::BrightCut => "BRIGHT CUT",
            Self::Void => "VOID",
            Self::Danger => "DANGER",
            Self::Love => "LOVE",
        }
    }

    const fn resolve(self, symbol: Symbol) -> MonoglyphFinish {
        match self {
            Self::Semantic => symbol.default_finish(),
            Self::BrightCut => MonoglyphFinish::BrightCut,
            Self::Void => MonoglyphFinish::Void,
            Self::Danger => MonoglyphFinish::Danger,
            Self::Love => MonoglyphFinish::Love,
        }
    }
}

/// Persistent state for the feature-gated Foundry Optics Atelier.
pub struct FoundryOpticsAtelier {
    bench: Bench,
    soot_symbol: Symbol,
    charge: BrassCharge,
    tool_mark: ToolMark,
    room: LightRoom,
    topology: DieTopology,
    finish: FinishLaw,
    soot_width: u8,
    soot_srgb: u8,
}

impl Default for FoundryOpticsAtelier {
    fn default() -> Self {
        Self {
            bench: Bench::Forge,
            soot_symbol: Symbol::Settings,
            charge: BrassCharge::Yellow,
            tool_mark: ToolMark::Isotropic,
            room: LightRoom::Overhead,
            topology: DieTopology::ShadowWall,
            finish: FinishLaw::Semantic,
            soot_width: 8,
            soot_srgb: 10,
        }
    }
}

impl FoundryOpticsAtelier {
    /// Render one complete atelier pass inside the caller's scroll surface.
    pub fn show(&mut self, ui: &mut egui::Ui) {
        let _title = ui.label(chrome::title("FOUNDRY OPTICS ATELIER"));
        let _law = ui.label(chrome::muted(
            "native-scale verdicts · production fonts and meshes · egui tessellation · WGPU composite",
        ));
        ui.add_space(12.0);
        self.control_panel(ui);
        ui.add_space(12.0);
        let _tabs = ui.horizontal_wrapped(|ui| {
            for bench in Bench::ALL {
                let _tab = ui.selectable_value(&mut self.bench, bench, bench.name());
            }
        });
        ui.add_space(18.0);

        match self.bench {
            Bench::Forge => self.forge_bench(ui),
            Bench::Topologies => self.topology_bench(ui),
            Bench::Soot => self.soot_bench(ui),
            Bench::Legacy => legacy_bench(ui),
            Bench::Mandate => working_mandate(ui),
        }
    }

    fn control_panel(&mut self, ui: &mut egui::Ui) {
        let _panel = egui::Frame::new()
            .fill(chrome::SURFACE)
            .stroke(Stroke::new(1.0_f32, chrome::EDGE_STRONG))
            .inner_margin(12)
            .show(ui, |ui| {
                let _title = ui.label(chrome::eyebrow("CENTRAL FOUNDRY REGISTER"));
                let _selection = ui.label(chrome::muted(format!(
                    "LINEAR-LIGHT BLINN · {} BRASS · {} TOOL · {} · {} · {}",
                    self.charge.name(),
                    self.tool_mark.name(),
                    self.room.name(),
                    self.topology.name(),
                    self.finish.name(),
                )));
                ui.add_space(8.0);
                selector_axis(
                    ui,
                    "BRASS CHARGE",
                    &mut self.charge,
                    BrassCharge::ALL,
                    |x| x.name(),
                );
                selector_axis(ui, "TOOL MARK", &mut self.tool_mark, ToolMark::ALL, |x| {
                    x.name()
                });
                selector_axis(ui, "LIGHT ROOM", &mut self.room, LightRoom::ALL, |x| {
                    x.name()
                });
                selector_axis(
                    ui,
                    "DIE TOPOLOGY",
                    &mut self.topology,
                    DieTopology::ALL,
                    |x| x.name(),
                );
                selector_axis(ui, "FACE FINISH", &mut self.finish, FinishLaw::ALL, |x| {
                    x.name()
                });
                selector_axis(
                    ui,
                    "SOOT WIDTH",
                    &mut self.soot_width,
                    SOOT_WIDTHS,
                    |x| match x {
                        4 => "0.50 PX",
                        6 => "0.75 PX",
                        8 => "1.00 PX",
                        10 => "1.25 PX",
                        _ => unreachable!("the soot-width register is closed"),
                    },
                );
                selector_axis(
                    ui,
                    "SOOT TONE",
                    &mut self.soot_srgb,
                    SOOT_SRGB,
                    |x| match x {
                        0 => "sRGB 0",
                        10 => "sRGB 10",
                        26 => "sRGB 26",
                        46 => "sRGB 46",
                        _ => unreachable!("the soot-tone register is closed"),
                    },
                );
                let _register = ui.label(chrome::muted(self.topology.register()));
            });
    }

    fn forge_bench(&self, ui: &mut egui::Ui) {
        heading(
            ui,
            "SHARED GEOMETRY COUPONS",
            "Every coupon and symbol crown below is selected from the same complete alloy × tool-mark × room atlas.",
        );
        geometry_coupons(ui, self.candidate_index());
        ui.add_space(18.0);
        study_witness(ui, self);
    }

    fn soot_bench(&mut self, ui: &mut egui::Ui) {
        heading(
            ui,
            "SOOT KEYLINE CALIBRATION",
            "Width is measured in physical pixels; tone is the exact 8-bit sRGB value submitted to the same egui glyph-atlas composite used by the selected die prototype.",
        );
        let _symbols = ui.horizontal_wrapped(|ui| {
            let _label = ui.label(chrome::eyebrow("DIE"));
            for symbol in DIFFICULT_SYMBOLS {
                let _choice = ui.selectable_value(&mut self.soot_symbol, symbol, symbol.name());
            }
        });
        ui.add_space(12.0);

        for srgb in SOOT_SRGB {
            let _row = ui.horizontal_top(|ui| {
                let _axis = ui.add_sized(
                    [108.0, 86.0],
                    egui::Label::new(chrome::eyebrow(format!(
                        "sRGB {srgb}\n{:>3}% CODE",
                        u16::from(srgb) * 100 / 255
                    ))),
                );
                for eighth_pixels in SOOT_WIDTHS {
                    study_soot_cell(ui, self, eighth_pixels, srgb);
                }
            });
            ui.add_space(6.0);
        }
        ui.add_space(8.0);
        let _note = ui.label(chrome::muted(
            "Fractional candidates remain subpixel offsets in the production font-atlas composite; they are not SDF reconstructions.",
        ));
    }

    fn topology_bench(&mut self, ui: &mut egui::Ui) {
        heading(
            ui,
            "DIE STROKE TOPOLOGIES",
            "All nine stroke families share the selected alloy, tool mark, room, finish, soot, font atlas, and S/M/L gauges. Select a family here to send it to the complete witness.",
        );
        let _symbols = ui.horizontal_wrapped(|ui| {
            let _label = ui.label(chrome::eyebrow("MARK"));
            for symbol in DIFFICULT_SYMBOLS {
                let _choice = ui.selectable_value(&mut self.soot_symbol, symbol, symbol.name());
            }
        });
        ui.add_space(14.0);
        for row in DieTopology::ALL.chunks(3) {
            let _row = ui.horizontal_top(|ui| {
                for &topology in row {
                    topology_cell(ui, self, topology);
                }
            });
            ui.add_space(8.0);
        }
        ui.add_space(12.0);
        let _frontier = egui::Frame::new()
            .fill(chrome::SURFACE)
            .stroke(Stroke::new(1.0_f32, chrome::HOT))
            .inner_margin(10)
            .show(ui, |ui| {
                let _title = ui.label(chrome::eyebrow("RELIEF-COMPILER BOUNDARY"));
                let _body = ui.label(chrome::muted(
                    "These are exact screen-space prototypes: production glyph masks, egui tessellation, and WGPU compositing with tabulated relief passes. They compare topology honestly at native size, but become physical dies only after the Foundry compiles authored S/M/L contours into wall normals, toe occlusion, and floor geometry.",
                ));
            });
    }

    fn candidate_index(&self) -> usize {
        debug_assert_eq!(optics_atlas::CHARGE_COUNT, BrassCharge::ALL.len());
        let index = (self.charge.atlas_index() * optics_atlas::GRAIN_COUNT
            + self.tool_mark.atlas_index())
            * optics_atlas::ROOM_COUNT
            + self.room.atlas_index();
        debug_assert!(index < optics_atlas::CANDIDATE_COUNT);
        index
    }

    fn treatment(&self, topology: DieTopology, soot_width: u8, soot_srgb: u8) -> StudyEtch {
        let colors = optics_atlas::ETCH_PALETTES[self.candidate_index()];
        StudyEtch::new(
            topology,
            StudyEtchPalette {
                surface: color(colors[0]),
                key_wall: color(colors[1]),
                lee_wall: color(colors[2]),
                flank: color(colors[3]),
            },
            soot_width,
            soot_srgb,
        )
    }
}

fn selector_axis<T: Copy + PartialEq, const N: usize>(
    ui: &mut egui::Ui,
    label: &str,
    selected: &mut T,
    options: [T; N],
    name: impl Fn(T) -> &'static str,
) {
    let _axis = ui.horizontal_wrapped(|ui| {
        let _label = ui.add_sized([110.0, 18.0], egui::Label::new(chrome::eyebrow(label)));
        for option in options {
            let _choice = ui.selectable_value(selected, option, name(option));
        }
    });
}

fn color([r, g, b, a]: BakedColor) -> Color32 {
    Color32::from_rgba_unmultiplied(r, g, b, a)
}

fn heading(ui: &mut egui::Ui, title: &str, law: &str) {
    let _title = ui.label(chrome::title(title));
    let _law = ui.label(chrome::muted(law));
    ui.add_space(14.0);
}

fn geometry_coupons(ui: &mut egui::Ui, candidate_index: usize) {
    let cell = optics_atlas::CELLS[candidate_index];
    let (rect, _response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 132.0), Sense::hover());
    let painter = ui.painter_at(rect);
    let _bed = painter.rect_filled(rect, 1.0, chrome::SURFACE);
    let _edge = painter.rect_stroke(
        rect,
        1.0,
        Stroke::new(1.0_f32, chrome::EDGE),
        StrokeKind::Inside,
    );
    let step = rect.width() / 5.0;
    let centers = (0..5).map(|index| rect.left() + step * (index as f32 + 0.5));
    for (center, name) in centers
        .clone()
        .zip(["CROWN", "BEVEL", "SPHERE", "BARREL", "SCREW"])
    {
        let _label = painter.text(
            Pos2::new(center, rect.top() + 11.0),
            Align2::CENTER_TOP,
            name,
            chrome::TypeRole::Annotation.monospace(ui.style()),
            chrome::MUTED,
        );
    }
    let centers = centers.collect::<Vec<_>>();
    let origin_y = rect.top() + 80.0;
    let button_origin = Pos2::new(centers[0], origin_y);
    let socket = Rect::from_center_size(button_origin, Vec2::splat(32.0));
    let _void = painter.rect_filled(socket, 1.0, Color32::from_rgb(2, 2, 3));
    paint_mesh(
        &painter,
        socket.shrink(1.0),
        optics_atlas::BUTTON_SHADOWS[2],
        button_origin,
    );
    paint_mesh(&painter, socket.shrink(1.0), cell.buttons[2], button_origin);
    paint_mesh(&painter, socket, cell.sockets[2], button_origin);
    paint_mesh(
        &painter,
        rect.shrink(1.0),
        optics_atlas::PLATE_SHADOW,
        Pos2::new(centers[1], origin_y),
    );
    paint_mesh(
        &painter,
        rect.shrink(1.0),
        cell.plate,
        Pos2::new(centers[1], origin_y),
    );
    paint_mesh(
        &painter,
        rect.shrink(1.0),
        cell.sphere,
        Pos2::new(centers[2], origin_y),
    );
    paint_mesh(
        &painter,
        rect.shrink(1.0),
        cell.cylinder,
        Pos2::new(centers[3], origin_y),
    );
    paint_mesh(
        &painter,
        rect.shrink(1.0),
        cell.screw,
        Pos2::new(centers[4], origin_y),
    );
    let _law = painter.text(
        Pos2::new(rect.center().x, rect.bottom() - 12.0),
        Align2::CENTER_CENTER,
        "same candidate law · actual-size meshes",
        chrome::TypeRole::Annotation.monospace(ui.style()),
        chrome::MUTED,
    );
}

fn study_witness(ui: &mut egui::Ui, atelier: &FoundryOpticsAtelier) {
    heading(
        ui,
        "COMPLETE SYMBOL WITNESS",
        "Medium is the primary reading gauge; Small remains an admission question. Every mark uses the selected topology and finish law.",
    );
    let candidate = optics_atlas::CELLS[atelier.candidate_index()];
    let treatment = atelier.treatment(atelier.topology, atelier.soot_width, atelier.soot_srgb);
    let column_count = 3;
    let band = Symbol::ALL.len().div_ceil(column_count);
    ui.columns(column_count, |columns| {
        for (column, symbols) in columns.iter_mut().zip(Symbol::ALL.chunks(band)) {
            gauge_header(column);
            column.add_space(8.0);
            for &symbol in symbols {
                study_symbol_row(
                    column,
                    symbol,
                    &candidate,
                    atelier.finish.resolve(symbol),
                    treatment,
                );
                column.add_space(7.0);
            }
        }
    });
    ui.add_space(14.0);
    let _warning = egui::Frame::new()
        .fill(chrome::SURFACE)
        .stroke(Stroke::new(1.0_f32, chrome::EDGE))
        .inner_margin(10)
        .show(ui, |ui| {
            let _title = ui.label(chrome::eyebrow("SMALL-GAUGE ADMISSION"));
            let _body = ui.label(chrome::muted(
                "Settings, save, rename, delete, and other fine dies must earn Small individually. This witness exposes failure; it does not grant admission.",
            ));
        });
}

fn study_symbol_row(
    ui: &mut egui::Ui,
    symbol: Symbol,
    candidate: &BakedOpticsCell,
    finish: MonoglyphFinish,
    treatment: StudyEtch,
) {
    let _row = ui.horizontal(|ui| {
        let _name = ui.add_sized(
            [84.0, MechanismSize::Large.side()],
            egui::Label::new(chrome::eyebrow(symbol.name())),
        );
        for size in MechanismSize::ALL {
            let _cell = egui::Frame::new()
                .fill(if size == MechanismSize::Medium {
                    chrome::SURFACE
                } else {
                    Color32::TRANSPARENT
                })
                .inner_margin(4)
                .show(ui, |ui| {
                    let response = study_monoglyph(ui, symbol, size, candidate, finish, treatment);
                    let _hover = response.on_hover_text(format!(
                        "{} · {} · {} · {}",
                        symbol.name(),
                        size_name(size),
                        finish.name(),
                        treatment.topology.name(),
                    ));
                });
        }
    });
}

fn study_monoglyph(
    ui: &mut egui::Ui,
    symbol: Symbol,
    size: MechanismSize,
    candidate: &BakedOpticsCell,
    finish: MonoglyphFinish,
    treatment: StudyEtch,
) -> egui::Response {
    let side = size.side();
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(side), Sense::hover());
    let origin = pixel_center(rect, side, ui.pixels_per_point());
    let socket = Rect::from_center_size(origin, Vec2::splat(side));
    let painter = ui.painter();
    let gauge = gauge_index(size);
    let _void = painter.rect_filled(socket, 1.0, Color32::from_rgb(2, 2, 3));
    paint_mesh(
        painter,
        socket.shrink(1.0),
        optics_atlas::BUTTON_SHADOWS[gauge],
        origin,
    );
    paint_mesh(
        painter,
        socket.shrink(1.0),
        candidate.buttons[gauge],
        origin,
    );
    chrome::paint_study_etch(
        painter,
        socket.shrink(1.0),
        origin,
        symbol.glyph(),
        finish,
        treatment,
        3.25,
        side * 0.5 * (89.0 / 132.0),
    );
    paint_mesh(painter, socket, candidate.sockets[gauge], origin);
    response
}

fn pixel_center(rect: Rect, side: f32, pixels_per_point: f32) -> Pos2 {
    let casing_side = side - 2.0;
    let physical_side = (casing_side * pixels_per_point).round() as u32;
    let phase = if physical_side.is_multiple_of(2) {
        0.0
    } else {
        0.5
    };
    let snap = |coordinate: f32| {
        ((coordinate * pixels_per_point - phase).round() + phase) / pixels_per_point
    };
    Pos2::new(snap(rect.center().x), snap(rect.center().y))
}

const fn gauge_index(size: MechanismSize) -> usize {
    match size {
        MechanismSize::Small => 0,
        MechanismSize::Medium => 1,
        MechanismSize::Large => 2,
    }
}

fn topology_cell(ui: &mut egui::Ui, atelier: &mut FoundryOpticsAtelier, topology: DieTopology) {
    let selected = atelier.topology == topology;
    let candidate = optics_atlas::CELLS[atelier.candidate_index()];
    let treatment = atelier.treatment(topology, atelier.soot_width, atelier.soot_srgb);
    let finish = atelier.finish.resolve(atelier.soot_symbol);
    let _cell = egui::Frame::new()
        .fill(chrome::SURFACE)
        .stroke(Stroke::new(
            if selected { 1.5 } else { 1.0 },
            if selected { chrome::HOT } else { chrome::EDGE },
        ))
        .inner_margin(10)
        .show(ui, |ui| {
            let _column = ui.vertical(|ui| {
                ui.set_width(326.0);
                let _select = ui.selectable_value(&mut atelier.topology, topology, topology.name());
                let _register = ui.label(chrome::muted(topology.register()));
                ui.add_space(7.0);
                let _gauges = ui.horizontal(|ui| {
                    for size in MechanismSize::ALL {
                        let _button = study_monoglyph(
                            ui,
                            atelier.soot_symbol,
                            size,
                            &candidate,
                            finish,
                            treatment,
                        );
                        ui.add_space(9.0);
                    }
                });
            });
        });
}

fn study_soot_cell(ui: &mut egui::Ui, atelier: &FoundryOpticsAtelier, eighth_pixels: u8, srgb: u8) {
    let selected = eighth_pixels == atelier.soot_width && srgb == atelier.soot_srgb;
    let candidate = optics_atlas::CELLS[atelier.candidate_index()];
    let treatment = atelier.treatment(atelier.topology, eighth_pixels, srgb);
    let finish = atelier.finish.resolve(atelier.soot_symbol);
    let _cell = egui::Frame::new()
        .fill(chrome::SURFACE)
        .stroke(Stroke::new(
            if selected { 1.5 } else { 1.0 },
            if selected { chrome::HOT } else { chrome::EDGE },
        ))
        .inner_margin(8)
        .show(ui, |ui| {
            ui.set_width(176.0);
            let _label = ui.label(chrome::eyebrow(format!(
                "{:.2} PX · sRGB {srgb}{}",
                f32::from(eighth_pixels) / 8.0,
                if selected { " · SELECTED" } else { "" },
            )));
            ui.add_space(6.0);
            let _specimens = ui.horizontal(|ui| {
                for size in MechanismSize::ALL {
                    let response = study_monoglyph(
                        ui,
                        atelier.soot_symbol,
                        size,
                        &candidate,
                        finish,
                        treatment,
                    );
                    let _hover = response.on_hover_text(format!(
                        "{} · {} · {:.2} physical-pixel soot · sRGB {srgb}",
                        atelier.soot_symbol.name(),
                        size_name(size),
                        f32::from(eighth_pixels) / 8.0,
                    ));
                }
            });
        });
}

fn production_witness(ui: &mut egui::Ui) {
    heading(
        ui,
        "COMPLETE SYMBOL WITNESS",
        "Every cell is a live production Monoglyph. Medium is the primary reading gauge; small is an admission question, never an automatic entitlement.",
    );
    let column_count = 3;
    let band = Symbol::ALL.len().div_ceil(column_count);
    ui.columns(column_count, |columns| {
        for (column, symbols) in columns.iter_mut().zip(Symbol::ALL.chunks(band)) {
            gauge_header(column);
            column.add_space(8.0);
            for &symbol in symbols {
                symbol_row(column, symbol);
                column.add_space(7.0);
            }
        }
    });

    ui.add_space(14.0);
    let _warning = egui::Frame::new()
        .fill(chrome::SURFACE)
        .stroke(Stroke::new(1.0_f32, chrome::EDGE))
        .inner_margin(10)
        .show(ui, |ui| {
            let _title = ui.label(chrome::eyebrow("SMALL-GAUGE ADMISSION"));
            let _body = ui.label(chrome::muted(
                "No minimum-size policy is encoded yet. Settings, save, rename, delete, and other fine dies must earn Small individually; authored gauge-specific cuts are admissible.",
            ));
        });
}

fn gauge_header(ui: &mut egui::Ui) {
    let _legend = ui.horizontal(|ui| {
        let _spacer = ui.add_sized([84.0, 12.0], egui::Label::new(""));
        for (size, name) in [
            (MechanismSize::Small, "S · 20"),
            (MechanismSize::Medium, "M · 24"),
            (MechanismSize::Large, "L · 32"),
        ] {
            let width = size.side() + 12.0;
            let _name = ui.add_sized([width, 12.0], egui::Label::new(chrome::eyebrow(name)));
        }
    });
}

fn symbol_row(ui: &mut egui::Ui, symbol: Symbol) {
    let _row = ui.horizontal(|ui| {
        let _name = ui.add_sized(
            [84.0, MechanismSize::Large.side()],
            egui::Label::new(chrome::eyebrow(symbol.name())),
        );
        for size in MechanismSize::ALL {
            let _cell = egui::Frame::new()
                .fill(if size == MechanismSize::Medium {
                    chrome::SURFACE
                } else {
                    Color32::TRANSPARENT
                })
                .inner_margin(4)
                .show(ui, |ui| {
                    let _button = Monoglyph::symbol(symbol)
                        .size(size)
                        .show(ui)
                        .on_hover_text(format!("{} · {}", symbol.name(), size_name(size)));
                });
        }
    });
}

const fn size_name(size: MechanismSize) -> &'static str {
    match size {
        MechanismSize::Small => "SMALL",
        MechanismSize::Medium => "MEDIUM",
        MechanismSize::Large => "LARGE",
    }
}

fn legacy_bench(ui: &mut egui::Ui) {
    production_witness(ui);
    ui.add_space(24.0);
    let production = material_coordinate(
        material_atlas::PRODUCTION_ROW,
        material_atlas::PRODUCTION_COLUMN,
    );
    heading(
        ui,
        "DARK-BRONZE MATERIAL LAW",
        &format!(
            "Fixed eye and 60° key · identical 32 pt plunger and plate · {production} is production · native WGPU output"
        ),
    );
    let size = Vec2::new(ui.available_width(), 560.0);
    let (canvas, _response) = ui.allocate_exact_size(size, Sense::hover());
    paint_material_matrix(&ui.painter_at(canvas), canvas);
    ui.add_space(8.0);
    let _caveat = ui.label(chrome::muted(
        "This inherited forge varies heuristic Blinn-Phong contrast and exposure. It remains the production witness, not the new linear-light brass hypothesis.",
    ));
}

fn paint_material_matrix(painter: &egui::Painter, canvas: Rect) {
    const LABEL_WIDTH: f32 = 168.0;
    const HEADER_HEIGHT: f32 = 58.0;
    const ROW_HEIGHT: f32 = 98.0;
    const GUTTER: f32 = 7.0;

    let grid_left = canvas.left() + LABEL_WIDTH;
    let grid_width = canvas.width() - LABEL_WIDTH;
    let column_width = (grid_width
        - GUTTER * material_atlas::COLUMN_COUNT.saturating_sub(1) as f32)
        / material_atlas::COLUMN_COUNT as f32;
    let ctx = painter.ctx();
    let style = ctx.style_of(ctx.theme());
    let label_font = chrome::TypeRole::Label.monospace(&style);
    let small_font = chrome::TypeRole::Annotation.monospace(&style);
    let cell_font = chrome::TypeRole::Label.monospace(&style);

    let _contrast = painter.text(
        Pos2::new(canvas.left(), canvas.top() + 12.0),
        Align2::LEFT_TOP,
        "SPECULAR CONTRAST ↓",
        label_font.clone(),
        chrome::HOT,
    );
    let _exposure = painter.text(
        Pos2::new(grid_left, canvas.top() + 12.0),
        Align2::LEFT_TOP,
        "GAMMA-SPACE EXPOSURE →",
        label_font.clone(),
        chrome::HOT,
    );

    for column in 0..material_atlas::COLUMN_COUNT {
        let center_x = grid_left + column as f32 * (column_width + GUTTER) + column_width * 0.5;
        let exposure = material_atlas::EXPOSURES[column];
        let _heading = painter.text(
            Pos2::new(center_x, canvas.top() + 31.0),
            Align2::CENTER_TOP,
            format!("{} · {exposure:.2}×", column + 1),
            label_font.clone(),
            chrome::TEXT,
        );
        let _production = painter.text(
            Pos2::new(center_x, canvas.top() + 46.0),
            Align2::CENTER_TOP,
            if column == material_atlas::PRODUCTION_COLUMN {
                "PRODUCTION"
            } else {
                ""
            },
            small_font.clone(),
            chrome::HOT,
        );
    }

    for row in 0..material_atlas::ROW_COUNT {
        let top = canvas.top() + HEADER_HEIGHT + row as f32 * (ROW_HEIGHT + GUTTER);
        let letter = (b'A' + row as u8) as char;
        let _row = painter.text(
            Pos2::new(canvas.left(), top + 27.0),
            Align2::LEFT_CENTER,
            format!("{letter}  {}", material_atlas::ROW_NAMES[row]),
            label_font.clone(),
            if row == material_atlas::PRODUCTION_ROW {
                chrome::HOT
            } else {
                chrome::TEXT
            },
        );
        let _exponent = painter.text(
            Pos2::new(canvas.left(), top + 45.0),
            Align2::LEFT_CENTER,
            format!("GLINT n = {:.0}", material_atlas::GLINT_EXPONENTS[row]),
            small_font.clone(),
            chrome::MUTED,
        );
        for column in 0..material_atlas::COLUMN_COUNT {
            let left = grid_left + column as f32 * (column_width + GUTTER);
            let cell =
                Rect::from_min_size(Pos2::new(left, top), Vec2::new(column_width, ROW_HEIGHT));
            paint_material_cell(painter, cell, row, column, cell_font.clone());
        }
    }
}

fn paint_material_cell(
    painter: &egui::Painter,
    cell: Rect,
    row: usize,
    column: usize,
    font: FontId,
) {
    let production =
        row == material_atlas::PRODUCTION_ROW && column == material_atlas::PRODUCTION_COLUMN;
    let _bed = painter.rect_filled(cell, 1.0, chrome::SURFACE);
    let _edge = painter.rect_stroke(
        cell,
        1.0,
        Stroke::new(
            if production { 1.5 } else { 1.0 },
            if production {
                chrome::HOT
            } else {
                chrome::EDGE
            },
        ),
        StrokeKind::Inside,
    );

    let origin_y = cell.top() + 42.0;
    let button_origin = Pos2::new(cell.center().x - 27.0, origin_y);
    let plate_origin = Pos2::new(cell.center().x + 27.0, origin_y);
    let socket = Rect::from_center_size(button_origin, Vec2::splat(32.0));
    let _void = painter.rect_filled(socket, 1.0, Color32::from_rgb(2, 2, 3));
    paint_mesh(
        painter,
        socket.shrink(1.0),
        material_atlas::BUTTON_SHADOW,
        button_origin,
    );
    paint_mesh(
        painter,
        cell.shrink(1.0),
        material_atlas::PLATE_SHADOW,
        plate_origin,
    );
    let candidate = material_atlas::CELLS[row * material_atlas::COLUMN_COUNT + column];
    paint_mesh(painter, socket.shrink(1.0), candidate.button, button_origin);
    paint_mesh(painter, cell.shrink(1.0), candidate.plate, plate_origin);
    let _rim = painter.rect_stroke(
        socket,
        1.0,
        Stroke::new(1.0_f32, chrome::EDGE),
        StrokeKind::Inside,
    );
    let _coordinate = painter.text(
        Pos2::new(cell.center().x, cell.bottom() - 9.0),
        Align2::CENTER_CENTER,
        if production {
            format!("{} · PRODUCTION", material_coordinate(row, column))
        } else {
            material_coordinate(row, column)
        },
        font,
        if production {
            chrome::HOT
        } else {
            chrome::MUTED
        },
    );
}

fn paint_mesh(painter: &egui::Painter, clip: Rect, baked: BakedMesh, origin: Pos2) {
    let mut mesh = egui::Mesh::default();
    baked.stamp(&mut mesh, origin);
    let _mesh = painter
        .with_clip_rect(clip)
        .add(Shape::mesh(Arc::new(mesh)));
}

fn material_coordinate(row: usize, column: usize) -> String {
    let letter = (b'A' + row as u8) as char;
    format!("{letter}{}", column + 1)
}

fn working_mandate(ui: &mut egui::Ui) {
    heading(
        ui,
        "WORKING HYPOTHESIS",
        "Directional die relief and a yellower linear-light brass response are the current research vector, not a production decision.",
    );
    mandate(
        ui,
        "SHADOWED STAMP",
        "Soot belongs primarily on the key-occluded wall and toe. A uniform comic-book perimeter is the control, not the destination.",
    );
    mandate(
        ui,
        "LINEAR-LIGHT BLINN",
        "Evaluate illumination and exposure in linear light, then encode once. Retain a broad metallic lobe at native pixel scale instead of relying on a subpixel glint.",
    );
    mandate(
        ui,
        "YELLOWER BRASS",
        "Rotate the charge away from chocolate oxide while preserving dark-value reach, colored specular response, and coherent fixed-key geometry.",
    );
    mandate(
        ui,
        "TOOL-MARK ANISOTROPY",
        "Treat machining direction as an independent surface-role parameter. Compare it against the same isotropic material and environment before admitting it by part.",
    );
    mandate(
        ui,
        "FURNACE LINE",
        "Retain the low hot-horizon environment as an independent light-rig trial. Its reflected band must earn use without recoloring every surface into furnace light.",
    );
    mandate(
        ui,
        "AUTHORED DIES",
        "The Unicode scalar names the action; it need not remain the final outline. Fine symbols may require independent S/M/L cuts, and some may lawfully reject Small.",
    );
    mandate(
        ui,
        "JUDGMENT SURFACE",
        "Actual-size native WGPU is authoritative. Enlargements diagnose raster structure only. Browser publication remains gated on fixed-DPR native/browser image comparison.",
    );
}

fn mandate(ui: &mut egui::Ui, name: &str, body: &str) {
    let _row = egui::Frame::new()
        .fill(chrome::SURFACE)
        .stroke(Stroke::new(1.0_f32, chrome::EDGE))
        .inner_margin(12)
        .show(ui, |ui| {
            let _name = ui.label(chrome::eyebrow(name));
            let _body = ui.label(chrome::muted(body));
        });
    ui.add_space(8.0);
}
