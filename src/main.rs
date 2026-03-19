#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use lighter::{
    core::{
        app::{window::*, *},
        event::*,
        layout::{types::size::*, *},
        reactive::signal::*,
        style::Color,
    },
    elements::{div::*, text::*, *},
};
use taffy::FlexDirection;

fn root() -> impl Element {
    let count = signal(0);
    let label = derived(move || count.get().to_string());
    let direction = derived(move || {
        if count.get() % 2 == 0 {
            FlexDirection::Column
        } else {
            FlexDirection::ColumnReverse
        }
    });

    let color = signal(Color::RED);

    div()
        .bg(color)
        .size(percent(1.0))
        .align(AlignItems::Center)
        .justify(JustifyContent::Center)
        .flex_direction(direction)
        .child(
            text(label)
                .font_size(148.0)
                .font_weight(FontWeight::BOLD)
                .color(Color::WHITE)
                .font_family("Cascadia Code"),
        )
        .child(
            text(label)
                .font_size(30.0)
                .font_weight(FontWeight::BOLD)
                .color(Color::WHITE)
                .font_family("Cascadia Code"),
        )
        .on_click(move || {
            count.update(|c| {
                *c += 1;
            })
        })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    app()?
        .add(
            window()
                .title("simple counter!")
                .mode(WindowMode::Dark)
                .backdrop(WindowBackdrop::Mica)
                .size(Size::wh(300, 300))
                .root(root()),
        )?
        .run()?;

    Ok(())
}
