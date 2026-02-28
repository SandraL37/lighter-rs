use lighter::{
    core::{
        application::Application,
        layout::{
            AlignItems, ContainerStylePropsExt, FlexDirection, JustifyContent, Padding, auto,
            percent, px,
        },
        style::Color,
        window::window,
    },
    elements::{
        div::{ChildrenExt, DivPropsExt, div},
        text::{FontWeight, TextPropsExt, text},
    },
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let page = div().size(percent(1.0)).bg(Color::RED).child(
        div()
            .w(px(800.0))
            .h(px(600.0))
            .bg(Color::BLACK)
            .p(Padding::uniform(px(10.0)))
            .child(div().bg(Color::WHITE).size(percent(1.0))),
    );

    let mut app = Application::new()?;
    app.add_window(window().root(page))?;

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
