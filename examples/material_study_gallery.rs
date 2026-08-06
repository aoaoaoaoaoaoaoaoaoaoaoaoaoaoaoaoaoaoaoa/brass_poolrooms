#![expect(
    unused_crate_dependencies,
    reason = "the gallery deliberately consumes egui and wgpu through the crate's version-locked re-exports"
)]

mod support;

use std::sync::Arc;

use anyhow::Result;
use dwemer_poolrooms::{
    chrome,
    egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Vec2},
    water::{Surface, Wetness},
};
use support::Exhibit;

#[derive(Clone, Copy)]
struct BakedVertex {
    position: [f32; 2],
    color: [u8; 4],
}

#[derive(Clone, Copy)]
struct BakedMesh {
    vertices: &'static [BakedVertex],
    indices: &'static [u32],
}

#[derive(Clone, Copy)]
struct BakedStudyCell {
    button: BakedMesh,
    plate: BakedMesh,
}

mod atlas {
    use super::{BakedMesh, BakedStudyCell, BakedVertex};

    include!(concat!(env!("OUT_DIR"), "/material_study_atlas.rs"));
}

#[derive(Default)]
struct MaterialStudy;

impl Exhibit for MaterialStudy {
    const TITLE: &'static str = "Poolrooms · dark-bronze material forge";
    const SIZE: [f64; 2] = [1_120.0, 760.0];

    fn ui(&mut self, ui: &mut egui::Ui, water: &mut Surface) {
        // Optical motion would make spatially separated samples incomparable.
        // The study still traverses the production WGPU/egui stack, but on a
        // dry calibration field.
        water.set_wetness(Wetness::Dry);
        let _panel = egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(chrome::PAGE).inner_margin(28))
            .show_inside(ui, show_study);
    }
}

fn show_study(ui: &mut egui::Ui) {
    let title = egui::RichText::new("DARK-BRONZE MATERIAL FORGE")
        .color(chrome::TEXT)
        .size(20.0)
        .strong();
    let _title = ui.label(title);
    let production = cell_coordinate(atlas::PRODUCTION_ROW, atlas::PRODUCTION_COLUMN);
    let _law = ui.label(
        egui::RichText::new(format!(
            "FIXED EYE · COMMON 60° KEY · IDENTICAL 32 PT PLUNGER AND PLATE · {production} IS PRODUCTION"
        ))
        .color(chrome::MUTED)
        .size(12.0),
    );
    ui.add_space(16.0);

    let size = Vec2::new(ui.available_width(), 560.0);
    let (canvas, _response) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter_at(canvas);
    paint_matrix(&painter, canvas);

    ui.add_space(8.0);
    let _legend = ui.label(
        egui::RichText::new(
            "LEFT: STANDARD CLICK PLUNGER  ·  RIGHT: SHALLOW FORGED PLATE  ·  ROWS TIGHTEN AND AMPLIFY THE REFLECTION; COLUMNS ALTER EXPOSURE ONLY",
        )
        .color(chrome::MUTED)
        .size(11.0),
    );
}

fn paint_matrix(painter: &egui::Painter, canvas: Rect) {
    const LABEL_WIDTH: f32 = 168.0;
    const HEADER_HEIGHT: f32 = 58.0;
    const ROW_HEIGHT: f32 = 98.0;
    const GUTTER: f32 = 7.0;

    let grid_left = canvas.left() + LABEL_WIDTH;
    let grid_width = canvas.width() - LABEL_WIDTH;
    let column_width = (grid_width - GUTTER * (atlas::COLUMN_COUNT.saturating_sub(1)) as f32)
        / atlas::COLUMN_COUNT as f32;
    let label_font = FontId::monospace(12.0);
    let small_font = FontId::monospace(10.5);
    let cell_font = FontId::monospace(11.0);

    let _concentration_axis = painter.text(
        Pos2::new(canvas.left(), canvas.top() + 12.0),
        Align2::LEFT_TOP,
        "SPECULAR CONTRAST ↓",
        label_font.clone(),
        chrome::HOT,
    );
    let _lightness_axis = painter.text(
        Pos2::new(grid_left, canvas.top() + 12.0),
        Align2::LEFT_TOP,
        "OVERALL LIGHTNESS / EXPOSURE →",
        label_font.clone(),
        chrome::HOT,
    );

    for column in 0..atlas::COLUMN_COUNT {
        let center_x = grid_left + column as f32 * (column_width + GUTTER) + column_width * 0.5;
        let exposure = atlas::EXPOSURES[column];
        let delta = (exposure - 1.0) * 100.0;
        let heading = format!("{} · {exposure:.2}×", column + 1);
        let subtitle = if column == atlas::PRODUCTION_COLUMN {
            "PRODUCTION".to_owned()
        } else {
            format!("{delta:+.0}%")
        };
        let _heading = painter.text(
            Pos2::new(center_x, canvas.top() + 31.0),
            Align2::CENTER_TOP,
            heading,
            label_font.clone(),
            chrome::TEXT,
        );
        let _subtitle = painter.text(
            Pos2::new(center_x, canvas.top() + 46.0),
            Align2::CENTER_TOP,
            subtitle,
            small_font.clone(),
            if column == atlas::PRODUCTION_COLUMN {
                chrome::HOT
            } else {
                chrome::MUTED
            },
        );
    }

    for row in 0..atlas::ROW_COUNT {
        let top = canvas.top() + HEADER_HEIGHT + row as f32 * (ROW_HEIGHT + GUTTER);
        let letter = (b'A' + row as u8) as char;
        let _row_name = painter.text(
            Pos2::new(canvas.left(), top + 27.0),
            Align2::LEFT_CENTER,
            format!("{letter}  {}", atlas::ROW_NAMES[row]),
            label_font.clone(),
            if row == atlas::PRODUCTION_ROW {
                chrome::HOT
            } else {
                chrome::TEXT
            },
        );
        let _exponent = painter.text(
            Pos2::new(canvas.left(), top + 45.0),
            Align2::LEFT_CENTER,
            format!("GLINT n = {:.0}", atlas::GLINT_EXPONENTS[row]),
            small_font.clone(),
            chrome::MUTED,
        );

        for column in 0..atlas::COLUMN_COUNT {
            let left = grid_left + column as f32 * (column_width + GUTTER);
            let cell =
                Rect::from_min_size(Pos2::new(left, top), Vec2::new(column_width, ROW_HEIGHT));
            paint_cell(painter, cell, row, column, cell_font.clone());
        }
    }
}

fn paint_cell(painter: &egui::Painter, cell: Rect, row: usize, column: usize, font: FontId) {
    let production = row == atlas::PRODUCTION_ROW && column == atlas::PRODUCTION_COLUMN;
    let stroke = if production {
        Stroke::new(1.5_f32, chrome::HOT)
    } else {
        Stroke::new(1.0_f32, chrome::EDGE)
    };
    let _bed = painter.rect_filled(cell, 1.0, chrome::SURFACE);
    let _edge = painter.rect_stroke(cell, 1.0, stroke, StrokeKind::Inside);

    let artifact_y = cell.top() + 42.0;
    let button_origin = Pos2::new(cell.center().x - 27.0, artifact_y);
    let plate_origin = Pos2::new(cell.center().x + 27.0, artifact_y);
    let socket = Rect::from_center_size(button_origin, Vec2::splat(32.0));
    paint_socket_bed(painter, socket);

    let candidate = atlas::CELLS[row * atlas::COLUMN_COUNT + column];
    paint_mesh(
        painter,
        socket.shrink(1.0),
        atlas::BUTTON_SHADOW,
        button_origin,
    );
    paint_mesh(painter, cell.shrink(1.0), atlas::PLATE_SHADOW, plate_origin);
    paint_mesh(painter, socket.shrink(1.0), candidate.button, button_origin);
    paint_mesh(painter, cell.shrink(1.0), candidate.plate, plate_origin);
    paint_socket_rim(painter, socket);

    let coordinate = cell_coordinate(row, column);
    let _coordinate = painter.text(
        Pos2::new(cell.center().x, cell.bottom() - 9.0),
        Align2::CENTER_CENTER,
        if production {
            format!("{coordinate} · PRODUCTION")
        } else {
            coordinate
        },
        font,
        if production {
            chrome::HOT
        } else {
            chrome::MUTED
        },
    );
}

fn cell_coordinate(row: usize, column: usize) -> String {
    let letter = (b'A' + row as u8) as char;
    format!("{letter}{}", column + 1)
}

fn paint_mesh(painter: &egui::Painter, clip: Rect, baked: BakedMesh, origin: Pos2) {
    let mut mesh = egui::Mesh::default();
    mesh.vertices.reserve(baked.vertices.len());
    mesh.indices.reserve(baked.indices.len());
    for vertex in baked.vertices {
        let [x, y] = vertex.position;
        let [r, g, b, a] = vertex.color;
        mesh.colored_vertex(
            origin + Vec2::new(x, y),
            Color32::from_rgba_unmultiplied(r, g, b, a),
        );
    }
    mesh.indices.extend_from_slice(baked.indices);
    let _mesh = painter
        .with_clip_rect(clip)
        .add(Shape::mesh(Arc::new(mesh)));
}

fn paint_socket_bed(painter: &egui::Painter, socket: Rect) {
    let _void = painter.rect_filled(socket, 1.0, Color32::from_rgb(2, 2, 3));
    let _shadow = painter.line_segment(
        [socket.left_top(), socket.right_top()],
        Stroke::new(1.6_f32, Color32::from_rgb(1, 1, 2)),
    );
    let _catch = painter.line_segment(
        [socket.left_bottom(), socket.right_bottom()],
        Stroke::new(1.0_f32, chrome::EDGE),
    );
}

fn paint_socket_rim(painter: &egui::Painter, socket: Rect) {
    let _rim = painter.rect_stroke(
        socket,
        1.0,
        Stroke::new(1.0_f32, chrome::EDGE),
        StrokeKind::Inside,
    );
}

fn main() -> Result<()> {
    support::run(MaterialStudy)
}
