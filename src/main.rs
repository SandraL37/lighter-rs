#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
pub use lighter::{
    core::{
        app::{window::*, *},
        event::*,
        layout::{
            types::{alignment::*, dimension::*, flex::*, size::*, *},
            *,
        },
        reactive::signal::*,
        style::Color,
    },
    elements::{div::*, text::*, *},
};

fn scene() -> Div {
    let counter = signal(0);
    let evenodd = derived(move || {
        if counter.get() % 2 == 0 {
            "v even"
        } else {
            "x odd"
        }
    });
    let color = derived(move || {
        if counter.get() % 2 == 0 {
            Color::GREEN
        } else {
            Color::RED
        }
    });

    div()
        .bg(Color::WHITE)
        .size(percent(1.0))
        .align(Some(AlignItems::Center))
        .justify(Some(JustifyContent::Center))
        .child(
            div()
                .align(Some(AlignItems::Center))
                .flex_direction(FlexDirection::Column)
                .gap(px(10.0))
                .child(text(evenodd).color(color).font_size(20.0))
                .child(
                    div()
                        .bg(Color::BLUE)
                        .px(px(10.0))
                        .py(px(5.0))
                        .rounded(16.0)
                        .on_click(move || {
                            counter.update(move |c| {
                                *c += 1;
                            })
                        })
                        .child(
                            text(derived(move || format!("Counter: {}", counter.get())))
                                .color(Color::WHITE)
                                .font_size(28.0)
                                .font_weight(FontWeight::MEDIUM)
                                .font_family("Cascadia Code"),
                        ),
                ),
        )
}

fn root() -> impl Element {
    scene()
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
