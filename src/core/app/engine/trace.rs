#![cfg(debug_assertions)]

use crate::core::{app::engine::HoverDelta, event::EngineEvent};

macro_rules! trace {
    ($x:expr, $y:block) => {
        if std::env::var(concat!("LIGHTER_TRACE_", $x))
            .map(|v| v == "1")
            .unwrap_or(false)
        $y
    };
}

pub fn trace_engine_event(event: &EngineEvent) {
    trace!("EVENT", { eprintln!("[engine][event] {:?}", event) })
}

pub fn trace_hover_change(hover_delta: &HoverDelta) {
    trace!("HOVER", {
        eprintln!(
            "[engine][hover] old_leaf={:?} new_leaf={:?} old_len={} new_len={}",
            hover_delta.old_leaf,
            hover_delta.new_leaf,
            hover_delta.leaving.len(),
            hover_delta.entering.len(),
        )
    })
}
