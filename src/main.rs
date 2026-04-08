#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// A prelude::* is needed lol.
use lighter::{
    core::{
        app::{window::*, *},
        event::*,
        layout::{
            types::{dimension::*, size::*},
            *,
        },
        reactive::signal::*,
        state::*,
        style::*,
    },
    elements::{
        div::{style::*, *},
        text::{style::*, *},
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
        .bg(Color::RED.hover(Color::GREEN))
        .rounded(4.0.hover(8.0).active(16.0))
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
