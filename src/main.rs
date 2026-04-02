#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use lighter::{
    core::{
        app::{window::*, *},
        arena::node::NodeStyleBuilder,
        event::*,
        layout::{
            types::{alignment::*, dimension::*, flex::*, size::*, *},
            *,
        },
        reactive::signal::*,
        style::{Color, Transform},
    },
    elements::{
        div::{style::*, *},
        text::{style::TextStyleBuilder, *},
        *,
    },
};

fn page() -> Div {
    div().size(percent(1.0))
}

fn root() -> impl Element {
    let square = div().size(px(100.0)).bg(Color::GREEN).rounded(4.0);
    let counter = signal(0);

    page()
        .bg(Color::BLACK)
        .items_center()
        .justify_center()
        .flex_column()
        .gap(px(10.0))
        .child(text("OK funziono").color(Color::WHITE))
        .child(square.on_click(move |e| {
            counter.update(|c| *c += 1);
            e.stop_propagation();
        }))
        .child(text(counter).color(Color::WHITE))
        .on_click(move |_| counter.update(|c| *c += 1))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    app()?
        .add(
            window()
                .title("COUNTER")
                .backdrop(WindowBackdrop::Mica)
                .root(root()),
        )?
        .run()?;

    Ok(())
}
