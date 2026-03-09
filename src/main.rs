#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use lighter::{
    core::{
        application::Application,
        layout::{AlignItems, ContainerStylePropsExt, JustifyContent, percent},
        style::Color,
        window::window,
    },
    elements::{
        Element,
        div::{ChildrenExt, Div, DivPropsExt, div},
        text::{FontWeight, TextPropsExt, text},
    },
};

fn scene() -> impl Element {
    center(
        text("Hello, world! 🎉")
            .font_family("Cascadia Code")
            .font_size(50.0)
            .font_weight(FontWeight::BOLD),
    )
}

fn center(element: impl Element + 'static) -> Div {
    div()
        .size(percent(1.0))
        .align(AlignItems::Center)
        .justify(JustifyContent::Center)
        .child(element)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let page = div().child(scene()).bg(Color::WHITE).size(percent(1.0));

    let mut app = Application::new()?;
    app.add_window(window().title("lighter").root(page))?;

    app.run();
    Ok(())
}
