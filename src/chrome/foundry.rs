//! The common Poolrooms foundry: one illuminant, one bronze charge, one set of
//! dies. Screen coordinates lie in the x-y plane with +y downward; the viewer
//! lies on +z. The distant key is confined to the y-z plane at 60° above the
//! top-of-screen horizon, so L=(0, -½, √3/2). Every authored metal part is cut
//! from these responses rather than carrying a private imitation.

use std::{f32::consts::FRAC_1_SQRT_2, sync::Arc};

use egui::{Color32, Galley, Pos2, Rect, Shape, Stroke, Vec2};

use super::mechanism::CouplingPort;
pub(super) mod law;

use law::*;

pub(crate) const ABYSS: Color32 = Color32::from_rgb(3, 3, 4);
pub(crate) const CONTROL_STOCK_DIAMETER: f32 = 14.0;
pub(crate) const RIM_RADIUS: f32 = 1.0;
pub(crate) const RIM_WIDTH: f32 = 1.0;

const STAMP_GAUGE: f32 = 1.0;
const PLAQUE_RISE: f32 = 2.0;
const PLAQUE_BEVEL_RUN: f32 = PLAQUE_RISE;
const PLAQUE_TEXT_PAD_X: f32 = 6.0;
const PLAQUE_ETCH_DEPTH: f32 = 0.72;
const PLAQUE_ETCH_BEVEL_RUN: f32 = 0.20;
/// Radial allowance cut into the soot-black floor beyond the nominal glyph.
/// It preserves a black core after antialiasing without widening the exposed
/// bronze wall.
const PLAQUE_ETCH_FLOOR_SHOULDER: f32 = 0.16;
const _: () = assert!(PLAQUE_ETCH_DEPTH / PLAQUE_ETCH_BEVEL_RUN > 3.0);
const COUPLING_TIE_DIAMETER: f32 = 1.55;
const COUPLING_TIE_BURIAL: f32 = 0.7;

#[derive(Clone, Copy)]
pub(crate) enum StockAxis {
    ScreenX,
    ScreenY,
}

/// Diffuse and Blinn-Phong response from the y and z components of a unit
/// normal. The omitted x component is immaterial because the illuminant and
/// half-vector both lie in the y-z plane.
pub(crate) fn yz_lumen(ny: f32, nz: f32, shine: f32) -> (f32, f32) {
    let diffuse = (ny * LIGHT_Y + nz * LIGHT_Z).max(0.0);
    let specular = (ny * HALF_Y + nz * HALF_Z).max(0.0).powf(shine);
    (diffuse, specular)
}

/// The foundry's oxidized-bronze ramp. `tone` is illumination, not a new
/// material choice: shadow, body, and polished glint are fixed alloy swatches.
pub(crate) fn bronze(tone: f32) -> Color32 {
    let [r, g, b] = bronze_rgb(tone);
    Color32::from_rgb(r, g, b)
}

/// Bronze cut on a lathe, evaluated under the foundry illuminant. `ny` and
/// `nz` are the visible surface normal's components in the common universe.
pub(crate) fn turned_bronze(ny: f32, nz: f32) -> Color32 {
    let (diffuse, specular) = yz_lumen(ny, nz, METAL_SHINE);
    bronze(0.16 + 0.5 * diffuse + 0.8 * specular)
}

/// Freshly exposed tool-cut bronze under the foundry illuminant.
///
/// The stronger diffuse and specular charge is an exposure compression for
/// subpixel cut walls: it preserves their alloy hue and directional light while
/// keeping a narrow incision legible against darkened stock.
fn fresh_cut_bronze(ny: f32, nz: f32) -> Color32 {
    let (diffuse, specular) = yz_lumen(ny, nz, METAL_SHINE);
    bronze(0.24 + 0.68 * diffuse + specular)
}

/// Perspective magnification at a faceplate-normal elevation in the common
/// fixed camera. Dynamic glyph masks use this same projection as baked solids.
pub(crate) fn perspective_scale(z: f32) -> f32 {
    EYE_Z / (EYE_Z - z).max(1.0)
}

fn project([x, y, z]: [f32; 3]) -> Vec2 {
    Vec2::new(x, y) * perspective_scale(z)
}

pub(crate) fn darkened_bronze(position: [f32; 3], normal: [f32; 3]) -> Color32 {
    let [r, g, b] = darkened_bronze_rgb(darkened_metal_tone(position, normal));
    Color32::from_rgb(r, g, b)
}

/// Engrave a dynamic plaque glyph with a steep, flat-bottomed cutter.
///
/// The exposed key-facing wall advances only by the cutter's narrow bevel run;
/// the soot-black floor then occludes its center. Unlike a broad V-bit, this
/// preserves a decisive black letterfield at small UI gauges.
fn plaque_engraving(
    painter: &egui::Painter,
    clip: Rect,
    pos: Pos2,
    galley: Arc<Galley>,
    surface_z: f32,
    depth: f32,
) {
    let incision = painter.with_clip_rect(clip);
    let wall_run = PLAQUE_ETCH_BEVEL_RUN * perspective_scale(surface_z);
    let normalizer = depth.hypot(PLAQUE_ETCH_BEVEL_RUN);
    incision.galley_with_override_text_color(
        pos + Vec2::new(0.0, wall_run),
        galley.clone(),
        fresh_cut_bronze(-depth / normalizer, PLAQUE_ETCH_BEVEL_RUN / normalizer),
    );
    for shoulder in [
        Vec2::new(-PLAQUE_ETCH_FLOOR_SHOULDER, 0.0),
        Vec2::new(0.0, -PLAQUE_ETCH_FLOOR_SHOULDER),
        Vec2::ZERO,
        Vec2::new(0.0, PLAQUE_ETCH_FLOOR_SHOULDER),
        Vec2::new(PLAQUE_ETCH_FLOOR_SHOULDER, 0.0),
    ] {
        incision.galley_with_override_text_color(pos + shoulder, galley.clone(), Color32::BLACK);
    }
}

/// A freshly exposed V-cut whose key-facing wall is the dominant glyph face.
///
/// Identification plaques use a steeper, soot-black cut through
/// [`plaque_engraving`].
/// Action glyphs reverse the visible-wall ordering: the same recessed floor
/// remains along the up-screen lip, while the illuminated bronze wall occupies
/// most of the cut. Both passes still derive from the groove depth and fixed
/// light rather than an ornamental text outline.
pub(crate) fn bright_cut_etch(
    painter: &egui::Painter,
    clip: Rect,
    pos: Pos2,
    galley: Arc<Galley>,
    surface_z: f32,
    depth: f32,
) {
    let incision = painter.with_clip_rect(clip);
    let light_fall = depth * (-LIGHT_Y / LIGHT_Z) * perspective_scale(surface_z);
    incision.galley_with_override_text_color(
        pos - Vec2::new(0.0, light_fall * 0.20),
        galley.clone(),
        bronze(0.07),
    );
    incision.galley_with_override_text_color(
        pos + Vec2::new(0.0, light_fall),
        galley,
        fresh_cut_bronze(-FRAC_1_SQRT_2, FRAC_1_SQRT_2),
    );
}

/// A casing-height bronze identification plate with a dynamically etched face.
/// Its 2 pt rise and 2 pt run form literal 45° edge facets; runtime varies only
/// the extrusion's width and its text mask.
pub(crate) struct Plaque {
    galley: Arc<Galley>,
    face_size: Vec2,
    footprint: Vec2,
}

impl Plaque {
    pub(crate) fn new(galley: Arc<Galley>, height: f32) -> Self {
        assert!(
            height > 2.0 * PLAQUE_BEVEL_RUN,
            "a plaque must admit its two bevels"
        );
        let face_h = height - 2.0 * PLAQUE_BEVEL_RUN;
        let floor_scale = perspective_scale(PLAQUE_RISE - PLAQUE_ETCH_DEPTH);
        let face_size = Vec2::new(
            galley.size().x / floor_scale + 2.0 * PLAQUE_TEXT_PAD_X,
            face_h,
        );
        let base_size = face_size + Vec2::splat(2.0 * PLAQUE_BEVEL_RUN);
        let crown_size = face_size * perspective_scale(PLAQUE_RISE);
        Self {
            galley,
            face_size,
            footprint: Vec2::new(base_size.x.max(crown_size.x), height),
        }
    }

    pub(crate) const fn size(&self) -> Vec2 {
        self.footprint
    }

    pub(crate) fn paint(&self, painter: &egui::Painter, center: Pos2) {
        let base_size = self.face_size + Vec2::splat(2.0 * PLAQUE_BEVEL_RUN);
        let base = Rect::from_center_size(center, base_size);
        let crown = Rect::from_center_size(center, self.face_size * perspective_scale(PLAQUE_RISE));
        let shadow = Rect::from_center_size(
            center + Vec2::new(0.0, -LIGHT_Y / LIGHT_Z * PLAQUE_RISE),
            self.face_size,
        );
        let _shadow = painter.rect_filled(shadow, 0.0, Color32::from_black_alpha(64));

        let [btl, btr, bbr, bbl] = [
            [-base_size.x * 0.5, -base_size.y * 0.5, 0.0],
            [base_size.x * 0.5, -base_size.y * 0.5, 0.0],
            [base_size.x * 0.5, base_size.y * 0.5, 0.0],
            [-base_size.x * 0.5, base_size.y * 0.5, 0.0],
        ];
        let [ctl, ctr, cbr, cbl] = [
            [
                -self.face_size.x * 0.5,
                -self.face_size.y * 0.5,
                PLAQUE_RISE,
            ],
            [self.face_size.x * 0.5, -self.face_size.y * 0.5, PLAQUE_RISE],
            [self.face_size.x * 0.5, self.face_size.y * 0.5, PLAQUE_RISE],
            [-self.face_size.x * 0.5, self.face_size.y * 0.5, PLAQUE_RISE],
        ];
        let columns = (self.face_size.x / DARK_REFLECTION_CELL).ceil() as usize;
        let rows = (self.face_size.y / DARK_REFLECTION_CELL).ceil() as usize;
        let mut metal = egui::Mesh::default();
        metal.vertices.reserve(16 + 4 * columns * rows);
        metal.indices.reserve(24 + 6 * columns * rows);
        let mut facet = |corners: [[f32; 3]; 4], normal| {
            let base = metal.vertices.len() as u32;
            for corner in corners {
                metal.colored_vertex(center + project(corner), darkened_bronze(corner, normal));
            }
            metal.add_triangle(base, base + 1, base + 2);
            metal.add_triangle(base, base + 2, base + 3);
        };
        facet([btl, btr, ctr, ctl], [0.0, -PLAQUE_RISE, PLAQUE_BEVEL_RUN]);
        facet([btr, bbr, cbr, ctr], [PLAQUE_RISE, 0.0, PLAQUE_BEVEL_RUN]);
        facet([bbr, bbl, cbl, cbr], [0.0, PLAQUE_RISE, PLAQUE_BEVEL_RUN]);
        facet([bbl, btl, ctl, cbl], [-PLAQUE_RISE, 0.0, PLAQUE_BEVEL_RUN]);
        let point = |x: usize, y: usize| {
            [
                -self.face_size.x * 0.5 + self.face_size.x * x as f32 / columns as f32,
                -self.face_size.y * 0.5 + self.face_size.y * y as f32 / rows as f32,
                PLAQUE_RISE,
            ]
        };
        for y in 0..rows {
            for x in 0..columns {
                facet(
                    [
                        point(x, y),
                        point(x + 1, y),
                        point(x + 1, y + 1),
                        point(x, y + 1),
                    ],
                    [0.0, 0.0, 1.0],
                );
            }
        }
        let _metal = painter.add(Shape::mesh(metal));
        let _silhouette = painter.rect_stroke(
            base,
            0.0,
            Stroke::new(0.7_f32, bronze(0.10)),
            egui::StrokeKind::Inside,
        );
        plaque_engraving(
            painter,
            crown,
            crown.center() - self.galley.size() * 0.5,
            self.galley.clone(),
            PLAQUE_RISE,
            PLAQUE_ETCH_DEPTH,
        );
    }
}

/// A variable-span sheet cut from work-darkened bronze.
///
/// Dynamic numerical registers vary only in x-span, so their casing face
/// remains one planar surface in the common universe. Cells merely sample the
/// finite-eye reflection law densely enough for egui's color interpolation;
/// they do not counterfeit relief.
pub(crate) fn darkened_sheet(painter: &egui::Painter, rect: Rect) {
    let columns = (rect.width() / DARK_REFLECTION_CELL).ceil() as usize;
    let rows = (rect.height() / DARK_REFLECTION_CELL).ceil() as usize;
    let point = |x: usize, y: usize| {
        [
            -rect.width() * 0.5 + rect.width() * x as f32 / columns as f32,
            -rect.height() * 0.5 + rect.height() * y as f32 / rows as f32,
            0.0,
        ]
    };
    let mut mesh = egui::Mesh::default();
    mesh.vertices.reserve(4 * columns * rows);
    mesh.indices.reserve(6 * columns * rows);
    for y in 0..rows {
        for x in 0..columns {
            let base = mesh.vertices.len() as u32;
            for p in [
                point(x, y),
                point(x + 1, y),
                point(x + 1, y + 1),
                point(x, y + 1),
            ] {
                mesh.colored_vertex(
                    rect.center() + project(p),
                    darkened_bronze(p, [0.0, 0.0, 1.0]),
                );
            }
            mesh.add_triangle(base, base + 1, base + 2);
            mesh.add_triangle(base, base + 2, base + 3);
        }
    }
    let _face = painter.add(Shape::mesh(mesh));
    let _edge = painter.rect_stroke(
        rect,
        0.0,
        Stroke::new(0.7_f32, bronze(0.10)),
        egui::StrokeKind::Inside,
    );
}

/// Forge the two rods joining adjacent physical attachment ports. Their ends
/// bury into both casings, so no counterfeit end caps remain visible.
pub(crate) fn tie_pair(left: CouplingPort, right: CouplingPort) -> Shape {
    assert!(
        left.anchors[0].x < right.anchors[0].x,
        "coupling ports must advance left-to-right"
    );
    Shape::Vec(
        left.anchors
            .into_iter()
            .zip(right.anchors)
            .map(|(left, right)| {
                let axis = (right - left).normalized();
                cylinder_between(
                    left - axis * COUPLING_TIE_BURIAL,
                    right + axis * COUPLING_TIE_BURIAL,
                    COUPLING_TIE_DIAMETER,
                )
            })
            .collect(),
    )
}

/// Replay an already projected and illuminated 2D foundry artifact.
pub(crate) fn paint_compiled(painter: &egui::Painter, clip: Rect, mesh: &Arc<egui::Mesh>) {
    let _shape = painter.with_clip_rect(clip).add(Shape::mesh(mesh.clone()));
}

/// Orthographic cylindrical stock with a strictly untapered silhouette. A
/// screen-x roller and a screen-y handle share the same circular section; only
/// the section's orientation against the global light differs.
pub(crate) fn cylinder(painter: &egui::Painter, rect: Rect, axis: StockAxis) {
    let (start, end, diameter) = match axis {
        StockAxis::ScreenX => (
            Pos2::new(rect.left(), rect.center().y),
            Pos2::new(rect.right(), rect.center().y),
            rect.height(),
        ),
        StockAxis::ScreenY => (
            Pos2::new(rect.center().x, rect.top()),
            Pos2::new(rect.center().x, rect.bottom()),
            rect.width(),
        ),
    };
    let _stock = painter.add(cylinder_between(start, end, diameter));
}

fn cylinder_between(start: Pos2, end: Pos2, diameter: f32) -> Shape {
    const BANDS: usize = 14;
    let axis = (end - start).normalized();
    let wing = Vec2::new(-axis.y, axis.x);
    let radius = diameter * 0.5;
    let mut mesh = egui::Mesh::default();
    for band in 0..=BANDS {
        let f = band as f32 / BANDS as f32;
        let s = f * 2.0 - 1.0;
        let nz = (1.0 - s * s).max(0.0).sqrt();
        let ny = wing.y * s;
        let color = turned_bronze(ny, nz);
        let offset = wing * (s * radius);
        mesh.colored_vertex(start + offset, color);
        mesh.colored_vertex(end + offset, color);
        if band > 0 {
            let base = (band as u32 - 1) * 2;
            mesh.add_triangle(base, base + 1, base + 2);
            mesh.add_triangle(base + 1, base + 3, base + 2);
        }
    }
    let edge = Stroke::new(0.7_f32, bronze(0.12));
    Shape::Vec(vec![
        Shape::mesh(mesh),
        Shape::line_segment([start - wing * radius, end - wing * radius], edge),
        Shape::line_segment([start + wing * radius, end + wing * radius], edge),
    ])
}

/// Project a triangular ridge whose two visible strips are its actual planar
/// facets. Line tessellation supplies coverage antialiasing after projection;
/// color still comes from each facet's three-dimensional normal.
pub(crate) fn triangular_ridge(painter: &egui::Painter, segment: [Pos2; 2], width: f32, rise: f32) {
    assert!(width > 0.0 && rise > 0.0, "ridge stock must be positive");
    let axis = (segment[1] - segment[0]).normalized();
    let wing = Vec2::new(-axis.y, axis.x);
    let keyward = if wing.y <= 0.0 { wing } else { -wing };
    let run = width * 0.5;
    let normalizer = (run * run + rise * rise).sqrt();
    let normal = |side: f32| {
        [
            keyward.x * rise * side / normalizer,
            keyward.y * rise * side / normalizer,
            run / normalizer,
        ]
    };
    let facet = |side: f32| {
        let offset = keyward * (side * width * 0.25);
        Shape::line_segment(
            [segment[0] + offset, segment[1] + offset],
            Stroke::new(
                width * 0.5,
                darkened_bronze([0.0, 0.0, rise * 0.5], normal(side)),
            ),
        )
    };
    let _ridge = painter.add(Shape::Vec(vec![facet(1.0), facet(-1.0)]));
}

/// Paint the abyss and its machined inner walls. Contents are inserted after
/// this pass; [`socket_rim`] is struck last so every assembly seats under it.
pub(crate) fn socket_bed(painter: &egui::Painter, rect: Rect) {
    let _void = painter.rect_filled(rect, RIM_RADIUS, ABYSS);
    let _shadow = painter.line_segment(
        [rect.left_top(), rect.right_top()],
        Stroke::new(1.6_f32, Color32::from_rgb(1, 1, 2)),
    );
    let _catch = painter.line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        Stroke::new(RIM_WIDTH, bronze(0.26)),
    );
}

pub(crate) fn socket_rim(painter: &egui::Painter, rect: Rect) {
    let _rim = painter.rect_stroke(
        rect,
        RIM_RADIUS,
        Stroke::new(RIM_WIDTH, bronze(0.13)),
        egui::StrokeKind::Inside,
    );
}

/// A flat part stamped from the common bronze sheet. `crowns` face the global
/// key; `soles` face away. Supplying those die edges gives arrows and detents
/// exactly the same body, crown, and undercut.
pub(crate) fn stamp(
    painter: &egui::Painter,
    silhouette: Vec<Pos2>,
    crowns: &[[Pos2; 2]],
    soles: &[[Pos2; 2]],
    dim: f32,
) {
    let _body = painter.add(Shape::convex_polygon(
        silhouette,
        bronze(0.60 + dim),
        Stroke::NONE,
    ));
    for edge in crowns {
        let _crown = painter.line_segment(*edge, Stroke::new(STAMP_GAUGE, bronze(0.86 + dim)));
    }
    for edge in soles {
        let _sole = painter.line_segment(*edge, Stroke::new(STAMP_GAUGE, bronze(0.18 + dim)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn illuminant_is_unit_length_and_sixty_degrees_above_the_horizon() {
        assert!((LIGHT_Y * LIGHT_Y + LIGHT_Z * LIGHT_Z - 1.0).abs() < 1e-6);
        assert!((LIGHT_Z.asin().to_degrees() - 60.0).abs() < 1e-4);
        assert!((HALF_Y * HALF_Y + HALF_Z * HALF_Z - 1.0).abs() < 1e-6);
    }

    #[test]
    fn darkened_bronze_keeps_a_quiet_body_beneath_its_mirror_glint() {
        let tones = [-15.0, -8.0, 0.0, 8.0, 15.0]
            .map(|y| darkened_metal_tone([0.0, y, 1.0], [0.0, 0.0, 1.0]));
        let mean = tones.iter().sum::<f32>() / tones.len() as f32;
        assert!((0.49..=0.54).contains(&mean));
        assert!(tones[0] - tones[4] > 0.28);
        assert_eq!(tones[0], DARK_TONE_CEILING);
        assert!(tones.into_iter().all(|tone| tone <= DARK_TONE_CEILING));
        let [red, _, _] = darkened_bronze_rgb(DARK_TONE_CEILING);
        assert!(f32::from(red) > BRONZE_BODY[0]);
        assert!(f32::from(red) < BRONZE_GLINT[0]);
    }
}
