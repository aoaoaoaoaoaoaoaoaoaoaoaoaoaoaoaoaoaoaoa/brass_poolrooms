use std::sync::{Arc, OnceLock};

use egui::CustomCursorImage;

mod artifact {
    include!(concat!(env!("OUT_DIR"), "/longinus_cursor.rs"));
}

/// The Poolrooms' bronze Lance of Longinus fork cursor.
///
/// Its fixed three-dimensional foundry model, illumination, antialiasing, and
/// projection are compiled into this native cursor at build time.
pub struct LonginusCursor;

impl LonginusCursor {
    /// Return the immutable native cursor image.
    pub fn image() -> CustomCursorImage {
        static RGBA: OnceLock<Arc<[u8]>> = OnceLock::new();
        CustomCursorImage {
            rgba: Arc::clone(RGBA.get_or_init(|| Arc::from(artifact::RGBA))),
            size: [artifact::SIDE; 2],
            hotspot: artifact::HOTSPOT,
        }
    }
}
