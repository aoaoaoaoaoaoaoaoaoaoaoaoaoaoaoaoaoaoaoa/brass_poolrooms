//! Scalar material and illuminant law shared by runtime chrome and the
//! build-time fixed-view geometry compiler. This module deliberately depends
//! only on `std`, so `build.rs` can consume the exact same alloy constants.

pub(crate) const BRONZE_SHADOW: [f32; 3] = [34.0, 28.0, 19.0];
pub(crate) const BRONZE_BODY: [f32; 3] = [104.0, 86.0, 58.0];
pub(crate) const BRONZE_GLINT: [f32; 3] = [196.0, 170.0, 124.0];
/// Largest interpolation cell admitted by the dark-bronze reflection lobe.
pub(crate) const DARK_REFLECTION_CELL: f32 = 4.0;

pub(crate) const LIGHT_Y: f32 = -0.5;
pub(crate) const LIGHT_Z: f32 = 0.866_025_4;
pub(crate) const HALF_Y: f32 = -0.258_819_04;
pub(crate) const HALF_Z: f32 = 0.965_925_8;
pub(crate) const METAL_SHINE: f32 = 14.0;
pub(crate) const MIRROR_SHINE: f32 = 72.0;
pub(crate) const DARK_AMBIENT: f32 = 0.13;
pub(crate) const DARK_DIFFUSE_WEIGHT: f32 = 0.32;
pub(crate) const DARK_BROAD_WEIGHT: f32 = 0.01;
pub(crate) const DARK_BROAD_SHINE: f32 = 12.0;
pub(crate) const DARK_GLINT_WEIGHT: f32 = 3.0;
pub(crate) const DARK_GLINT_SHINE: f32 = 128.0;
pub(crate) const DARK_TONE_CEILING: f32 = 0.72;
pub(crate) const DARK_EXPOSURE: f32 = 1.20;
pub(crate) const MECHANISM_SIDE_SMALL: u8 = 20;
pub(crate) const MECHANISM_SIDE_MEDIUM: u8 = 24;
pub(crate) const MECHANISM_SIDE_LARGE: u8 = 32;
#[allow(
    dead_code,
    reason = "the complete bake roster is consumed only by the build-time geometry compiler"
)]
pub(crate) const MECHANISM_SIDES: [u8; 3] = [
    MECHANISM_SIDE_SMALL,
    MECHANISM_SIDE_MEDIUM,
    MECHANISM_SIDE_LARGE,
];

/// X-y dimensions of one momentary mechanism gauge. Z travel and cutting-tool
/// depths remain common foundry stock, so changing the footprint changes the
/// bevel normals rather than resampling a finished projection.
#[derive(Clone, Copy)]
pub(crate) struct MomentaryGauge {
    pub(crate) socket_half: f32,
    pub(crate) top_half: f32,
    pub(crate) body_half: f32,
}

pub(crate) const fn momentary_gauge(side: u8) -> MomentaryGauge {
    let side = side as f32;
    let socket_half = side * 0.5;
    MomentaryGauge {
        socket_half,
        // Preserve the established crown-to-socket proportions while making
        // the public mechanism gauge denote the casing itself.
        top_half: socket_half * (89.0 / 132.0),
        body_half: socket_half * (49.0 / 66.0),
    }
}

/// Fixed perspective eye above the assembly plane, in logical points.
#[allow(
    dead_code,
    reason = "consumed only by the build-time geometry compiler"
)]
pub(crate) const EYE_Z: f32 = 80.0;

pub(crate) fn bronze_rgb(tone: f32) -> [u8; 3] {
    let tone = tone.clamp(0.0, 1.0);
    let (lo, hi, t) = if tone < 0.6 {
        (BRONZE_SHADOW, BRONZE_BODY, tone / 0.6)
    } else {
        (BRONZE_BODY, BRONZE_GLINT, (tone - 0.6) / 0.4)
    };
    let channel = |i: usize| (lo[i] + (hi[i] - lo[i]) * t).round() as u8;
    [channel(0), channel(1), channel(2)]
}

pub(crate) fn darkened_bronze_rgb(tone: f32) -> [u8; 3] {
    let rgb = bronze_rgb(tone);
    rgb.map(|channel| {
        (f32::from(channel) * DARK_EXPOSURE)
            .round()
            .clamp(0.0, 255.0) as u8
    })
}

/// Material illumination at one point in the common finite-eye universe.
/// Arrays keep this law independent of both egui and the build compiler's
/// private vector representation.
#[allow(
    dead_code,
    reason = "polished finite-eye response is consumed only by the build-time geometry compiler"
)]
pub(crate) fn metal_tone(position: [f32; 3], normal: [f32; 3]) -> f32 {
    metal_tone_with_key(position, normal, 1.0)
}

/// Bare bronze brought to a mirror polish for ceremonial edge tools.
#[allow(
    dead_code,
    reason = "ceremonial polished response is consumed only by the build-time geometry compiler"
)]
pub(crate) fn polished_metal_tone(position: [f32; 3], normal: [f32; 3]) -> f32 {
    let (diffuse, reflection) = material_terms(position, normal);
    (0.13 + 0.43 * diffuse + 1.85 * reflection.powf(MIRROR_SHINE)).min(1.0)
}

/// The same bronze charge beneath a work-darkened oxide film.
///
/// Stamped detents established this quieter register before the larger
/// mechanisms existed. Oxide supplies a dark diffuse body while the underlying
/// conductor retains broad reflection and a tight directional glint; turned
/// roller stock keeps [`metal_tone`]'s bare polished response.
pub(crate) fn darkened_metal_tone(position: [f32; 3], normal: [f32; 3]) -> f32 {
    darkened_metal_tone_with_key(position, normal, 1.0)
}

/// Material illumination with explicit geometric visibility of the distant
/// key. Occlusion extinguishes diffuse and specular response while preserving
/// the foundry's ambient term.
#[allow(
    dead_code,
    reason = "key visibility is consumed only by the build-time geometry compiler"
)]
pub(crate) fn metal_tone_with_key(
    position: [f32; 3],
    normal: [f32; 3],
    key_visibility: f32,
) -> f32 {
    let (diffuse, reflection) = material_terms(position, normal);
    0.16 + key_visibility.clamp(0.0, 1.0) * (0.5 * diffuse + 0.8 * reflection.powf(METAL_SHINE))
}

/// Work-darkened bronze illumination with explicit distant-key visibility.
pub(crate) fn darkened_metal_tone_with_key(
    position: [f32; 3],
    normal: [f32; 3],
    key_visibility: f32,
) -> f32 {
    let (diffuse, reflection) = material_terms(position, normal);
    (DARK_AMBIENT
        + key_visibility.clamp(0.0, 1.0)
            * (DARK_DIFFUSE_WEIGHT * diffuse
                + DARK_BROAD_WEIGHT * reflection.powf(DARK_BROAD_SHINE)
                + DARK_GLINT_WEIGHT * reflection.powf(DARK_GLINT_SHINE)))
    .min(DARK_TONE_CEILING)
}

pub(crate) fn material_terms(position: [f32; 3], normal: [f32; 3]) -> (f32, f32) {
    let normalize = |[x, y, z]: [f32; 3]| {
        let length = (x * x + y * y + z * z).sqrt().max(f32::EPSILON);
        [x / length, y / length, z / length]
    };
    let [nx, ny, nz] = normalize(normal);
    let [vx, vy, vz] = normalize([-position[0], -position[1], EYE_Z - position[2]]);
    let [hx, hy, hz] = normalize([vx, vy + LIGHT_Y, vz + LIGHT_Z]);
    let diffuse = (ny * LIGHT_Y + nz * LIGHT_Z).max(0.0);
    let reflection = (nx * hx + ny * hy + nz * hz).max(0.0);
    (diffuse, reflection)
}
