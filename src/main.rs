#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use lighter::{
    core::{
        application::Application,
        layout::{
            AlignItems, ContainerStylePropsExt, FlexDirection, JustifyContent, Padding, percent, px,
        },
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
    div()
        .p(Padding::xy(px(30.0), px(30.0)))
        .flex_direction(FlexDirection::Column)
        .gap(px(6.0))
        .child(
            text("Hello World")
                .font_size(132.0)
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
    let page = center(div().child(scene())).bg(Color::WHITE);

    let mut app = Application::new()?;
    app.add_window(window().title("lighter").root(page))?;

    app.run();
    Ok(())
}

// TODO: resizing is very slow / lagging. Resizing is terrible, needs to be fixed asap
/*
BUGGED CODE:
when resizing the black rectangle follows the width (not the height) of the window

    let page = div().size(percent(1.0)).bg(Color::RED).child(
    div()
        .w(px(800.0))
        .h(px(600.0))
        .bg(Color::BLACK)
        .p(Padding::uniform(px(10.0)))
        .child(div().bg(Color::WHITE).size(percent(1.0))),
);
*/
