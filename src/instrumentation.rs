//! Optional semantic anchors for deterministic external UI choreography.
//!
//! Poolrooms names each anchored mechanism and leaves its wire spelling to
//! the driver: an owner-first dotted semantic path and, where one kind of
//! mechanism repeats, its raw runtime identity, never escaped here.

const STORE: &str = "brass-poolroom-anchors";

/// One named rectangle recorded during the current egui pass.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Anchor {
    /// Owner-first dotted semantic path, such as `poolrooms.section.header`.
    pub path: String,
    /// Raw runtime identity separating repeated mechanisms, such as a
    /// [`Section`](crate::chrome::Section) title.
    pub identity: Option<String>,
    /// Rectangle in logical points.
    pub rect: egui::Rect,
}

/// Record one anchor for the current pass.
pub fn record(ui: &egui::Ui, path: impl Into<String>, identity: Option<String>, rect: egui::Rect) {
    ui.ctx().data_mut(|data| {
        data.get_temp_mut_or_default::<Vec<Anchor>>(egui::Id::new(STORE))
            .push(Anchor {
                path: path.into(),
                identity,
                rect,
            });
    });
}

/// Drain the anchors recorded since the last drain.
pub fn take(ctx: &egui::Context) -> Vec<Anchor> {
    ctx.data_mut(|data| data.remove_temp::<Vec<Anchor>>(egui::Id::new(STORE)))
        .unwrap_or_default()
}
