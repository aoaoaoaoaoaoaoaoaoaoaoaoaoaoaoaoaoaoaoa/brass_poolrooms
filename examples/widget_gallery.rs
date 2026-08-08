#![expect(
    unused_crate_dependencies,
    reason = "the gallery deliberately consumes egui and wgpu through the crate's version-locked re-exports"
)]

pub mod exhibits;
mod support;

use anyhow::Result;
use dwemer_poolrooms::{chrome, egui, water::Surface};
use exhibits::{Checkboxes, Closures, Dates, Handles, Numbers, Pins, Sliders, Symbols};
use support::Exhibit;

#[derive(Default)]
struct Menagerie {
    closures: Closures,
    handles: Handles,
    symbols: Symbols,
    pins: Pins,
    numbers: Numbers,
    sliders: Sliders,
    checkboxes: Checkboxes,
    dates: Dates,
}

impl Exhibit for Menagerie {
    const TITLE: &'static str = "Poolrooms · crafted-widget menagerie";
    const SIZE: [f64; 2] = [820.0, 920.0];

    fn ui(&mut self, ui: &mut egui::Ui, water: &mut Surface) {
        let panel = egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(chrome::PAGE).inner_margin(28))
            .show_inside(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("crafted-widget-menagerie")
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.set_max_width(760.0);
                        let _title = ui.label(chrome::title("CRAFTED MECHANISMS"));
                        let _law = ui.label(chrome::muted(
                            "one foundry · one physical universe · one living water table",
                        ));
                        ui.add_space(28.0);
                        self.closures.show(ui, water);
                        ui.add_space(42.0);
                        self.handles.show(ui, water);
                        ui.add_space(42.0);
                        self.symbols.show(ui, water);
                        ui.add_space(42.0);
                        self.pins.show(ui);
                        ui.add_space(42.0);
                        self.numbers.show(ui, water);
                        ui.add_space(42.0);
                        self.sliders.show(ui, water);
                        ui.add_space(42.0);
                        self.checkboxes.show(ui, water);
                        ui.add_space(42.0);
                        self.dates.show(ui, water, "menagerie-date");
                        ui.add_space(28.0);
                    })
            });
        water.heave(ui.ctx(), panel.inner.state.offset.y);
    }
}

fn main() -> Result<()> {
    support::run(Menagerie::default())
}
