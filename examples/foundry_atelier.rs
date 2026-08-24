#![expect(
    unused_crate_dependencies,
    reason = "the atelier consumes egui and wgpu through the crate's version-locked re-exports"
)]

mod support;

use anyhow::Result;
use brass_poolrooms::{
    atelier::FoundryOpticsAtelier,
    chrome, egui,
    water::{Surface, Wetness},
};
use support::Exhibit;

#[derive(Default)]
struct FoundryAtelier(FoundryOpticsAtelier);

impl Exhibit for FoundryAtelier {
    const TITLE: &'static str = "Poolrooms · Foundry Optics Atelier";
    const SIZE: [f64; 2] = [1_180.0, 900.0];

    fn ui(&mut self, ui: &mut egui::Ui, water: &mut Surface) {
        water.set_wetness(Wetness::Dry);
        let _panel = egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(chrome::PAGE).inner_margin(26))
            .show(ui, |ui| {
                let _scroll = chrome::ScrewScroll::vertical()
                    .id_salt("foundry-optics-atelier")
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.set_max_width(1_118.0);
                        self.0.show(ui);
                        ui.add_space(24.0);
                    });
            });
    }
}

fn main() -> Result<()> {
    support::run(FoundryAtelier::default())
}
