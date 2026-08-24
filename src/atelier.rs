//! Native Foundry judgment surfaces. This module is development apparatus,
//! feature-gated out of production dependencies, and rendered by the same
//! egui/WGPU path as shipped mechanisms.

use std::sync::Arc;

use crate::{
    chrome::{self, ForgedMesh, ForgedVertex, MechanismSize, Monoglyph, MonoglyphFinish, Symbol},
    egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Vec2},
};

type BakedVertex = ForgedVertex;
type BakedMesh = ForgedMesh;

#[derive(Clone, Copy)]
struct BakedStudyCell {
    button: BakedMesh,
    plate: BakedMesh,
}

#[derive(Clone, Copy)]
struct BakedOpticsCell {
    button: BakedMesh,
    plate: BakedMesh,
    sphere: BakedMesh,
    cylinder: BakedMesh,
}

mod material_atlas {
    use super::{BakedMesh, BakedStudyCell, BakedVertex};

    include!(concat!(env!("OUT_DIR"), "/material_study_atlas.rs"));
}

mod optics_atlas {
    use super::{BakedMesh, BakedOpticsCell, BakedVertex};

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
    Witness,
    Soot,
    Dies,
    Material,
    Mandate,
}

impl Bench {
    const ALL: [Self; 5] = [
        Self::Witness,
        Self::Soot,
        Self::Dies,
        Self::Material,
        Self::Mandate,
    ];

    const fn name(self) -> &'static str {
        match self {
            Self::Witness => "PRODUCTION WITNESS",
            Self::Soot => "SOOT KEYLINES",
            Self::Dies => "DIE TOOLING",
            Self::Material => "MATERIAL LAW",
            Self::Mandate => "WORKING MANDATE",
        }
    }
}

/// Persistent state for the feature-gated Foundry Optics Atelier.
pub struct FoundryOpticsAtelier {
    bench: Bench,
    soot_symbol: Symbol,
}

impl Default for FoundryOpticsAtelier {
    fn default() -> Self {
        Self {
            bench: Bench::Witness,
            soot_symbol: Symbol::Settings,
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
        let _tabs = ui.horizontal_wrapped(|ui| {
            for bench in Bench::ALL {
                let _tab = ui.selectable_value(&mut self.bench, bench, bench.name());
            }
        });
        ui.add_space(18.0);

        match self.bench {
            Bench::Witness => production_witness(ui),
            Bench::Soot => self.soot_bench(ui),
            Bench::Dies => self.die_bench(ui),
            Bench::Material => material_bench(ui),
            Bench::Mandate => working_mandate(ui),
        }
    }

    fn soot_bench(&mut self, ui: &mut egui::Ui) {
        heading(
            ui,
            "SOOT KEYLINE CALIBRATION",
            "The candidates alter the real monoglyph painter. Width is measured in physical pixels; tone is the exact 8-bit sRGB value submitted to egui.",
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
                    soot_cell(ui, self.soot_symbol, eighth_pixels, srgb);
                }
            });
            ui.add_space(6.0);
        }
        ui.add_space(8.0);
        let _note = ui.label(chrome::muted(
            "The 1.00 px / sRGB 0 cell is production. Fractional candidates remain subpixel offsets in the same font-atlas composite; they are not SDF reconstructions.",
        ));
    }

    fn die_bench(&mut self, ui: &mut egui::Ui) {
        heading(
            ui,
            "GLYPH DIE TOOLING",
            "This bench isolates the current hinted font outline from finish and gauge. It is the admission surface for authored S/M/L cuts, not evidence that one Unicode outline should serve every die.",
        );
        let _symbols = ui.horizontal_wrapped(|ui| {
            let _label = ui.label(chrome::eyebrow("MARK"));
            for symbol in DIFFICULT_SYMBOLS {
                let _choice = ui.selectable_value(&mut self.soot_symbol, symbol, symbol.name());
            }
        });
        ui.add_space(14.0);
        for finish in MonoglyphFinish::ALL {
            die_finish_row(ui, self.soot_symbol, finish);
            ui.add_space(8.0);
        }
        ui.add_space(12.0);
        let _frontier = egui::Frame::new()
            .fill(chrome::SURFACE)
            .stroke(Stroke::new(1.0_f32, chrome::HOT))
            .inner_margin(10)
            .show(ui, |ui| {
                let _title = ui.label(chrome::eyebrow("SHADOWED-STAMP FRONTIER"));
                let _body = ui.label(chrome::muted(
                    "The directional stamp remains an ideation result until the Foundry owns a real outline-to-relief compiler. The next lawful step is a mask/contour source that admits symbol-specific S/M/L cuts, then bakes wall normals, toe occlusion, and floor material through the selected lighting law.",
                ));
            });
    }
}

fn heading(ui: &mut egui::Ui, title: &str, law: &str) {
    let _title = ui.label(chrome::title(title));
    let _law = ui.label(chrome::muted(law));
    ui.add_space(14.0);
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

fn soot_cell(ui: &mut egui::Ui, symbol: Symbol, eighth_pixels: u8, srgb: u8) {
    let production = eighth_pixels == 8 && srgb == 0;
    let _cell = egui::Frame::new()
        .fill(chrome::SURFACE)
        .stroke(Stroke::new(
            if production { 1.5 } else { 1.0 },
            if production {
                chrome::HOT
            } else {
                chrome::EDGE
            },
        ))
        .inner_margin(8)
        .show(ui, |ui| {
            ui.set_width(176.0);
            let _label = ui.label(chrome::eyebrow(format!(
                "{:.2} PX · sRGB {srgb}{}",
                f32::from(eighth_pixels) / 8.0,
                if production { " · PRODUCTION" } else { "" }
            )));
            ui.add_space(6.0);
            let _specimens = ui.horizontal(|ui| {
                for size in MechanismSize::ALL {
                    let _button = Monoglyph::symbol(symbol)
                        .size(size)
                        .study_soot(eighth_pixels, srgb)
                        .show(ui)
                        .on_hover_text(format!(
                            "{} · {} · {:.2} physical-pixel keyline · sRGB {srgb}",
                            symbol.name(),
                            size_name(size),
                            f32::from(eighth_pixels) / 8.0
                        ));
                }
            });
        });
}

fn die_finish_row(ui: &mut egui::Ui, symbol: Symbol, finish: MonoglyphFinish) {
    let _row = egui::Frame::new()
        .fill(chrome::SURFACE)
        .stroke(Stroke::new(1.0_f32, chrome::EDGE))
        .inner_margin(10)
        .show(ui, |ui| {
            let _content = ui.horizontal(|ui| {
                let _finish = ui.add_sized(
                    [130.0, MechanismSize::Large.side()],
                    egui::Label::new(chrome::eyebrow(finish.name())),
                );
                for size in MechanismSize::ALL {
                    let _button = Monoglyph::symbol(symbol)
                        .finish(finish)
                        .size(size)
                        .show(ui)
                        .on_hover_text(format!(
                            "{} · {} · {}",
                            symbol.name(),
                            finish.name(),
                            size_name(size)
                        ));
                    ui.add_space(10.0);
                }
                let _register = ui.label(chrome::muted(match finish {
                    MonoglyphFinish::BrightCut => {
                        "fresh-bronze key-facing wall · soot-black groove floor"
                    }
                    MonoglyphFinish::Void => "steep wall · soot-black flat floor",
                    MonoglyphFinish::Danger => "steep wall · vermilion floor · soot primer",
                    MonoglyphFinish::Love => "steep wall · pink floor · soot primer",
                }));
            });
        });
}

const fn size_name(size: MechanismSize) -> &'static str {
    match size {
        MechanismSize::Small => "SMALL",
        MechanismSize::Medium => "MEDIUM",
        MechanismSize::Large => "LARGE",
    }
}

fn material_bench(ui: &mut egui::Ui) {
    heading(
        ui,
        "LINEAR-LIGHT BRASS HYPOTHESES",
        "One variable changes within each row. Illumination is evaluated in linear light with a filmic shoulder, then baked to the same egui meshes used by production.",
    );
    optics_group(ui, "CHARGE · ISOTROPIC · BLACK ROOM", &[0, 1, 2, 3]);
    optics_group(ui, "TOOL MARK · YELLOW BRASS · BLACK ROOM", &[2, 4, 5]);
    optics_group(ui, "LIGHT RIG · YELLOW BRASS · ISOTROPIC", &[2, 6, 7]);
    optics_group(ui, "PROVOCATION · TWO VARIABLES", &[8]);
    let _limit = ui.label(chrome::muted(
        "Candidate illumination is linear at Foundry vertices; egui still performs Gouraud interpolation on encoded colors. These are exact candidate meshes, not a claim of a fully linear framebuffer.",
    ));
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

fn optics_group(ui: &mut egui::Ui, name: &str, candidates: &[usize]) {
    let _name = ui.label(chrome::eyebrow(name));
    ui.add_space(5.0);
    let _row = ui.horizontal(|ui| {
        for &index in candidates {
            optics_coupon(ui, index);
        }
    });
    ui.add_space(12.0);
}

fn optics_coupon(ui: &mut egui::Ui, index: usize) {
    let (rect, _response) = ui.allocate_exact_size(Vec2::new(236.0, 94.0), Sense::hover());
    let painter = ui.painter_at(rect);
    let _bed = painter.rect_filled(rect, 1.0, chrome::SURFACE);
    let _edge = painter.rect_stroke(
        rect,
        1.0,
        Stroke::new(1.0_f32, chrome::EDGE),
        StrokeKind::Inside,
    );
    let _name = painter.text(
        rect.left_top() + Vec2::new(9.0, 8.0),
        Align2::LEFT_TOP,
        optics_atlas::NAMES[index],
        FontId::monospace(10.5),
        chrome::TEXT,
    );
    let button_origin = Pos2::new(rect.left() + 42.0, rect.top() + 59.0);
    let plate_origin = Pos2::new(rect.left() + 85.0, rect.top() + 59.0);
    let socket = Rect::from_center_size(button_origin, Vec2::splat(32.0));
    let _void = painter.rect_filled(socket, 1.0, Color32::from_rgb(2, 2, 3));
    paint_mesh(
        &painter,
        socket.shrink(1.0),
        optics_atlas::BUTTON_SHADOW,
        button_origin,
    );
    paint_mesh(
        &painter,
        rect.shrink(1.0),
        optics_atlas::PLATE_SHADOW,
        plate_origin,
    );
    let candidate = optics_atlas::CELLS[index];
    paint_mesh(
        &painter,
        socket.shrink(1.0),
        candidate.button,
        button_origin,
    );
    paint_mesh(&painter, rect.shrink(1.0), candidate.plate, plate_origin);
    paint_mesh(
        &painter,
        rect.shrink(1.0),
        candidate.sphere,
        Pos2::new(rect.left() + 143.0, rect.top() + 59.0),
    );
    paint_mesh(
        &painter,
        rect.shrink(1.0),
        candidate.cylinder,
        Pos2::new(rect.left() + 198.0, rect.top() + 59.0),
    );
    let _rim = painter.rect_stroke(
        socket,
        1.0,
        Stroke::new(1.0_f32, chrome::EDGE),
        StrokeKind::Inside,
    );
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
    let label_font = FontId::monospace(12.0);
    let small_font = FontId::monospace(10.5);
    let cell_font = FontId::monospace(11.0);

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
