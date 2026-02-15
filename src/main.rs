use lighter::{
    core::{engine::Engine, render::tinyskia::TinySkiaRenderer, tree::Tree},
    elements::Element,
    prelude::*,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let page = div()
        .size(percent(1.0))
        .bg(Color::rgba(1.0, 1.0, 1.0, 0.5))
        .align(AlignItems::Center)
        .justify(JustifyContent::Center)
        .child(some_squares());

    let tree = Tree::build(page)?;
    let renderer = TinySkiaRenderer::new(600, 600)?;

    let mut engine = Engine::new(tree, renderer);
    engine.frame()?;
    engine.renderer().save_png("output.png")?;

    Ok(())
}

fn some_squares() -> impl Element {
    div()
        .align(AlignItems::Center)
        .justify(JustifyContent::Center)
        .gap(px(10.0))
        .bg(Color::rgba(0.68, 0.65, 0.23, 0.5))
        .flex_wrap(FlexWrap::Wrap)
        .p(Padding::uniform(px(20.0)))
        .child(
            div()
                .p(Padding::uniform(px(10.0)))
                .bg(Color::rgba(0.86, 0.56, 0.32, 1.0))
                .child(div().w(px(100.0)).h(px(50.0)).bg(Color::WHITE)),
        )
        .child(
            div()
                .w(px(123.0))
                .h(px(85.0))
                .bg(Color::rgba(0.68, 0.56, 0.32, 1.0)),
        )
        .child(
            div()
                .w(px(75.0))
                .h(px(123.0))
                .bg(Color::rgba(0.86, 0.65, 0.32, 1.0)),
        )
        .child(
            div()
                .w(px(89.0))
                .h(px(97.0))
                .bg(Color::rgba(0.86, 0.56, 0.23, 1.0)),
        )
}
