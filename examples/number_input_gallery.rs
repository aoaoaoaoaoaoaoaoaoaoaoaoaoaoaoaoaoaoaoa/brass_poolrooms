#![expect(
    unused_crate_dependencies,
    reason = "the gallery deliberately consumes egui and wgpu through the crate's version-locked re-exports"
)]

pub mod exhibits;
mod support;

use anyhow::Result;
use brass_poolrooms::{chrome, egui, water::Surface};
use exhibits::Numbers;
use support::Exhibit;

impl Exhibit for Numbers {
    const TITLE: &'static str = "Poolrooms · numerical-thumbwheel gallery";
    const SIZE: [f64; 2] = [680.0, 350.0];

    fn ui(&mut self, ui: &mut egui::Ui, water: &mut Surface) {
        let _panel = egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(chrome::PAGE).inner_margin(28))
            .show(ui, |ui| self.show(ui, water));
    }
}

fn main() -> Result<()> {
    support::run(Numbers::default())
}
