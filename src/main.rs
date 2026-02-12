use lighter::prelude::*;

fn main() {
    let page = div()
        .align(AlignItems::Center)
        .justify(JustifyContent::Center)
        .child(text("hello world"));

    println!("{page:#?}");
}
