use std::{
    sync::atomic::{AtomicU32, Ordering},
    time::Instant,
};

use lighter::{
    core::{
        engine::Engine,
        layout::{
            AlignItems, ContainerStylePropsExt, FlexDirection, JustifyContent, Size, percent, px,
        },
        render::tinyskia::TinySkiaRenderer,
        signal::signal,
        style::Color,
        tree::Tree,
    },
    elements::{
        div::{ChildrenExt, DivPropsExt, div},
        text::text,
    },
};

fn engine_frame(engine: &mut Engine<TinySkiaRenderer>) {
    static FRAME_COUNT: AtomicU32 = AtomicU32::new(0);
    let count = FRAME_COUNT.fetch_add(1, Ordering::Relaxed);
    let output = format!("output/{}.png", count);

    let t0 = Instant::now();
    engine.frame().unwrap();
    let render_time = t0.elapsed();

    let t1 = Instant::now();
    engine.renderer().save_png(output.as_str()).unwrap();
    let save_time = t1.elapsed();
    println!(
        "frame: {}, render: {:?}, png save: {:?}",
        count, render_time, save_time
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let width = signal(px(100.0));
    let square = |color: Color| div().size(px(200.0)).bg(color);
    let fd = signal(FlexDirection::Column);

    let page = div()
        .size(percent(1.0))
        .bg(Color::RED)
        .align(AlignItems::Center)
        .justify(JustifyContent::Center)
        .flex_direction(fd.read())
        .gap(px(10.0))
        .child(square(Color::ORANGE).w(width.read()))
        .child(square(Color::ORANGE))
        .child(text("ciao"));

    let (tree, cx) = Tree::build(page)?;
    let renderer = TinySkiaRenderer::new(600, 600)?;
    let mut engine = Engine::new(tree, renderer, cx);

    engine_frame(&mut engine);

    width.set(px(200.0));

    engine_frame(&mut engine);

    fd.set(FlexDirection::Row);

    engine_frame(&mut engine);

    for _ in 1..10 {
        engine_frame(&mut engine);
    }

    Ok(())
}
