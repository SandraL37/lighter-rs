#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use lighter::{app::*, core::*, render::*, window::*};
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_ALPHA_MODE_PREMULTIPLIED, DXGI_FORMAT_B8G8R8A8_UNORM,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = App::new()?;
    let renderer = Renderer::new()?;

    let window = Window::new(&app, WindowStyle::default())?;
    let window_ctx = renderer.create_window_context(window.hwnd())?;

    let width = 600.0;
    let height = 600.0;

    let surface = renderer.create_surface(
        Size::new(width, height),
        DXGI_FORMAT_B8G8R8A8_UNORM,
        DXGI_ALPHA_MODE_PREMULTIPLIED,
    )?;

    let layer = CompositionLayer::new(&renderer)?;
    layer.set_content(&surface)?;
    window_ctx.add_layer(&layer, true)?;

    {
        let ctx = DrawContext::new(&surface, renderer.d2d_device(), None)?;
        ctx.clear(Color::rgba(0.0, 0.0, 0.0, 0.0));

        let count = 30;
        let wstep = width / count as f32;
        let hstep = height / count as f32;

        for i in 0..count {
            let i = i as f32;

            ctx.draw_line(
                Point::new(0.0, i * hstep),
                Point::new((i + 1.0) * wstep, height),
                Color::rgba(0.0, 1.0, 1.0, 1.0),
                2.0,
            )?;
            ctx.draw_line(
                Point::new(i * wstep, 0.0),
                Point::new(width, (i + 1.0) * hstep),
                Color::rgba(0.0, 0.0, 1.0, 1.0),
                2.0,
            )?;
        }
    }

    renderer.commit()?;

    app.run();
    Ok(())
}
