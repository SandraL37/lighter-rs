#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// A prelude::* is needed lol.
use lighter::{
    core::{
        animation::Interpolate,
        app::{window::*, *},
        event::MouseEvents,
        layout::{
            types::{dimension::*, size::*},
            *,
        },
        reactive::signal::*,
        state::StateExt,
        style::*,
    },
    elements::{
        div::{style::DivStyleBuilder, *},
        text::{style::TextStyleBuilder, *},
        *,
    },
};

fn page() -> Div {
    div().size(percent(1.0))
}

fn root() -> impl Element {
    let square = div()
        .items_center()
        .justify_center()
        .bg(Color::interpolate(Color::GREEN, Color::RED, 0.0)
            .hover(Color::interpolate(Color::GREEN, Color::RED, 0.5))
            .active(Color::interpolate(Color::GREEN, Color::RED, 1.0)))
        .rounded((4.0).hover(8.0).active(16.0))
        .min_size(px(100.0));

    let counter = signal(0.0f32);

    page()
        .bg(Color::WHITE)
        .items_center()
        .justify_center()
        .flex_column()
        .gap(px(10.0))
        .child(text("OK funziono").color(Color::BLACK.hover(Color::CYAN)))
        .child(
            square
                .child(text(counter).font_size(30.0))
                .on_click(move |_| {
                    counter.update(|c| {
                        *c += 1.0;
                    })
                }),
        )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    app()?
        .add(
            window()
                .title("COUNTER")
                .size(Size::wh(500, 350))
                .mode(WindowMode::Light)
                .backdrop(WindowBackdrop::Mica)
                .root(root()),
        )?
        .run()?;

    Ok(())
}
