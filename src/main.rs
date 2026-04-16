#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use lighter::{
    core::{
        animation::{DurationExt, Easing},
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
        .bg(Color::GREEN
            .hover(Color::GREEN.darken(0.2))
            .transition(200.ms(), Easing::EASE_IN)
            .active(Color::RED))
        .rounded(8.0)
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
