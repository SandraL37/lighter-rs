use std::{ sync::atomic::{ AtomicU32, Ordering }, time::Instant };

use lighter::{
    core::{
        engine::Engine,
        layout::{
            AlignItems,
            ContainerStylePropsExt,
            FlexDirection,
            JustifyContent,
            Size,
            percent,
            px,
        },
        render::piet::PietRenderer,
        signal::signal,
        style::Color,
        tree::Tree,
    },
    elements::{ div::{ ChildrenExt, DivPropsExt, div }, text::{ FontWeight, TextPropsExt, text } },
};

fn engine_frame(engine: &mut Engine<PietRenderer>) {
    static FRAME_COUNT: AtomicU32 = AtomicU32::new(0);
    let count = FRAME_COUNT.fetch_add(1, Ordering::Relaxed);

    if !std::fs::exists("output").unwrap() {
        std::fs::create_dir("output").unwrap();
    }

    let output = format!("output/{}.png", count);

    let t0 = Instant::now();
    engine.renderer().set_output(output);
    engine.frame().unwrap();
    let render_time = t0.elapsed();

    println!("frame: {}, render + save: {:?}", count, render_time);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut counter = 0;
    let counter_text = signal(String::from("Not clicked"));

    let mut click = || {
        counter += 1;
        counter_text.set(format!("Clicked {} time{}", counter, if counter == 1 { "" } else { "s" }));
    };

    let page = div()
        .size(percent(1.0))
        .bg(Color::BLACK)
        .align(AlignItems::Center)
        .justify(JustifyContent::Center)
        .flex_direction(FlexDirection::Column)
        .gap(px(20.0))
        .child(
            text("HELLO WORLD!🎉")
                .font_size(100.0)
                .color(Color::WHITE)
                .font_weight(FontWeight::EXTRA_BLACK)
        )
        .child(
            text(counter_text.read())
                .font_size(30.0)
                .color(Color::WHITE)
                .font_weight(FontWeight::NORMAL)
        );

    let (tree, cx) = Tree::build(page)?;
    let renderer = PietRenderer::new(Size::wh(1920, 1080))?;
    let mut engine = Engine::new(tree, renderer, cx);

    engine_frame(&mut engine);
    click();
    engine_frame(&mut engine);
    click();
    engine_frame(&mut engine);
    click();
    engine_frame(&mut engine);
    click();
    engine_frame(&mut engine);
    click();
    engine_frame(&mut engine);
    Ok(())
}
