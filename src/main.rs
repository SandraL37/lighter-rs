#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use lighter::{
    core::{
        app::{window::*, *},
        event::*,
        layout::{types::size::*, *},
        reactive::signal::*,
        style::*,
    },
    elements::{div::*, text::*, *},
};

fn root() -> impl Element {
    let count = signal(0);
    let label = derived(move || format!("count: {}", count.get()));

    div()
        .size(percent(1.0))
        .bg(Color::WHITE)
        .align(AlignItems::Center)
        .justify(JustifyContent::Center)
        .child(text(label).font_size(48.0).on_click(move || {
            count.update(|c| {
                *c += 1;
            })
        }))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    app()?
        .add(window().title("simple counter!").root(root()))?
        .run()?;

    Ok(())
}
