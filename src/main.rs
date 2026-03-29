#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use lighter::{ core::{ arena::node::NodePropsExt, style::Transform }, deg, rad };
pub use lighter::{
    core::{
        app::{ window::*, * },
        event::*,
        layout::{ types::{ alignment::*, dimension::*, flex::*, size::*, * }, * },
        reactive::signal::*,
        style::Color,
    },
    elements::{ div::*, text::*, * },
};

pub mod palette {
    use lighter::core::style::Color;

    pub const TEXT: Color = Color::hex(0xecfef3);
    pub const BACKGROUND: Color = Color::hex(0x000f05);
    pub const PRIMARY: Color = Color::hex(0x06ea56);
    pub const SECONDARY: Color = Color::hex(0x049064);
    pub const ACCENT: Color = Color::hex(0x05bda1);
}

fn scene() -> Div {
    let counter = signal(0);
    let evenodd = derived(move || {
        if counter.get() % 2 == 0 { "v even" } else { "x odd" }
    });
    let color = derived(move || {
        if counter.get() % 2 == 0 { Color::GREEN } else { Color::RED }
    });

    page()
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
                        .align(Some(AlignItems::Center))
                        .justify(Some(JustifyContent::Center))
                        .bg(palette::PRIMARY)
                        .transition(|t| {
                            t.color(130)
                                .transform(110)
                                .opacity(90)
                                .radius(130)
                                .ease(TransitionEasing::OutCubic)
                        })
                        .hover(|s| s.bg(palette::PRIMARY.with_alpha(0.8)).opacity(0.92))
                        .active(|s| {
                            s.bg(palette::ACCENT).transform(
                                Transform::scale(0.1, 0.1) * Transform::translate(0.0, 50.0)
                            )
                        })
                        .px(px(40.0))
                        .py(px(20.0))
                        .rounded(16.0)
                        .on_click(move || {
                            counter.update(move |c| {
                                *c += 1;
                            })
                        })
                        .child(
                            text(derived(move || format!("Counter: {}", counter.get())))
                                .color(palette::BACKGROUND)
                                .font_size(20.0)
                                .font_weight(FontWeight::BOLD)
                        )
                )
        )
}

fn root() -> impl Element {
    scene()
}

fn page() -> Div {
    div().size(percent(1.0)).bg(palette::BACKGROUND)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    app()?.add(window().title("COUNTER").mode(WindowMode::Dark).root(root()))?.run()?;

    Ok(())
}
