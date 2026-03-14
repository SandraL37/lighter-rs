#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use lighter::{
    core::{
        app::{window::*, *},
        event::*,
        layout::*,
        reactive::signal::*,
        style::*,
    },
    elements::{div::*, text::*},
};

fn button(label: &str, on_click: impl Fn() + 'static) -> Div {
    div()
        .bg(Color::hex(0x2563EB))
        .rounded(8.0)
        .p(Padding::new(px(12.0), px(24.0), px(12.0), px(24.0)))
        .align(AlignItems::Center)
        .justify(JustifyContent::Center)
        .child(
            text(label)
                .font_size(24.0)
                .color(Color::WHITE)
                .font_weight(FontWeight::BOLD),
        )
        .on_click(on_click)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let count = signal(0);
    let word = signal("Counter");

    let count_text = count.read().map(|n: i32| format!("{}", n));

    let inc = count.clone();
    let dec = count.clone();

    let page = div()
        .size(percent(1.0))
        .flex_direction(FlexDirection::Column)
        .justify(JustifyContent::Center)
        .align(AlignItems::Center)
        .gap(px(16.0))
        .child(
            text(word.read())
                .color(Color::WHITE)
                .font_size(50.0)
                .font_weight(FontWeight::BOLD)
                .on_click(move || {
                    if word.get() == "Counter" {
                        word.set("Easter Egg!!");
                    } else {
                        word.set("Counter");
                    }
                }),
        )
        .child(
            text(count_text)
                .color(Color::WHITE)
                .font_size(64.0)
                .font_family("Cascadia Code"),
        )
        .child(
            div()
                .flex_direction(FlexDirection::Row)
                .gap(px(12.0))
                .child(button("\u{2212}", move || dec.set(dec.get() - 1)))
                .child(button("+", move || inc.set(inc.get() + 1))),
        );

    let mut app = application()?;
    let window = window()
        .title("lighter \u{2014} counter")
        .mode(WindowMode::Dark)
        .root(page);

    app.add_window(window)?;
    app.run();
    Ok(())
}
