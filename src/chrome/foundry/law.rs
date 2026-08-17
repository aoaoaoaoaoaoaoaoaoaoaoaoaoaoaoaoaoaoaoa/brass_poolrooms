//! Poolrooms gauges adjoining the public Brass Foundry material law.

pub(crate) use brass_foundry::*;

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
        top_half: socket_half * (89.0 / 132.0),
        body_half: socket_half * (49.0 / 66.0),
    }
}
