use std::hash::Hash;

use dwemer_poolrooms::{
    chrome::{
        self, Checkbox, CornerClose, Coupled, CouplingGap, DateReels, DateSpool, DragHandle,
        GregorianDay, LabelSide, MechanismSize, Monoglyph, NumberInput, Rail, WheelPlane,
    },
    egui,
    water::Surface,
};

pub struct Closures {
    open: bool,
}

impl Default for Closures {
    fn default() -> Self {
        Self { open: true }
    }
}

impl Closures {
    pub fn show(&mut self, ui: &mut egui::Ui, water: &mut Surface) {
        let _title = ui.label(chrome::title("CORNER CLOSURE"));
        let _law = ui.label(chrome::muted(
            "corner-bisected socket · die-sunk X · sprung swept-volume coupling",
        ));
        ui.add_space(18.0);

        if self.open {
            ui.add_space(CornerClose::HEADROOM);
            let pane = egui::Frame::new()
                .fill(chrome::SURFACE)
                .stroke(egui::Stroke::new(1.0_f32, chrome::EDGE_STRONG))
                .inner_margin(egui::Margin::symmetric(14, 11))
                .show(ui, |ui| {
                    ui.set_min_width(310.0);
                    let _heading = ui.label(chrome::eyebrow("PRESSURE VESSEL 07"));
                    ui.add_space(7.0);
                    let _reading = ui.label(chrome::section_title("NOMINAL · 2.41 bar"));
                    let _detail = ui.label(chrome::muted(
                        "The pane corner passes through the closure axis.",
                    ));
                });
            let close = CornerClose::new()
                .show(ui, pane.response.rect, "gallery-close")
                .on_hover_text("close pane");
            water.corner_close(&close);
            if close.clicked() {
                self.open = false;
            }
            let _telemetry = ui.label(chrome::muted(format!(
                "CLOSURE CROWN Z · {:+06.2} pt",
                close.elevation()
            )));
        } else {
            let _sealed = ui.label(chrome::eyebrow("PANE WITHDRAWN"));
            let _rearm = ui.horizontal(|ui| {
                let restore = Monoglyph::new('↺').show(ui).on_hover_text("restore pane");
                water.monoglyph(&restore);
                if restore.clicked() {
                    self.open = true;
                }
                let _label = ui.label(chrome::muted("restore the exhibit"));
            });
        }
    }
}

#[derive(Default)]
pub struct Handles;

impl Handles {
    pub fn show(&mut self, ui: &mut egui::Ui, water: &mut Surface) {
        let _title = ui.label(chrome::title("DRAG HANDLES · TWIN-TIE ASSEMBLIES"));
        let _law = ui.label(chrome::muted(
            "knurled friction pad · rigid bail · optional sprung bail hinge",
        ));
        ui.add_space(20.0);

        let _friction = ui.horizontal(|ui| {
            let assembly = Coupled::horizontal_with_gap(
                ui,
                CouplingGap::MINIMUM,
                |ui| {
                    DragHandle::friction_pad()
                        .size(MechanismSize::Small)
                        .show(ui)
                },
                |ui| {
                    Coupled::horizontal_with_gap(
                        ui,
                        CouplingGap::MINIMUM,
                        |ui| Monoglyph::new('×').size(MechanismSize::Small).show(ui),
                        |ui| Monoglyph::new('▣').size(MechanismSize::Small).show(ui),
                    )
                },
            );
            water.drag_handle(&assembly.left);
            water.monoglyph(&assembly.right.left);
            water.monoglyph(&assembly.right.right);
            let _telemetry = ui.label(chrome::muted(
                "SMALL FRICTION PAD · 10×20 pt · RIGID · 2 pt TIES",
            ));
        });
        ui.add_space(10.0);

        bail_exhibit(
            ui,
            water,
            "MEDIUM RIGID BAIL",
            MechanismSize::Medium,
            CouplingGap::new(4.5),
            DragHandle::rigid_bail,
        );
        ui.add_space(10.0);
        bail_exhibit(
            ui,
            water,
            "LARGE FOLDING BAIL",
            MechanismSize::Large,
            CouplingGap::STANDARD,
            DragHandle::folding_bail,
        );
        ui.add_space(10.0);
    }
}

pub struct Numbers {
    fine: f64,
    count: i32,
    upper_stop: f32,
}

impl Default for Numbers {
    fn default() -> Self {
        Self {
            fine: 0.375,
            count: 42,
            upper_stop: 1.0,
        }
    }
}

impl Numbers {
    pub fn show(&mut self, ui: &mut egui::Ui, water: &mut Surface) {
        let _title = ui.label(chrome::title("NUMERICAL THUMBWHEELS"));
        let _law = ui.label(chrome::muted(
            "scalloped oblate wheel · exact scalar register · torsional hard stops",
        ));
        ui.add_space(20.0);

        let _fine = ui.horizontal(|ui| {
            let response = NumberInput::new(&mut self.fine, -2.0..=2.0, 0.005, 3)
                .wheel_plane(WheelPlane::YZ)
                .register_width(82.0)
                .show(ui)
                .on_hover_text("scroll wheel · double-click register for exact entry");
            water.number_input(&response);
            let _legend = ui.label(chrome::muted("YZ · f64 · −2…2 · QUANTUM 0.005 · 3 PLACES"));
        });

        ui.add_space(12.0);
        let _integer = ui.horizontal(|ui| {
            let response = NumberInput::new(&mut self.count, 0..=99, 1, 0)
                .wheel_plane(WheelPlane::XZ)
                .register_width(62.0)
                .show(ui)
                .on_hover_text("scroll wheel · double-click register for exact entry");
            water.number_input(&response);
            let _legend = ui.label(chrome::muted("XZ · i32 · 0…99 · QUANTUM 1 · INTEGER"));
        });

        ui.add_space(12.0);
        let _limit = ui.horizontal(|ui| {
            let response = NumberInput::new(&mut self.upper_stop, -1.0..=1.0, 0.1, 1)
                .wheel_plane(WheelPlane::YZ)
                .show(ui)
                .on_hover_text("scroll upward against the upper hard stop");
            water.number_input(&response);
            let contact = response
                .refusal()
                .map_or("ARMED", |strike| match strike.bound() {
                    chrome::NumberBound::Minimum => "LOWER STOP",
                    chrome::NumberBound::Maximum => "UPPER STOP",
                });
            let _legend = ui.label(chrome::muted(format!("LIMIT REFUSAL · −1…1 · {contact}")));
        });

        ui.add_space(8.0);
        let _instruction = ui.label(chrome::eyebrow(
            "HOVER A WHEEL TO SCROLL · DOUBLE-CLICK A REGISTER TO TYPE",
        ));
    }
}

fn bail_exhibit(
    ui: &mut egui::Ui,
    water: &mut Surface,
    name: &str,
    size: MechanismSize,
    gap: CouplingGap,
    forge: fn() -> DragHandle,
) {
    let _row = ui.horizontal(|ui| {
        let assembly = Coupled::horizontal_with_gap(
            ui,
            gap,
            |ui| forge().size(size).show(ui),
            |ui| {
                Coupled::horizontal_with_gap(
                    ui,
                    gap,
                    |ui| Monoglyph::new('×').size(size).show(ui),
                    |ui| Monoglyph::new('▣').size(size).show(ui),
                )
            },
        );
        water.drag_handle(&assembly.left);
        water.monoglyph(&assembly.right.left);
        water.monoglyph(&assembly.right.right);
        let side = size.side() as u8;
        let _telemetry = ui.label(chrome::muted(format!(
            "{name} · {side}×{side} pt · {:.1} pt TIES · {:05.1}° · DRAG {:+05.1},{:+05.1}",
            gap.points(),
            assembly.left.angle().to_degrees(),
            assembly.left.drag_delta().x,
            assembly.left.drag_delta().y,
        )));
    });
}

pub struct Sliders {
    free: u16,
    barred: u16,
    ceiling: u16,
}

impl Default for Sliders {
    fn default() -> Self {
        Self {
            free: 4,
            barred: 4,
            ceiling: 6,
        }
    }
}

impl Sliders {
    pub fn show(&mut self, ui: &mut egui::Ui, water: &mut Surface) {
        let _title = ui.label(chrome::title("POOLROOMS SLIDER"));
        let _law = ui.label(chrome::muted(
            "drag, click or wheel · offset slider-crank · swept-volume water coupling",
        ));
        ui.add_space(24.0);

        let _label = ui.horizontal(|ui| {
            let _name = ui.label(chrome::eyebrow("UNRESTRICTED TRAVEL · SIX STATIONS"));
            let _value = ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(chrome::muted(format!("{}", self.free)))
            });
        });
        let rail = Rail::new(&mut self.free, 0..=10)
            .detents(6)
            .wheel()
            .width(ui.available_width())
            .show(ui);
        water.rail(&rail);

        ui.add_space(28.0);
        let _label = ui.horizontal(|ui| {
            let _name = ui.label(chrome::eyebrow("DYNAMIC ADMISSIBLE TRAVEL"));
            let _value = ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(chrome::muted(format!(
                    "{}  ∈  0..{}",
                    self.barred, self.ceiling
                )))
            });
        });
        let rail = Rail::new(&mut self.barred, 0..=10)
            .allowed(0..=self.ceiling)
            .detents(11)
            .wheel()
            .width(ui.available_width())
            .show(ui);
        water.rail(&rail);

        ui.add_space(15.0);
        let _gate = ui.horizontal(|ui| {
            let _caption = ui.label(chrome::muted("external stop"));
            let decrement = Monoglyph::new('−').show(ui);
            water.monoglyph(&decrement);
            if decrement.clicked() {
                self.ceiling = self.ceiling.saturating_sub(1);
            }
            let increment = Monoglyph::new('+').show(ui);
            water.monoglyph(&increment);
            if increment.clicked() {
                self.ceiling = self.ceiling.saturating_add(1).min(10);
            }
        });
    }
}

pub struct Checkboxes {
    live: [bool; 3],
    guarded: [bool; 3],
}

impl Default for Checkboxes {
    fn default() -> Self {
        Self {
            live: [false, false, true],
            guarded: [false, true, false],
        }
    }
}

impl Checkboxes {
    pub fn show(&mut self, ui: &mut egui::Ui, water: &mut Surface) {
        let _title = ui.label(chrome::title("LATCHING PLUNGERS"));
        let _law = ui.label(chrome::muted(
            "three forged gauges · fixed guard stock · stiff spring · swept-volume coupling",
        ));
        ui.add_space(23.0);

        let _live = ui.label(chrome::eyebrow("LIVE MECHANISMS"));
        ui.add_space(5.0);
        let live_z = ui.horizontal(|ui| {
            let bare = Checkbox::without_text(&mut self.live[0])
                .size(MechanismSize::Small)
                .show(ui);
            water.checkbox(&bare);
            ui.add_space(24.0);
            let intake = Checkbox::new(&mut self.live[1], "INTAKE PUMP")
                .size(MechanismSize::Medium)
                .show(ui);
            water.checkbox(&intake);
            ui.add_space(36.0);
            let return_pump = Checkbox::new(&mut self.live[2], "RETURN PUMP")
                .label_side(LabelSide::Left)
                .size(MechanismSize::Large)
                .show(ui);
            water.checkbox(&return_pump);
            [
                bare.elevation(),
                intake.elevation(),
                return_pump.elevation(),
            ]
        });
        let _telemetry = ui.label(chrome::muted(format!(
            "LIVE CROWN Z · BARE {:+06.2} · INTAKE {:+06.2} · RETURN {:+06.2} pt",
            live_z.inner[0], live_z.inner[1], live_z.inner[2]
        )));

        ui.add_space(18.0);
        let _guarded = ui.label(chrome::eyebrow("HAND-GUARDED · STATE REMAINS VISIBLE"));
        ui.add_space(5.0);
        let _row = ui.horizontal(|ui| {
            let _small = ui.vertical(|ui| {
                let _label = ui.label(chrome::muted("SMALL · 2×2 · OFF"));
                let _guard = ui.add_enabled_ui(false, |ui| {
                    Checkbox::without_text(&mut self.guarded[0])
                        .size(MechanismSize::Small)
                        .show(ui)
                });
            });
            ui.add_space(28.0);
            let _medium = ui.vertical(|ui| {
                let _label = ui.label(chrome::muted("MEDIUM · 3×3 · ON"));
                let _guard = ui.add_enabled_ui(false, |ui| {
                    Checkbox::without_text(&mut self.guarded[1])
                        .size(MechanismSize::Medium)
                        .show(ui)
                });
            });
            ui.add_space(28.0);
            let _large = ui.vertical(|ui| {
                let _label = ui.label(chrome::muted("LARGE · 4×4 · OFF"));
                let _guard = ui.add_enabled_ui(false, |ui| {
                    Checkbox::without_text(&mut self.guarded[2])
                        .size(MechanismSize::Large)
                        .show(ui)
                });
            });
        });
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct Day(i32, u32, u32);

impl GregorianDay for Day {
    fn ymd(self) -> (i32, u32, u32) {
        (self.0, self.1, self.2)
    }

    fn from_ymd(year: i32, month: u32, day: u32) -> Self {
        Self(year, month, day)
    }
}

pub struct Dates {
    values: [Day; 5],
    cassette_loaded: bool,
}

impl Default for Dates {
    fn default() -> Self {
        Self {
            values: [Day(2026, 7, 20); 5],
            cassette_loaded: false,
        }
    }
}

impl Dates {
    pub fn show(&mut self, ui: &mut egui::Ui, water: &mut Surface, id: impl Hash) {
        let _title = ui.label(chrome::title("DATE TRANSPORT"));
        let _law = ui.label(chrome::muted(
            "rigid reel banks · explicit width · optionality belongs to the application",
        ));
        ui.add_space(22.0);
        let spool = DateSpool::new(&mut self.values[0], 2005..=2027)
            .label("FULL DATE · EXPLICIT 360 pt")
            .width(360.0)
            .show(ui, (&id, "all"));
        water.date_spool(&spool);

        ui.add_space(18.0);
        let _banks = ui.horizontal_top(|ui| {
            let year = DateReels::YEAR;
            let spool = DateSpool::new(&mut self.values[1], 2005..=2027)
                .label("YEAR")
                .reels(year)
                .width(year.minimum_width())
                .show(ui, (&id, "year"));
            water.date_spool(&spool);

            let year_month = DateReels::YEAR_MONTH;
            let spool = DateSpool::new(&mut self.values[2], 2005..=2027)
                .label("YEAR · MONTH")
                .reels(year_month)
                .width(year_month.minimum_width())
                .show(ui, (&id, "year-month"));
            water.date_spool(&spool);

            let month_day = DateReels::MONTH_DAY;
            let spool = DateSpool::new(&mut self.values[3], 2005..=2027)
                .label("MONTH · DAY")
                .reels(month_day)
                .width(month_day.minimum_width())
                .show(ui, (&id, "month-day"));
            water.date_spool(&spool);
        });

        ui.add_space(18.0);
        let cassette = if self.cassette_loaded {
            "INSERTED"
        } else {
            "WITHDRAWN"
        };
        let _optional = ui.label(chrome::eyebrow(format!(
            "APPLICATION-OWNED OPTIONALITY · {cassette} CASSETTE"
        )));
        ui.add_space(5.0);
        let _assembly = ui.horizontal_top(|ui| {
            let present = Checkbox::new(&mut self.cassette_loaded, "CASSETTE PRESENT").show(ui);
            water.checkbox(&present);
            ui.add_space(24.0);

            let reels = DateReels::YEAR_MONTH;
            let caption = if self.cassette_loaded {
                "REINSERTED RETAINED VALUE"
            } else {
                "VALUE RETAINED WHILE UNLOADED"
            };
            let spool = DateSpool::new(&mut self.values[4], 2005..=2027)
                .label(caption)
                .reels(reels)
                .loaded(self.cassette_loaded)
                .width(reels.minimum_width())
                .show(ui, (&id, "optional"));
            water.date_spool(&spool);
        });
    }
}
