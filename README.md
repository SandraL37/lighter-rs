# Lighter
This is a lightweight UI framework focused on performance.
It is in development. So it isn't ready for production.

# Current state of development

```rust
use lighter::prelude::*;

fn main() {
    let page = div()
        .align(AlignItems::Center)
        .justify(JustifyContent::Center)
        .child(text("hello world"));

    println!("{page:#?}");
}
```
