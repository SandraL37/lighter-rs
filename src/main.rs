#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use lighter::prelude::*;

fn scene(label: impl IntoTextContent) -> Div {
    div()
        .bg(Color::WHITE)
        .size(percent(1.0))
        .align(AlignItems::Center)
        .justify(JustifyContent::Center)
        .child(
            text(label)
                .font_size(28.0)
                .font_weight(FontWeight::MEDIUM)
                .font_family("Cascadia Code"),
        )
}

fn root_a() -> impl Element {
    let label = signal(String::from("Hello, World!"));

    scene(label).on_click(move || {
        label.update(|l| {
            *l += "a";
        })
    })
}

fn root_b() -> impl Element {
    let label = signal(String::from("Hello, World!"));

    scene(label).on_click(move || {
        label.update(|l| {
            *l += "b";
        })
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    app()?
        .add(
            window()
                .title("Click for a")
                .backdrop(WindowBackdrop::Mica)
                .root(root_a()),
        )?
        .add(
            window()
                .title("Click for b")
                .backdrop(WindowBackdrop::Mica)
                .root(root_b()),
        )?
        .run()?;

    Ok(())
}
