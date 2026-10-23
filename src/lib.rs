#![cfg_attr(
    test,
    expect(
        unused_crate_dependencies,
        reason = "egui-winit belongs to the dev-only live gallery host"
    )
)]
#![doc = include_str!("../README.md")]

pub mod chrome;
mod tide;

#[cfg(feature = "foundry-atelier")]
#[doc(hidden)]
pub mod atelier;

#[cfg(feature = "water")]
pub mod water;

#[cfg(feature = "instrumentation")]
pub mod instrumentation;

pub use egui;

#[cfg(feature = "water")]
pub use egui_wgpu;

/// Register a widget rectangle for an external deterministic UI driver under
/// a semantic path and, optionally, the raw runtime identity of one repeated
/// mechanism. The entire expression disappears when the invoking crate's
/// `instrumentation` feature is disabled.
#[macro_export]
macro_rules! poolroom_anchor {
    ($ui:expr, $path:expr, $rect:expr) => {{
        #[cfg(feature = "instrumentation")]
        $crate::instrumentation::record($ui, $path, None, $rect);
    }};
    ($ui:expr, $path:expr, $identity:expr, $rect:expr) => {{
        #[cfg(feature = "instrumentation")]
        $crate::instrumentation::record($ui, $path, Some($identity), $rect);
    }};
}
