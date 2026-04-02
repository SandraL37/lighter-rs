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
        style::Color,
    },
    elements::{
        div::{style::*, *},
        text::*,
        *,
    },
};

fn page() -> Div {
    div().size(percent(1.0))
}

fn root() -> impl Element {
    let square = div()
        .size(px(100.0))
        .bg(Color::GREEN)
        .hover(|s| s.bg(Color::BLUE))
        .rounded(4.0);

    page()
        .bg(Color::BLACK)
        .items_center()
        .justify_center()
        .child(square)
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
