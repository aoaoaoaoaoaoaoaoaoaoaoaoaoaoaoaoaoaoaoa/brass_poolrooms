#![expect(
    unused_crate_dependencies,
    reason = "the gallery deliberately consumes egui and wgpu through the crate's version-locked re-exports"
)]

pub mod exhibits;
mod support;

use anyhow::Result;
use dwemer_poolrooms::{chrome, egui, water::Surface};
use exhibits::Dates;
use support::Exhibit;

impl Exhibit for Dates {
    const TITLE: &'static str = "Poolrooms · date-spool gallery";
    const SIZE: [f64; 2] = [760.0, 600.0];

    fn ui(&mut self, ui: &mut egui::Ui, water: &mut Surface) {
        let _panel = egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(chrome::PAGE).inner_margin(28))
            .show_inside(ui, |ui| self.show(ui, water, "gallery-date"));
    }
}

fn main() -> Result<()> {
    support::run(Dates::default())
}
