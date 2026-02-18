# Lighter
A barely working ~~UI Framework~~ (Frame generator). It uses tiny-skia now but in future it will be
rewired to Vulkan or DirectX.

# Current state of development

```rust
use lighter::prelude::*;

fn main() {
    let page = div()
        .align(AlignItems::Center)
        .justify(JustifyContent::Center)
        .child(text("hello world"));

    println!("{page:#?}");
    
    // + some dirty tracking
}
```
