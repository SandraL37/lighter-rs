# Lighter

A barely working Ui framework. It uses D2D now but in future it will be
rewired to Vulkan or DirectX. You can use it but for now you can only draw text and rectangles!
It is reactive!

Coming soon...

# Current state of development

```rust
fn root() -> impl Element {
    let count = signal(0);
    let label = derived(move || format!("count: {}", count.get()));

    div()
        .size(percent(1.0))
        .bg(Color::WHITE)
        .align(AlignItems::Center)
        .justify(JustifyContent::Center)
        .child(text(label).font_size(48.0).on_click(move || {
            count.update(|c| {
                *c += 1;
            })
        }))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    app()?
        .add(window().title("simple counter!").root(root()))?
        .run()?;

    Ok(())
}
```

# How it works

## Runtime Ownership Contract

- NodeData owns interaction and visual state:
  - interaction_state
  - shared node style
  - element-specific style payload (DivStyle or TextStyle)
- NodeLayout owns layout style plus computed layout/cache.
- Deferred bindings are build-time only and must not be stored in runtime node state.
- Event handling may mutate interaction state, which then drives dirty flags and rendering.
