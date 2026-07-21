#![expect(
    unused_crate_dependencies,
    reason = "the gallery deliberately consumes egui and wgpu through the crate's version-locked re-exports"
)]

mod support;

use anyhow::Result;
use dwemer_poolrooms::{
    chrome::{self, Checkbox},
    egui,
    water::Surface,
};

use support::Exhibit;

struct Checkboxes {
    live: [bool; 2],
    guarded: [bool; 2],
}

impl Default for Checkboxes {
    fn default() -> Self {
        Self {
            live: [false, true],
            guarded: [false, true],
        }
    }
}

impl Exhibit for Checkboxes {
    const TITLE: &'static str = "Poolrooms · checkbox gallery";
    const SIZE: [f64; 2] = [620.0, 390.0];

    fn ui(&mut self, ui: &mut egui::Ui, water: &mut Surface) {
        let _panel = egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(chrome::PAGE).inner_margin(28))
            .show_inside(ui, |ui| {
                let _title = ui.label(chrome::title("LATCHING PLUNGERS"));
                let _law = ui.label(chrome::muted(
                    "concave crown · stiff spring · swept-volume water coupling",
                ));
                ui.add_space(23.0);

                let _live = ui.label(chrome::eyebrow("LIVE MECHANISMS"));
                ui.add_space(5.0);
                let live_z = ui.horizontal(|ui| {
                    let intake = Checkbox::new(&mut self.live[0], "INTAKE PUMP").show(ui);
                    water.checkbox(&intake);
                    ui.add_space(36.0);
                    let return_pump = Checkbox::new(&mut self.live[1], "RETURN PUMP").show(ui);
                    water.checkbox(&return_pump);
                    [intake.elevation(), return_pump.elevation()]
                });
                let _telemetry = ui.label(chrome::muted(format!(
                    "LIVE CROWN Z · INTAKE {:+06.2} pt · RETURN {:+06.2} pt",
                    live_z.inner[0], live_z.inner[1]
                )));

                ui.add_space(18.0);
                let _guarded = ui.label(chrome::eyebrow("HAND-GUARDED · STATE REMAINS VISIBLE"));
                ui.add_space(5.0);
                let _row = ui.horizontal(|ui| {
                    let _off = ui.add_enabled_ui(false, |ui| {
                        Checkbox::new(&mut self.guarded[0], "LOCKED OFF").show(ui)
                    });
                    ui.add_space(45.0);
                    let _on = ui.add_enabled_ui(false, |ui| {
                        Checkbox::new(&mut self.guarded[1], "LOCKED ON").show(ui)
                    });
                });
            });
    }
}

fn main() -> Result<()> {
    support::run(Checkboxes::default())
}
