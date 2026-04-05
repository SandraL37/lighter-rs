#[cfg(test)]
pub mod tests {
    use crate::{
        core::{
            error::*,
            layout::{AvailableSpace, types::size::Size},
            render::{Dpi, RenderCommand, Renderer},
        },
        elements::text::style::TextStyle,
    };

    pub struct TestRenderer {
        size: Size<usize>,
        dpi: Dpi,
        render_calls: usize,
    }

    impl TestRenderer {
        pub fn new(size: Size<usize>) -> Self {
            Self {
                size,
                dpi: Dpi::uniform(96.0),
                render_calls: 0,
            }
        }
    }

    impl Renderer for TestRenderer {
        fn render(&mut self, _commands: &[RenderCommand]) -> Result<()> {
            self.render_calls += 1;
            Ok(())
        }

        fn resize(&mut self, size: Size<usize>) -> Result<()> {
            self.size = size;
            Ok(())
        }

        fn measure_text(
            &mut self,
            _text_props: &TextStyle,
            _available_size: Size<AvailableSpace>,
        ) -> Result<Size<f32>> {
            Ok(Size::wh(0.0, 0.0))
        }

        fn set_dpi(&mut self, dpi: Dpi) -> Result<()> {
            self.dpi = dpi;
            Ok(())
        }
    }
}
