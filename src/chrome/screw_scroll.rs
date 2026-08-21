//! A fixed-gauge vertical lead screw governing an egui scroll surface.
//!
//! The bronze indicator is the screw's translating nut. Its travel selects the
//! scroll offset and its displacement determines shaft rotation through one
//! single-start 30° lead law. Six-cove handwheels are welded to both ends.

#![deny(missing_docs)]

use std::sync::Arc;

use egui::{
    CursorIcon, IdSalt, Pos2, Rect, Sense, Vec2, Vec2b, WidgetInfo, WidgetType,
    scroll_area::{ScrollAreaOutput, ScrollBarVisibility, ScrollSource},
};

use super::{ForgedMesh, foundry, plunger};

const BAR_WIDTH: f32 = 14.0;
const INNER_MARGIN: f32 = 3.0;
const NUT_BEVEL: f32 = 1.0;
const NUT_RISE: f32 = 1.8;
const NUT_HEADROOM: f32 = 3.0;
const NUT_MIN_LENGTH: f32 = 18.0;

#[derive(Clone, Copy)]
struct BakedScrewPose {
    phase: f32,
    mesh: ForgedMesh,
}

#[derive(Clone, Copy)]
struct BakedCapPose {
    phase: f32,
    top: ForgedMesh,
    bottom: ForgedMesh,
}

#[derive(Clone, Copy)]
struct BakedScrewScroll {
    screws: &'static [BakedScrewPose],
    caps: &'static [BakedCapPose],
}

mod baked {
    use super::{BakedCapPose, BakedScrewPose, BakedScrewScroll};
    use crate::chrome::plunger::{BakedMesh, BakedVertex};

    include!(concat!(env!("OUT_DIR"), "/screw_scroll_atlas.rs"));
}

/// A narrow, permanently present vertical scrollbar embodied as a lead screw.
///
/// The mechanism reserves its own gutter and suppresses egui's stock paint.
/// Wheel, touch, and programmatic scrolling remain egui-owned; direct pointer
/// manipulation of the bronze nut writes the same persisted scroll state.
#[derive(Clone, Debug)]
#[must_use = "call show to present the scroll surface"]
pub struct ScrewScroll {
    id_salt: Option<IdSalt>,
    max_size: Vec2,
    min_scrolled_height: f32,
    auto_shrink: Vec2b,
    offset_y: Option<f32>,
    animated: bool,
}

impl Default for ScrewScroll {
    fn default() -> Self {
        Self::vertical()
    }
}

impl ScrewScroll {
    /// Construct a vertical lead-screw scroll surface.
    pub const fn vertical() -> Self {
        Self {
            id_salt: None,
            max_size: Vec2::INFINITY,
            min_scrolled_height: 64.0,
            auto_shrink: Vec2b::TRUE,
            offset_y: None,
            animated: true,
        }
    }

    /// Assign a stable identity within the enclosing UI.
    pub fn id_salt(mut self, salt: impl egui::AsIdSalt) -> Self {
        self.id_salt = Some(IdSalt::new(salt));
        self
    }

    /// Limit the complete scroll surface width, including its screw gutter.
    pub const fn max_width(mut self, width: f32) -> Self {
        self.max_size.x = width;
        self
    }

    /// Limit the complete scroll surface height.
    pub const fn max_height(mut self, height: f32) -> Self {
        self.max_size.y = height;
        self
    }

    /// Set the minimum height at which vertical scrolling remains viable.
    pub const fn min_scrolled_height(mut self, height: f32) -> Self {
        self.min_scrolled_height = height;
        self
    }

    /// Control whether either axis shrinks around small content.
    pub fn auto_shrink(mut self, shrink: impl Into<Vec2b>) -> Self {
        self.auto_shrink = shrink.into();
        self
    }

    /// Force the vertical content offset for this pass.
    pub const fn vertical_scroll_offset(mut self, offset: f32) -> Self {
        self.offset_y = Some(offset);
        self
    }

    /// Enable or disable animation of programmatic scroll targets.
    pub const fn animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }

    /// Present content and its physically coupled scroll mechanism.
    pub fn show<R>(
        self,
        ui: &mut egui::Ui,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> ScrollAreaOutput<R> {
        let inherited_scroll = ui.spacing().scroll;
        self.present(ui, |area, ui| {
            area.show(ui, |content| {
                content.spacing_mut().scroll = inherited_scroll;
                add_contents(content)
            })
        })
    }

    /// Present a virtualized fixed-height sequence and its scroll mechanism.
    pub fn show_rows<R>(
        self,
        ui: &mut egui::Ui,
        row_height_sans_spacing: f32,
        total_rows: usize,
        add_contents: impl FnOnce(&mut egui::Ui, std::ops::Range<usize>) -> R,
    ) -> ScrollAreaOutput<R> {
        let inherited_scroll = ui.spacing().scroll;
        self.present(ui, |area, ui| {
            area.show_rows(ui, row_height_sans_spacing, total_rows, |content, rows| {
                content.spacing_mut().scroll = inherited_scroll;
                add_contents(content, rows)
            })
        })
    }

    fn present<R>(
        self,
        ui: &mut egui::Ui,
        show_area: impl FnOnce(egui::ScrollArea, &mut egui::Ui) -> ScrollAreaOutput<R>,
    ) -> ScrollAreaOutput<R> {
        let allocated = BAR_WIDTH + INNER_MARGIN;
        let content_width = (self.max_size.x - allocated).max(1.0);
        let mut area = egui::ScrollArea::vertical()
            .scroll_bar_visibility(ScrollBarVisibility::AlwaysVisible)
            .scroll_source(ScrollSource {
                scroll_bar: false,
                ..ScrollSource::default()
            })
            .max_width(content_width)
            .max_height(self.max_size.y)
            .min_scrolled_height(self.min_scrolled_height)
            .auto_shrink(self.auto_shrink)
            .animated(self.animated);
        if let Some(salt) = self.id_salt {
            area = area.id_salt(salt);
        }
        if let Some(offset) = self.offset_y {
            area = area.vertical_scroll_offset(offset.max(0.0));
        }

        let scoped = ui.scope(|ui| {
            let mut scroll = egui::style::ScrollStyle::solid();
            scroll.bar_width = BAR_WIDTH;
            scroll.bar_inner_margin = INNER_MARGIN;
            scroll.bar_outer_margin = 0.0;
            scroll.handle_min_length = NUT_MIN_LENGTH;
            ui.spacing_mut().scroll = scroll;
            let mut output = show_area(area, ui);
            embody(ui, &mut output);
            output
        });
        scoped.inner
    }
}

#[derive(Clone, Copy)]
struct Mechanics {
    bar: Rect,
    lead: Rect,
    nut: Rect,
    phase: f32,
    max_offset: f32,
}

fn embody<R>(ui: &mut egui::Ui, output: &mut ScrollAreaOutput<R>) {
    let bar = Rect::from_min_max(
        Pos2::new(
            output.inner_rect.right() + INNER_MARGIN,
            output.inner_rect.top(),
        ),
        Pos2::new(
            output.inner_rect.right() + INNER_MARGIN + BAR_WIDTH,
            output.inner_rect.bottom(),
        ),
    );
    let mut mechanism = mechanics(output, bar);
    let sense = if ui.is_enabled() {
        Sense::CLICK | Sense::DRAG
    } else {
        Sense::hover()
    };
    let mut response = ui.interact(bar, output.id.with("lead-screw"), sense);
    response.widget_info(|| WidgetInfo::new(WidgetType::ScrollBar));
    if response.enabled() {
        response = response.on_hover_cursor(CursorIcon::ResizeVertical);
    }
    if drive(ui, &response, output, mechanism) {
        mechanism = mechanics(output, bar);
    }
    paint(ui, response.id, mechanism);
}

fn mechanics<R>(output: &ScrollAreaOutput<R>, bar: Rect) -> Mechanics {
    let cap = baked::CAP_HEIGHT;
    let lead = Rect::from_min_max(
        Pos2::new(bar.left(), bar.top() + cap + NUT_HEADROOM),
        Pos2::new(bar.right(), bar.bottom() - cap - NUT_HEADROOM),
    );
    debug_assert!(
        lead.height() >= NUT_MIN_LENGTH,
        "screw scroll is shorter than its hardware"
    );
    let viewport = output.inner_rect.height();
    let content = output.content_size.y.max(viewport);
    let max_offset = (content - viewport).max(0.0);
    let nut_length = (lead.height() * viewport / content)
        .clamp(NUT_MIN_LENGTH.min(lead.height()), lead.height());
    let travel = (lead.height() - nut_length).max(0.0);
    let advance = if max_offset > 0.0 {
        travel * (output.state.offset.y / max_offset).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let nut = Rect::from_center_size(
        Pos2::new(bar.center().x, lead.top() + advance + nut_length * 0.5),
        Vec2::new(baked::CAP_WIDTH, nut_length),
    );
    Mechanics {
        bar,
        lead,
        nut,
        phase: (-advance / baked::LEAD * std::f32::consts::TAU).rem_euclid(std::f32::consts::TAU),
        max_offset,
    }
}

#[derive(Clone, Copy, Default)]
struct DragAnchor(f32);

fn drive<R>(
    ui: &egui::Ui,
    response: &egui::Response,
    output: &mut ScrollAreaOutput<R>,
    mechanics: Mechanics,
) -> bool {
    if mechanics.max_offset <= 0.0 {
        return false;
    }
    let anchor_id = response.id.with("nut-grab");
    if response.drag_started_by(egui::PointerButton::Primary)
        && let Some(pointer) = response.interact_pointer_pos()
    {
        let grab = if mechanics.nut.contains(pointer) {
            pointer.y - mechanics.nut.top()
        } else {
            mechanics.nut.height() * 0.5
        };
        ui.ctx().data_mut(|data| {
            let _old = data.insert_temp(anchor_id, DragAnchor(grab));
        });
    }

    let pointer = response.interact_pointer_pos();
    let dragged = response.dragged_by(egui::PointerButton::Primary);
    let track_clicked = response.clicked_by(egui::PointerButton::Primary)
        && pointer.is_some_and(|point| !mechanics.nut.contains(point));
    let changed = if (dragged || track_clicked)
        && let Some(pointer) = pointer
    {
        let grab = if track_clicked {
            mechanics.nut.height() * 0.5
        } else {
            ui.ctx()
                .data(|data| data.get_temp::<DragAnchor>(anchor_id))
                .unwrap_or(DragAnchor(mechanics.nut.height() * 0.5))
                .0
        };
        let travel = (mechanics.lead.height() - mechanics.nut.height()).max(0.0);
        let advance = (pointer.y - grab - mechanics.lead.top()).clamp(0.0, travel);
        let offset = if travel > 0.0 {
            mechanics.max_offset * advance / travel
        } else {
            0.0
        };
        let changed = (offset - output.state.offset.y).abs() > 0.01;
        if changed {
            output.state.offset.y = offset;
            output.state.store(ui.ctx(), output.id);
            ui.ctx().request_discard("Lead-screw nut moved");
        }
        changed
    } else {
        false
    };

    if response.drag_stopped_by(egui::PointerButton::Primary) {
        ui.ctx().data_mut(|data| {
            let _anchor = data.remove_temp::<DragAnchor>(anchor_id);
        });
    }
    changed
}

fn paint(ui: &egui::Ui, id: egui::Id, mechanics: Mechanics) {
    let painter = ui.painter().with_clip_rect(mechanics.bar);
    foundry::socket_bed(&painter, mechanics.bar);
    let hardware = ui.ctx().data_mut(|data| {
        data.get_temp_mut_or_default::<HardwareCache>(id.with("compiled-foundry"))
            .prepare(mechanics)
    });
    foundry::paint_compiled(&painter, mechanics.bar, &hardware);

    let face = mechanics.nut.size() - Vec2::splat(2.0 * NUT_BEVEL);
    let _crown =
        foundry::raised_scroll_nut(&painter, mechanics.nut.center(), face, NUT_RISE, NUT_BEVEL);
    foundry::socket_rim(&painter, mechanics.bar);
}

#[derive(Clone, Copy, PartialEq)]
struct HardwareKey {
    bar: Rect,
    lead: Rect,
    screw_pose: usize,
    cap_pose: usize,
}

#[derive(Clone, Default)]
struct HardwareCache(Option<(HardwareKey, Arc<egui::Mesh>)>);

impl HardwareCache {
    fn prepare(&mut self, mechanics: Mechanics) -> Arc<egui::Mesh> {
        let screw_pose = plunger::pose_index(
            mechanics.phase,
            0.0,
            std::f32::consts::TAU,
            baked::ATLAS.screws.len(),
        );
        let cap_period = std::f32::consts::TAU / 6.0;
        let cap_phase = mechanics.phase.rem_euclid(cap_period);
        let cap_pose = plunger::pose_index(cap_phase, 0.0, cap_period, baked::ATLAS.caps.len());
        let key = HardwareKey {
            bar: mechanics.bar,
            lead: mechanics.lead,
            screw_pose,
            cap_pose,
        };
        if let Some((cached_key, mesh)) = &self.0
            && *cached_key == key
        {
            return Arc::clone(mesh);
        }

        let mut mesh = egui::Mesh::default();
        let screw = baked::ATLAS.screws[screw_pose];
        let screw_step = std::f32::consts::TAU / (baked::ATLAS.screws.len() - 1) as f32;
        debug_assert!(
            circular_distance(screw.phase, mechanics.phase) <= screw_step,
            "forged screw phase escaped its atlas"
        );
        let mut y = mechanics.lead.top() - baked::LEAD;
        while y < mechanics.lead.bottom() + baked::LEAD {
            screw
                .mesh
                .stamp(&mut mesh, Pos2::new(mechanics.bar.center().x, y));
            y += baked::LEAD;
        }
        let caps = baked::ATLAS.caps[cap_pose];
        let cap_step = cap_period / (baked::ATLAS.caps.len() - 1) as f32;
        debug_assert!(
            circular_distance(caps.phase, cap_phase) <= cap_step,
            "forged cap phase escaped its atlas"
        );
        caps.top.stamp(
            &mut mesh,
            Pos2::new(
                mechanics.bar.center().x,
                mechanics.bar.top() + baked::CAP_HEIGHT * 0.5,
            ),
        );
        caps.bottom.stamp(
            &mut mesh,
            Pos2::new(
                mechanics.bar.center().x,
                mechanics.bar.bottom() - baked::CAP_HEIGHT * 0.5,
            ),
        );
        let mesh = Arc::new(mesh);
        self.0 = Some((key, Arc::clone(&mesh)));
        mesh
    }
}

fn circular_distance(left: f32, right: f32) -> f32 {
    let turn = std::f32::consts::TAU;
    let difference = (left - right).rem_euclid(turn);
    difference.min(turn - difference)
}
