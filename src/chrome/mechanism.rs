//! Shared footprint and coupling laws for foundry-owned mechanisms.

use std::ops::Deref;

use egui::{Pos2, Rect, Response, Shape, Ui};

use super::{COUPLING_SPACING, foundry};

/// A build-time forged casing gauge shared by compatible mechanisms.
///
/// Each variant selects independently projected geometry rather than rescaling
/// another gauge at runtime. The named point count is the nominal casing
/// height. Unencumbered square mechanisms use it as both interaction
/// dimensions; half-width hardware, such as
/// [`super::DragHandle::friction_pad`], derives its width from the same gauge.
/// Protective or kinematic overhang remains part of a mechanism's allocation:
/// [`super::Checkbox`] therefore derives a larger stable guard envelope.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MechanismSize {
    /// Dense 20-point hardware for compact tool strips.
    Small,
    /// Compact 24-point hardware.
    Medium,
    /// Standard 32-point hardware and the compatibility-preserving default.
    #[default]
    Large,
}

impl MechanismSize {
    /// Every public gauge, in ascending order.
    pub const ALL: [Self; 3] = [Self::Small, Self::Medium, Self::Large];

    /// Visible casing height in logical points.
    ///
    /// This is also the interaction height and side of unencumbered square
    /// mechanisms. Protective envelopes may derive a larger allocation.
    pub const fn side(self) -> f32 {
        match self {
            Self::Small => foundry::law::MECHANISM_SIDE_SMALL as f32,
            Self::Medium => foundry::law::MECHANISM_SIDE_MEDIUM as f32,
            Self::Large => foundry::law::MECHANISM_SIDE_LARGE as f32,
        }
    }

    pub(super) const fn atlas_index(self) -> usize {
        match self {
            Self::Small => 0,
            Self::Medium => 1,
            Self::Large => 2,
        }
    }
}

/// Clear casing-to-casing span crossed by a pair of coupling ties.
///
/// The foundry fixes tie stock, attachment stations, projection, and lighting;
/// this value controls only their visible length and the corresponding egui
/// layout gap. Two points is the shortest span that leaves the twin ties
/// legible at ordinary display scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CouplingGap(f32);

impl CouplingGap {
    /// Shortest mechanically legible clear span.
    pub const MIN_POINTS: f32 = 2.0;
    /// Minimum two-point spacing for dense repeated mechanisms.
    pub const MINIMUM: Self = Self(Self::MIN_POINTS);
    /// Tight three-point spacing for dense tool strips.
    pub const TIGHT: Self = Self(3.0);
    /// Canonical six-point spacing used by ordinary foundry assemblies.
    pub const STANDARD: Self = Self(COUPLING_SPACING);

    /// Specify a clear span in logical points.
    ///
    /// # Panics
    ///
    /// Panics unless `points` is finite and at least
    /// [`CouplingGap::MIN_POINTS`].
    pub const fn new(points: f32) -> Self {
        assert!(
            points >= Self::MIN_POINTS && points < f32::INFINITY,
            "coupling gap must be finite and mechanically legible"
        );
        Self(points)
    }

    /// Return the clear span in logical points.
    pub const fn points(self) -> f32 {
        self.0
    }
}

impl Default for CouplingGap {
    fn default() -> Self {
        Self::STANDARD
    }
}

/// One opaque attachment edge carrying the foundry's two tie stations.
///
/// Applications do not construct ports. Crafted response types expose them
/// through [`CouplingTarget`], allowing [`Coupled`] to join actual casings
/// rather than arbitrary response rectangles.
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct CouplingPort {
    pub(crate) anchors: [Pos2; 2],
}

/// Left and right attachment edges of one crafted mechanism.
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct CouplingPorts {
    pub(crate) left: CouplingPort,
    pub(crate) right: CouplingPort,
}

impl CouplingPorts {
    pub(crate) fn around(rect: Rect) -> Self {
        Self::spanning(rect, rect)
    }

    pub(crate) fn around_with_station_height(rect: Rect, station_height: f32) -> Self {
        let station =
            Rect::from_center_size(rect.center(), egui::vec2(rect.width(), station_height));
        Self::spanning(station, station)
    }

    pub(crate) fn spanning(left: Rect, right: Rect) -> Self {
        let port = |x: f32, rect: Rect| {
            let dy = rect.height() * 0.26;
            CouplingPort {
                anchors: [
                    Pos2::new(x, rect.center().y - dy),
                    Pos2::new(x, rect.center().y + dy),
                ],
            }
        };
        Self {
            left: port(left.left(), left),
            right: port(right.right(), right),
        }
    }
}

pub(super) mod sealed {
    pub trait Sealed {}
}

/// A foundry response whose physical casing admits twin-tie coupling.
///
/// This trait is sealed: only mechanisms whose attachment geometry is known
/// to this crate can be coupled. Consumers compose those mechanisms through
/// [`Coupled::horizontal`].
pub trait CouplingTarget: sealed::Sealed {
    /// Return the mechanism's opaque physical attachment ports.
    #[doc(hidden)]
    fn coupling_ports(&self) -> CouplingPorts;
}

/// Horizontal twin-tie composition for two crafted mechanisms.
///
/// Compositions are themselves coupling targets, so nested pairs form a
/// mechanically aligned strip without introducing a second layout vocabulary.
///
/// # Example
///
/// ```
/// use brass_poolrooms::{
///     chrome::{Coupled, MechanismSize, Monoglyph, Symbol},
///     egui,
/// };
///
/// fn paired_actions(ui: &mut egui::Ui) -> (bool, bool) {
///     let pair = Coupled::horizontal(
///         ui,
///         |ui| Monoglyph::symbol(Symbol::Remove).size(MechanismSize::Small).show(ui),
///         |ui| Monoglyph::new('▣').size(MechanismSize::Small).show(ui),
///     );
///     (pair.left.clicked(), pair.right.clicked())
/// }
/// ```
pub struct Coupled;

impl Coupled {
    /// Lay out two mechanisms at the canonical foundry gap and join their
    /// adjacent casings with two illuminated bronze ties.
    ///
    /// The ties are inserted beneath both child mechanisms. Interaction is not
    /// merged: the returned [`CoupledResponse`] retains each original response.
    pub fn horizontal<L, R>(
        ui: &mut Ui,
        left: impl FnOnce(&mut Ui) -> L,
        right: impl FnOnce(&mut Ui) -> R,
    ) -> CoupledResponse<L, R>
    where
        L: CouplingTarget,
        R: CouplingTarget,
    {
        Self::horizontal_with_gap(ui, CouplingGap::STANDARD, left, right)
    }

    /// Lay out and twin-tie two mechanisms across an explicit clear span.
    ///
    /// This is the dense-layout counterpart to [`Coupled::horizontal`].
    /// `gap` controls both the casing separation and the visible length of the
    /// correctly projected bronze ties; it does not stretch either mechanism.
    pub fn horizontal_with_gap<L, R>(
        ui: &mut Ui,
        gap: CouplingGap,
        left: impl FnOnce(&mut Ui) -> L,
        right: impl FnOnce(&mut Ui) -> R,
    ) -> CoupledResponse<L, R>
    where
        L: CouplingTarget,
        R: CouplingTarget,
    {
        let painter = ui.painter().clone();
        let ties = painter.add(Shape::Noop);
        let row = ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = gap.points();
            (left(ui), right(ui))
        });
        let (left, right) = row.inner;
        let lp = left.coupling_ports();
        let rp = right.coupling_ports();
        painter.set(ties, foundry::tie_pair(lp.right, rp.left));
        CoupledResponse {
            ports: CouplingPorts {
                left: lp.left,
                right: rp.right,
            },
            response: row.response,
            left,
            right,
        }
    }
}

/// Independent child responses and aggregate layout response from [`Coupled`].
pub struct CoupledResponse<L, R> {
    /// Left mechanism response.
    pub left: L,
    /// Right mechanism response.
    pub right: R,
    response: Response,
    ports: CouplingPorts,
}

impl<L, R> CoupledResponse<L, R> {
    /// Consume the composition and recover both mechanism responses.
    pub fn into_parts(self) -> (L, R) {
        (self.left, self.right)
    }
}

impl<L, R> Deref for CoupledResponse<L, R> {
    type Target = Response;

    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

impl<L, R> sealed::Sealed for CoupledResponse<L, R>
where
    L: CouplingTarget,
    R: CouplingTarget,
{
}

impl<L, R> CouplingTarget for CoupledResponse<L, R>
where
    L: CouplingTarget,
    R: CouplingTarget,
{
    fn coupling_ports(&self) -> CouplingPorts {
        self.ports
    }
}
