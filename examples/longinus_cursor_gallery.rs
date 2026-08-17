#![expect(
    unused_crate_dependencies,
    reason = "the gallery deliberately consumes egui and wgpu through the crate's version-locked re-exports"
)]

mod support;

use anyhow::Result;
use brass_poolrooms::{
    chrome::{self, ForgePin, LonginusCursor},
    egui::{self, Color32, CustomCursorImage, Pos2, Sense, Vec2},
    water::{Surface, Wetness},
};
use support::Exhibit;

#[derive(Default)]
struct LonginusForge;

impl Exhibit for LonginusForge {
    const TITLE: &'static str = "Poolrooms · native cursor forge";
    const SIZE: [f64; 2] = [900.0, 520.0];

    fn ui(&mut self, ui: &mut egui::Ui, water: &mut Surface) {
        water.set_wetness(Wetness::Dry);
        let _panel = egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(chrome::PAGE).inner_margin(32))
            .show(ui, |ui| {
                let _title = ui.label(chrome::title("NATIVE CURSOR FORGE"));
                let _law = ui.label(chrome::muted(
                    "shared bronze law · native 64 px projections · exact physical hotspots",
                ));
                ui.add_space(28.0);

                let _cursors = ui.horizontal(|ui| {
                    cursor_field(ui, "LONGINUS · FORK", LonginusCursor::image());
                    ui.add_space(18.0);
                    cursor_field(ui, "LARGE FORGE PIN", ForgePin::cursor_image());
                });
                ui.add_space(16.0);
                let _instruction = ui.label(chrome::eyebrow(
                    "ENTER A BLACK FIELD TO ARM ITS NATIVE CURSOR",
                ));
            });
    }
}

fn cursor_field(ui: &mut egui::Ui, label: &str, cursor: CustomCursorImage) {
    let _field = ui.vertical(|ui| {
        let _label = ui.label(chrome::eyebrow(label));
        ui.add_space(6.0);
        let frame = egui::Frame::new()
            .fill(chrome::CONTROL)
            .stroke(egui::Stroke::new(1.0_f32, chrome::EDGE_STRONG))
            .inner_margin(20)
            .show(ui, |ui| {
                let (rect, response) = ui.allocate_exact_size(Vec2::splat(320.0), Sense::hover());
                let scale = rect.width() / f32::from(cursor.size[0]);
                for (index, pixel) in cursor.rgba.chunks_exact(4).enumerate() {
                    if pixel[3] == 0 {
                        continue;
                    }
                    let x = (index % usize::from(cursor.size[0])) as f32;
                    let y = (index / usize::from(cursor.size[0])) as f32;
                    let min = Pos2::new(rect.left() + x * scale, rect.top() + y * scale);
                    let color =
                        Color32::from_rgba_unmultiplied(pixel[0], pixel[1], pixel[2], pixel[3]);
                    let _pixel = ui.painter().rect_filled(
                        egui::Rect::from_min_size(min, Vec2::splat(scale)),
                        0.0,
                        color,
                    );
                }
                if response.hovered() {
                    ui.ctx().set_cursor_image(Some(cursor));
                }
            });
        let _response = frame.response.on_hover_text("native cursor proving ground");
    });
}

fn main() -> Result<()> {
    support::run(LonginusForge)
}
