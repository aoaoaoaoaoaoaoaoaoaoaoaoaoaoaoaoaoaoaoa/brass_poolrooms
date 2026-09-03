//! Shared wheel/trackpad quantization for crafted mechanisms.

const CLAIM: &str = "poolrooms-control-wheel-claim";
const BANK: &str = "poolrooms-control-wheel-bank";
const POINTS_PER_NOTCH: f32 = 50.0;
const MAX_NOTCHES_PER_FRAME: i32 = 8;

/// Take the per-frame flag indicating that crafted chrome consumed wheel motion.
///
/// This destructive read is useful when a control sits inside an enclosing
/// scroll surface that must cancel its own response to the same gesture. A
/// second call before another control consumes motion returns `false`.
pub fn take_control_wheel(ctx: &egui::Context) -> bool {
    ctx.data_mut(|data| {
        data.remove_temp::<bool>(egui::Id::new(CLAIM))
            .unwrap_or(false)
    })
}

/// Claim this frame's unmodified vertical wheel travel and quantize it into
/// integer mechanism notches, banking a trackpad's fractional travel by `id`.
pub(super) fn notches(ui: &egui::Ui, id: egui::Id) -> i32 {
    let Frame { delta, scrolling } = frame(ui);
    if !scrolling {
        return 0;
    }
    claim(ui);
    let Some(delta) = delta else {
        return 0;
    };
    ui.ctx().data_mut(|data| {
        let id = id.with(BANK);
        let mut bank = data.get_temp::<f32>(id).unwrap_or_default();
        if bank != 0.0 && bank.signum() != delta.signum() {
            bank = 0.0;
        }
        bank += delta;
        let notches = bank.trunc();
        bank -= notches;
        let _old = data.insert_temp(id, bank);
        (notches as i32).clamp(-MAX_NOTCHES_PER_FRAME, MAX_NOTCHES_PER_FRAME)
    })
}

/// Take one direction-only stroke for a precision actuator.
///
/// Backends disagree about whether one physical mouse detent is one line,
/// several lines, or dozens of points. Collapsing all vertical travel
/// delivered in one frame makes the smallest caller-owned quantum invariant
/// across those encodings.
pub(super) fn stroke(ui: &egui::Ui) -> i32 {
    let Frame { delta, scrolling } = frame(ui);
    if !scrolling {
        return 0;
    }
    claim(ui);
    delta.map_or(0, |delta| delta.signum() as i32)
}

#[derive(Clone, Copy)]
struct Frame {
    delta: Option<f32>,
    scrolling: bool,
}

fn frame(ui: &egui::Ui) -> Frame {
    ui.input(|input| {
        let mut line = 0.0_f32;
        let mut point = 0.0_f32;
        for event in &input.events {
            if let egui::Event::MouseWheel {
                unit,
                delta,
                modifiers,
                ..
            } = event
                && unmodified(*modifiers)
            {
                match unit {
                    egui::MouseWheelUnit::Line | egui::MouseWheelUnit::Page => line += delta.y,
                    egui::MouseWheelUnit::Point => point += delta.y,
                }
            }
        }
        // Some stacks emit paired line events for one physical notch. Treat a
        // frame's line/page stream as one deliberate notch; meter the finer
        // point stream continuously across frames.
        let line = if line.abs() > f32::EPSILON {
            line.signum()
        } else {
            0.0
        };
        let delta = line + point / POINTS_PER_NOTCH;
        Frame {
            delta: (delta.abs() > 1e-4).then_some(delta),
            scrolling: input.smooth_scroll_delta.y.abs() > f32::EPSILON
                || input.events.iter().any(vertical_unmodified),
        }
    })
}

fn vertical_unmodified(event: &egui::Event) -> bool {
    matches!(
        event,
        egui::Event::MouseWheel {
            delta,
            modifiers,
            ..
        } if delta.y.abs() > f32::EPSILON && unmodified(*modifiers)
    )
}

fn unmodified(modifiers: egui::Modifiers) -> bool {
    !modifiers.ctrl && !modifiers.command && !modifiers.alt
}

fn claim(ui: &egui::Ui) {
    ui.ctx().input_mut(|input| {
        input.events.retain(|event| !vertical_unmodified(event));
        input.smooth_scroll_delta.y = 0.0;
    });
    ui.ctx().data_mut(|data| {
        let _old = data.insert_temp(egui::Id::new(CLAIM), true);
    });
}
