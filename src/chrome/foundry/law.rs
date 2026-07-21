//! Scalar material and illuminant law shared by runtime chrome and the
//! build-time fixed-view geometry compiler. This module deliberately depends
//! only on `std`, so `build.rs` can consume the exact same alloy constants.

pub(crate) const BRONZE_SHADOW: [f32; 3] = [34.0, 28.0, 19.0];
pub(crate) const BRONZE_BODY: [f32; 3] = [104.0, 86.0, 58.0];
pub(crate) const BRONZE_GLINT: [f32; 3] = [196.0, 170.0, 124.0];

pub(crate) const LIGHT_Y: f32 = -0.5;
pub(crate) const LIGHT_Z: f32 = 0.866_025_4;
pub(crate) const HALF_Y: f32 = -0.258_819_04;
pub(crate) const HALF_Z: f32 = 0.965_925_8;
pub(crate) const METAL_SHINE: f32 = 14.0;
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
