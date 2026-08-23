//! Fixed-stock hardware for physically locking out a rectangular control.

use egui::{Color32, Painter, Pos2, Rect, Stroke, StrokeKind, Vec2, pos2, vec2};

use super::foundry;

const FRAME_INSET: f32 = 2.5;
const BAR_PITCH: f32 = 24.0;
const BAR_SKEW: f32 = 2.5;
const SHADOW_STOCK: f32 = 2.6;
const WIRE_STOCK: f32 = 1.35;
const WELD_RADIUS: f32 = 1.45;

pub(super) fn paint(painter: &Painter, rect: Rect) {
    let pixel = painter.pixels_per_point().recip();
    let grille = Rect::from_min_max(
        snap(rect.min + Vec2::splat(FRAME_INSET), pixel),
        snap(rect.max - Vec2::splat(FRAME_INSET), pixel),
    );
    if grille.width() <= BAR_PITCH || grille.height() <= WIRE_STOCK * 4.0 {
        return;
    }

    let painter = painter.with_clip_rect(rect);
    let shadow = Stroke::new(SHADOW_STOCK, Color32::from_black_alpha(220));
    let stock = Stroke::new(WIRE_STOCK, foundry::bronze(0.46));
    let glint = Stroke::new(pixel, foundry::bronze(0.88));
    let shadow_shift = vec2(pixel, pixel);
    let glint_shift = vec2(-pixel * 0.5, -pixel * 0.5);
    let bars = bars(grille, pixel);

    for [top, bottom] in &bars {
        let _shadow = painter.line_segment([*top + shadow_shift, *bottom + shadow_shift], shadow);
    }
    let _frame_shadow = painter.rect_stroke(
        grille.translate(shadow_shift),
        1.0,
        shadow,
        StrokeKind::Middle,
    );

    for [top, bottom] in &bars {
        let _stock = painter.line_segment([*top, *bottom], stock);
        let _glint = painter.line_segment([*top + glint_shift, *bottom + glint_shift], glint);
    }
    let _frame = painter.rect_stroke(grille, 1.0, stock, StrokeKind::Middle);
    let _frame_glint = painter.rect_stroke(
        grille.translate(glint_shift),
        1.0,
        glint,
        StrokeKind::Middle,
    );

    for endpoint in bars.into_iter().flatten() {
        let _shadow = painter.circle_filled(endpoint + shadow_shift, WELD_RADIUS, Color32::BLACK);
        let _weld = painter.circle_filled(endpoint, WELD_RADIUS, foundry::bronze(0.50));
        let _glint = painter.circle_filled(
            endpoint + glint_shift,
            pixel.max(WELD_RADIUS * 0.34),
            foundry::bronze(0.92),
        );
    }
}

fn bars(rect: Rect, pixel: f32) -> Vec<[Pos2; 2]> {
    let count = (rect.width() / BAR_PITCH).floor() as usize;
    let gap = rect.width() / (count + 1) as f32;
    (1..=count)
        .map(|index| {
            let x = snap_scalar(rect.left() + gap * index as f32, pixel);
            [
                pos2((x - BAR_SKEW).max(rect.left()), rect.top()),
                pos2((x + BAR_SKEW).min(rect.right()), rect.bottom()),
            ]
        })
        .collect()
}

fn snap(point: Pos2, pixel: f32) -> Pos2 {
    pos2(snap_scalar(point.x, pixel), snap_scalar(point.y, pixel))
}

fn snap_scalar(value: f32, pixel: f32) -> f32 {
    (value / pixel).round() * pixel
}
