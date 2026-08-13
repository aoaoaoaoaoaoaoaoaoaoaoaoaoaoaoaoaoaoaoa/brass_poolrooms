#![expect(
    unused_crate_dependencies,
    reason = "the gallery deliberately consumes egui and wgpu through the crate's version-locked re-exports"
)]

pub mod exhibits;
mod support;

use anyhow::Result;
use brass_poolrooms::{chrome, egui, water::Surface};
use exhibits::Handles;
use support::Exhibit;

impl Exhibit for Handles {
    const TITLE: &'static str = "Poolrooms · drag-handle gallery";
    const SIZE: [f64; 2] = [760.0, 420.0];

    fn ui(&mut self, ui: &mut egui::Ui, water: &mut Surface) {
        let _panel = egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(chrome::PAGE).inner_margin(28))
            .show(ui, |ui| self.show(ui, water));
    }
}

fn main() -> Result<()> {
    support::run(Handles)
}
