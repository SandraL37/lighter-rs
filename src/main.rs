#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use lighter::{
    core::{ app::{ window::*, * }, event::*, layout::*, reactive::signal::*, style::* },
    elements::{ Element, div::*, text::* },
};

fn root() -> impl Element {
    let count = signal(0);

    div()
        .size(percent(1.0))
        .bg(Color::WHITE)
        .align(AlignItems::Center)
        .justify(JustifyContent::Center)
        .child(text(count.read().map(|c| { format!("count: {c}") })).font_size(48.0))
        .on_click(move || count.set(count.get() + 1))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    app()?.add(window().title("IT WORKS!").root(root()))?.run()?;

    Ok(())
}
