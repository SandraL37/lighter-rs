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
    let square = div()
        .min_size(px(100.0))
        .bg(Color::GREEN)
        .rounded(4.0)
        .items_center()
        .justify_center()
        .hover(|s| s.bg(Color::BLUE));
    let counter = signal(0.0f32);

    page()
        .bg(Color::BLACK)
        .items_center()
        .justify_center()
        .flex_column()
        .gap(px(10.0))
        .child(text("OK funziono").color(Color::WHITE))
        .child(
            square
                .child(
                    text(counter)
                        .mx(derived(move || {
                            DefiniteDimensionAuto::length(counter.get().powf(2.0) / 4.0)
                        }))
                        .font_size(derived(move || counter.get().powf(2.0) + 14.0)),
                )
                .on_click(move |e| {
                    counter.update(|c| *c += 1.0);
                    e.stop_propagation();
                }),
        )
        .on_click(move |_| counter.update(|c| *c += 1.0))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    app()?
        .add(
            window()
                .title("COUNTER")
                .size(Size::wh(500, 350))
                .mode(WindowMode::Dark)
                .backdrop(WindowBackdrop::Mica)
                .root(root()),
        )?
        .run()?;

    Ok(())
}
