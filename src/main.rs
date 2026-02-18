use std::{
    sync::atomic::{AtomicU32, Ordering},
    time::Instant,
};

use lighter::{
    core::{
        dirty::DirtyFlags, engine::Engine, node::NodeKind, render::tinyskia::TinySkiaRenderer,
        tree::Tree,
    },
    prelude::*,
};

fn engine_frame(engine: &mut Engine<TinySkiaRenderer>) {
    static FRAME_COUNT: AtomicU32 = AtomicU32::new(0);
    let count = FRAME_COUNT.fetch_add(1, Ordering::Relaxed);
    let output = format!("output/{}.png", count);

    engine.frame().unwrap();
    engine.renderer().save_png(output.as_str()).unwrap();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();

    let page = div()
        .size(percent(1.0))
        .bg(Color::rgba(1.0, 1.0, 1.0, 0.5))
        .align(AlignItems::Center)
        .justify(JustifyContent::Center)
        .key(18)
        .child(
            div()
                .key(42)
                .size(px(200.0))
                .opacity(0.5)
                .bg(Color::rgba(1.0, 0.0, 0.0, 1.0)),
        );

    let tree = Tree::build(page)?;
    let renderer = TinySkiaRenderer::new(600, 600)?;

    let mut engine = Engine::new(tree, renderer);
    engine_frame(&mut engine);

    engine.mutate(
        42,
        DirtyFlags::PAINT | DirtyFlags::LAYOUT,
        |node| match &mut node.kind {
            NodeKind::Div(props) => {
                props.background_color = Color::rgba(0.0, 1.0, 0.0, 1.0);
                node.layout.style.size.width = Dimension::percent(1.0).into();
            }
            _ => {}
        },
    )?;

    engine_frame(&mut engine);

    engine.mutate(
        18,
        DirtyFlags::PAINT | DirtyFlags::LAYOUT,
        |node| match &mut node.kind {
            NodeKind::Div(props) => {
                props.background_color = Color::rgba(0.47, 0.56, 0.29, 1.0);
                node.layout.style.size.width = Dimension::Pixels(400.0).into();
                node.layout.style.size.height = Dimension::Pixels(100.0).into();
            }
            _ => {}
        },
    )?;

    engine_frame(&mut engine);

    println!("elapsed: {:?}", start.elapsed());

    Ok(())
}
