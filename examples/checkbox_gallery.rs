#![expect(
    unused_crate_dependencies,
    reason = "the gallery deliberately consumes egui and wgpu through the crate's version-locked re-exports"
)]

pub mod exhibits;
mod support;

use anyhow::Result;
use dwemer_poolrooms::{chrome, egui, water::Surface};
use exhibits::Checkboxes;
use support::Exhibit;

impl Exhibit for Checkboxes {
    const TITLE: &'static str = "Poolrooms · checkbox gallery";
    const SIZE: [f64; 2] = [620.0, 390.0];

    fn ui(&mut self, ui: &mut egui::Ui, water: &mut Surface) {
        let _panel = egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(chrome::PAGE).inner_margin(28))
            .show(ui, |ui| self.show(ui, water));
    }
}

fn main() -> Result<()> {
    support::run(Checkboxes::default())
}
