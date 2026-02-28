# Lighter
A barely working Ui framework. It uses D2D now but in future it will be
rewired to Vulkan or DirectX. It is still in development and you can't use it
to build applications as of now :(

Coming soon...

# Current state of development

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let page = div()
        .size(percent(1.0))
        .bg(Color::WHITE)
        .align(AlignItems::Center)
        .justify(JustifyContent::Center)
        .child(
            text("Hello, Lighter!🎉")
                .font_size(64.0)
                .font_weight(FontWeight::BOLD)
                .color(Color::BLACK)
        );

    let mut app = Application::new()?;
    app.add_window(window().root(page))?;

    app.run();
    Ok(())
}
```
