#![cfg(debug_assertions)]

use crate::core::{arena::node::NodeId, event::EngineEvent};

fn trace_interaction_enabled() -> bool {
    std::env::var("LIGHTER_TRACE_INTERACTION")
        .map(|v| v == "1")
        .unwrap_or(false)
}

pub fn trace_engine_event(event: &EngineEvent) {
    if trace_interaction_enabled() {
        eprintln!("[engine][event] {:?}", event)
    }
}

pub fn trace_hover_change(
    old_leaf: Option<NodeId>,
    new_leaf: Option<NodeId>,
    old_len: usize,
    new_len: usize,
) {
    if trace_interaction_enabled() {
        eprint!(
            "[engine][hover] old_leaf={old_leaf:?} new_leaf={new_leaf:?} old_len={old_len} new_len={new_len}"
        )
    }
}
