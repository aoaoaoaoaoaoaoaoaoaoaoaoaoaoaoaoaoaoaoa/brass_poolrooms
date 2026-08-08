//! Map-anchored bronze pins forged as one visual and interaction mechanism.

#![deny(missing_docs)]

use std::f32::consts::TAU;

use egui::{Align2, Color32, FontId, Mesh, Painter, Pos2, Rect, Stroke, Vec2};

use super::{MechanismSize, TEXT, foundry};

#[derive(Clone, Copy)]
struct Gauge {
    rise: f32,
    grip_half: f32,
    grip_above_bulb: f32,
    grip_tail: f32,
    bulb_radius: f32,
    shoulder_half: f32,
    shoulder_drop: f32,
    shadow_offset: f32,
    rim_width: f32,
    inscription_size: f32,
}

const SMALL: Gauge = Gauge {
    rise: 18.0,
    grip_half: 8.0,
    grip_above_bulb: 8.0,
    grip_tail: -10.0,
    bulb_radius: 4.6,
    shoulder_half: 2.5,
    shoulder_drop: 2.2,
    shadow_offset: 0.8,
    rim_width: 0.75,
    inscription_size: 0.0,
};

const MEDIUM: Gauge = Gauge {
    rise: 25.0,
    grip_half: 14.0,
    grip_above_bulb: 14.0,
    grip_tail: 4.0,
    bulb_radius: 8.0,
    shoulder_half: 4.2,
    shoulder_drop: 3.7,
    shadow_offset: 1.1,
    rim_width: 1.1,
    inscription_size: 12.0,
};

const LARGE_SCALE: f32 = MechanismSize::Large.side() / MechanismSize::Medium.side();
const LARGE: Gauge = Gauge {
    rise: MEDIUM.rise * LARGE_SCALE,
    grip_half: MEDIUM.grip_half * LARGE_SCALE,
    grip_above_bulb: MEDIUM.grip_above_bulb * LARGE_SCALE,
    grip_tail: MEDIUM.grip_tail * LARGE_SCALE,
    bulb_radius: MEDIUM.bulb_radius * LARGE_SCALE,
    shoulder_half: MEDIUM.shoulder_half * LARGE_SCALE,
    shoulder_drop: MEDIUM.shoulder_drop * LARGE_SCALE,
    shadow_offset: MEDIUM.shadow_offset * LARGE_SCALE,
    rim_width: MEDIUM.rim_width * LARGE_SCALE,
    inscription_size: MEDIUM.inscription_size * LARGE_SCALE,
};

const fn gauge(size: MechanismSize) -> Gauge {
    match size {
        MechanismSize::Small => SMALL,
        MechanismSize::Medium => MEDIUM,
        MechanismSize::Large => LARGE,
    }
}

#[derive(Clone, Debug)]
struct Inscription {
    text: String,
    points: Option<f32>,
}

/// A bronze pin whose point fixes an exact coordinate and whose bulb is its grip.
///
/// [`ForgePin::size`] selects a complete discrete die: shaft, bulb, hit region,
/// highlight, and optional inscription remain one object and therefore cannot
/// silently acquire different gauges. The established compact and numbered
/// pins are the exact small and medium dies. The large die extends the medium
/// proportions without rescaling a rendered image.
///
/// Medium and large bulbs admit a centered monospace inscription. Small is a
/// deliberately unlettered precision pin.
///
/// # Example
///
/// ```
/// use dwemer_poolrooms::{chrome::{ForgePin, MechanismSize}, egui};
///
/// fn support(painter: &egui::Painter, anchor: egui::Pos2, seized: bool) {
///     ForgePin::new(anchor)
///         .size(MechanismSize::Medium)
///         .inscription("3")
///         .paint(painter, seized);
/// }
/// ```
#[derive(Clone, Debug)]
pub struct ForgePin {
    anchor: Pos2,
    size: MechanismSize,
    inscription: Option<Inscription>,
}

impl ForgePin {
    /// Forge an unlettered large pin at an exact map or canvas coordinate.
    pub const fn new(anchor: Pos2) -> Self {
        Self {
            anchor,
            size: MechanismSize::Large,
            inscription: None,
        }
    }

    /// Select the pin die.
    ///
    /// # Panics
    ///
    /// Panics when selecting [`MechanismSize::Small`] after adding an
    /// inscription, because the small bulb cannot admit text.
    pub fn size(mut self, size: MechanismSize) -> Self {
        assert!(
            size != MechanismSize::Small || self.inscription.is_none(),
            "the small forge pin does not admit an inscription"
        );
        self.size = size;
        self
    }

    /// Cut centered monospace text into a medium or large bulb.
    ///
    /// The default point size is 12 for medium and 16 for large. Use
    /// [`Self::inscription_size`] for a sigil that needs a distinct optical
    /// size, such as a deletion cross.
    ///
    /// # Panics
    ///
    /// Panics for a small pin, whose bulb is deliberately unlettered.
    pub fn inscription(mut self, text: impl Into<String>) -> Self {
        assert!(
            self.size != MechanismSize::Small,
            "the small forge pin does not admit an inscription"
        );
        self.inscription = Some(Inscription {
            text: text.into(),
            points: None,
        });
        self
    }

    /// Override the optical point size of an existing inscription.
    ///
    /// # Panics
    ///
    /// Panics unless an inscription has already been supplied and `points` is
    /// finite and positive.
    pub fn inscription_size(mut self, points: f32) -> Self {
        assert!(
            points > 0.0 && points < f32::INFINITY,
            "forge-pin inscription size must be finite and positive"
        );
        assert!(
            self.inscription.is_some(),
            "an inscription must precede its optical size"
        );
        if let Some(inscription) = &mut self.inscription {
            inscription.points = Some(points);
        }
        self
    }

    /// Return the pin's exact map or canvas coordinate.
    pub const fn anchor(&self) -> Pos2 {
        self.anchor
    }

    /// Return the center of the raised bulb.
    pub fn bulb(&self) -> Pos2 {
        self.anchor - Vec2::new(0.0, gauge(self.size).rise)
    }

    /// Return the complete draggable hit region belonging to this die.
    pub fn grip(&self) -> Rect {
        let gauge = gauge(self.size);
        Rect::from_min_max(
            self.bulb() - Vec2::new(gauge.grip_half, gauge.grip_above_bulb),
            self.anchor + Vec2::new(gauge.grip_half, gauge.grip_tail),
        )
    }

    /// Paint the pin above its coordinate, heating the bronze when `seized`.
    pub fn paint(&self, painter: &Painter, seized: bool) {
        let gauge = gauge(self.size);
        let bulb = self.bulb();
        let heat = if seized { 0.07 } else { 0.0 };
        let left = bulb + Vec2::new(-gauge.shoulder_half, gauge.shoulder_drop);
        let right = bulb + Vec2::new(gauge.shoulder_half, gauge.shoulder_drop);
        let shadow = vec![
            left + Vec2::splat(gauge.shadow_offset),
            right + Vec2::splat(gauge.shadow_offset),
            self.anchor + Vec2::new(0.0, 1.0),
        ];
        let _shadow = painter.add(egui::Shape::convex_polygon(
            shadow,
            Color32::from_black_alpha(96),
            Stroke::NONE,
        ));
        stamp(
            painter,
            vec![left, right, self.anchor],
            &[[right, self.anchor]],
            &[[self.anchor, left]],
            heat,
        );
        sphere(painter, bulb, gauge.bulb_radius, heat);
        let _rim = painter.circle_stroke(
            bulb,
            gauge.bulb_radius,
            Stroke::new(gauge.rim_width, foundry::bronze(0.16)),
        );
        if let Some(inscription) = &self.inscription {
            let _inscription = painter.text(
                bulb,
                Align2::CENTER_CENTER,
                &inscription.text,
                FontId::monospace(inscription.points.unwrap_or(gauge.inscription_size)),
                TEXT,
            );
        }
    }
}

fn stamp(
    painter: &Painter,
    silhouette: Vec<Pos2>,
    crowns: &[[Pos2; 2]],
    soles: &[[Pos2; 2]],
    heat: f32,
) {
    let _body = painter.add(egui::Shape::convex_polygon(
        silhouette,
        foundry::bronze(0.56 + heat),
        Stroke::NONE,
    ));
    for edge in crowns {
        let _crown =
            painter.line_segment(*edge, Stroke::new(0.8_f32, foundry::bronze(0.80 + heat)));
    }
    for edge in soles {
        let _sole = painter.line_segment(*edge, Stroke::new(0.8_f32, foundry::bronze(0.18 + heat)));
    }
}

fn sphere(painter: &Painter, center: Pos2, radius: f32, heat: f32) {
    const RINGS: u32 = 4;
    const SECTORS: u32 = 24;

    let mut mesh = Mesh::default();
    mesh.reserve_vertices((1 + RINGS * SECTORS) as usize);
    mesh.reserve_triangles((SECTORS * (2 * RINGS - 1)) as usize);
    mesh.colored_vertex(center, sphere_bronze(0.0, 0.0, heat));
    for ring in 1..=RINGS {
        let radius_fraction = ring as f32 / RINGS as f32;
        for sector in 0..SECTORS {
            let angle = TAU * sector as f32 / SECTORS as f32;
            let x = radius_fraction * angle.cos();
            let y = radius_fraction * angle.sin();
            mesh.colored_vertex(
                center + Vec2::new(x, y) * radius,
                sphere_bronze(
                    y,
                    radius_fraction.mul_add(-radius_fraction, 1.0).sqrt(),
                    heat,
                ),
            );
        }
    }
    for sector in 0..SECTORS {
        mesh.add_triangle(0, 1 + sector, 1 + (sector + 1) % SECTORS);
    }
    for ring in 2..=RINGS {
        let inner = 1 + (ring - 2) * SECTORS;
        let outer = inner + SECTORS;
        for sector in 0..SECTORS {
            let next = (sector + 1) % SECTORS;
            mesh.add_triangle(inner + sector, outer + sector, inner + next);
            mesh.add_triangle(inner + next, outer + sector, outer + next);
        }
    }
    let _bulb = painter.add(egui::Shape::mesh(mesh));
}

fn sphere_bronze(ny: f32, nz: f32, heat: f32) -> Color32 {
    let diffuse = nz
        .mul_add(foundry::law::LIGHT_Z, ny * foundry::law::LIGHT_Y)
        .max(0.0);
    let specular = nz
        .mul_add(foundry::law::HALF_Z, ny * foundry::law::HALF_Y)
        .max(0.0)
        .powi(14);
    foundry::bronze(0.38_f32.mul_add(specular, 0.42_f32.mul_add(diffuse, 0.12 + heat)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_and_medium_are_the_two_established_reference_dies() {
        let anchor = Pos2::new(30.0, 60.0);
        let small = ForgePin::new(anchor).size(MechanismSize::Small);
        assert_eq!(small.bulb(), Pos2::new(30.0, 42.0));
        assert_eq!(
            small.grip(),
            Rect::from_min_max(Pos2::new(22.0, 34.0), Pos2::new(38.0, 50.0))
        );

        let medium = ForgePin::new(anchor).size(MechanismSize::Medium);
        assert_eq!(medium.bulb(), Pos2::new(30.0, 35.0));
        assert_eq!(
            medium.grip(),
            Rect::from_min_max(Pos2::new(16.0, 21.0), Pos2::new(44.0, 64.0))
        );
    }

    #[test]
    fn large_extends_every_medium_dimension() {
        let anchor = Pos2::new(50.0, 70.0);
        let medium = ForgePin::new(anchor).size(MechanismSize::Medium);
        let large = ForgePin::new(anchor).size(MechanismSize::Large);
        assert!(large.bulb().y < medium.bulb().y);
        assert!(large.grip().width() > medium.grip().width());
        assert!(large.grip().height() > medium.grip().height());
    }

    #[test]
    #[should_panic(expected = "small forge pin does not admit an inscription")]
    fn small_rejects_inscription() {
        let _illegal = ForgePin::new(Pos2::ZERO)
            .size(MechanismSize::Small)
            .inscription("1");
    }

    #[test]
    fn bronze_charge_comes_from_the_common_foundry() {
        assert_eq!(foundry::bronze(0.0), Color32::from_rgb(34, 28, 19));
        assert_eq!(foundry::bronze(0.6), Color32::from_rgb(104, 86, 58));
        assert_eq!(foundry::bronze(1.0), Color32::from_rgb(196, 170, 124));
    }

    #[test]
    fn medium_die_preserves_the_trailgen_pixel_charge() {
        let reference_bronze = |tone: f32| {
            let tone = tone.clamp(0.0, 1.0);
            let (lo, hi, t) = if tone < 0.6 {
                (
                    foundry::law::BRONZE_SHADOW,
                    foundry::law::BRONZE_BODY,
                    tone / 0.6,
                )
            } else {
                (
                    foundry::law::BRONZE_BODY,
                    foundry::law::BRONZE_GLINT,
                    (tone - 0.6) / 0.4,
                )
            };
            let channel = |i: usize| (hi[i] - lo[i]).mul_add(t, lo[i]).round() as u8;
            Color32::from_rgb(channel(0), channel(1), channel(2))
        };
        for tone in [0.18, 0.25, 0.56, 0.63, 0.80, 0.87] {
            assert_eq!(foundry::bronze(tone), reference_bronze(tone));
        }
        for ring in 0..=4_u32 {
            let radius = ring as f32 / 4.0;
            for sector in 0..24_u32 {
                let angle = TAU * sector as f32 / 24.0;
                let ny = radius * angle.sin();
                let nz = radius.mul_add(-radius, 1.0).sqrt();
                let diffuse = nz
                    .mul_add(foundry::law::LIGHT_Z, ny * foundry::law::LIGHT_Y)
                    .max(0.0);
                let specular = nz
                    .mul_add(foundry::law::HALF_Z, ny * foundry::law::HALF_Y)
                    .max(0.0)
                    .powi(14);
                let tone = 0.38_f32.mul_add(specular, 0.42_f32.mul_add(diffuse, 0.12));
                assert_eq!(sphere_bronze(ny, nz, 0.0), reference_bronze(tone));
            }
        }
    }
}
