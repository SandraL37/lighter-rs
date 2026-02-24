// use crate::core::{
//     error::*,
//     layout::{Rect, Size},
//     render::{RenderCommand, Renderer},
//     style::Color,
// };

// pub struct TinySkiaRenderer {
//     pixmap: tiny_skia::Pixmap,
// }

// impl TinySkiaRenderer {
//     pub fn new(width: u32, height: u32) -> Result<Self> {
//         let pixmap = tiny_skia::Pixmap::new(width, height);
//         match pixmap {
//             Some(pixmap) => Ok(TinySkiaRenderer { pixmap }),
//             None => Err(Error::TinySkiaRendererError(
//                 "Failed to create Pixmap".into(),
//             )),
//         }
//     }

//     pub fn pixmap(&self) -> &tiny_skia::Pixmap {
//         &self.pixmap
//     }

//     pub fn save_png(&self, path: &str) -> Result<()> {
//         self.pixmap
//             .save_png(path)
//             .map_err(|_| Error::TinySkiaRendererError("Failed to save PNG".into()))
//     }

//     fn create_pixmap(size: Size<u32>) -> Result<tiny_skia::Pixmap> {
//         tiny_skia::Pixmap::new(size.width, size.height).ok_or(Error::TinySkiaRendererError(
//             "Failed to create Pixmap".into(),
//         ))
//     }
// }

// impl From<Color> for tiny_skia::Color {
//     fn from(color: Color) -> Self {
//         tiny_skia::Color::from_rgba(color.r, color.g, color.b, color.a).unwrap()
//     }
// }

// impl From<Rect<f32>> for tiny_skia::Rect {
//     fn from(rect: Rect<f32>) -> Self {
//         tiny_skia::Rect::from_xywh(
//             rect.location.x,
//             rect.location.y,
//             rect.size.width,
//             rect.size.height,
//         )
//         .unwrap()
//     }
// }

// impl Renderer for TinySkiaRenderer {
//     fn render(&mut self, commands: &[RenderCommand]) -> Result<()> {
//         self.pixmap.fill(Color::TRANSPARENT.into());

//         for cmd in commands {
//             match cmd {
//                 RenderCommand::Rect {
//                     bounds,
//                     color,
//                     opacity,
//                     ..
//                 } => {
//                     let mut paint = tiny_skia::Paint::default();
//                     let color = Color {
//                         a: color.a * opacity, // TODO: This is an escape hatch and needs rework!
//                         ..*color
//                     };
//                     paint.set_color(color.into());

//                     self.pixmap.fill_rect(
//                         (*bounds).into(),
//                         &paint,
//                         tiny_skia::Transform::identity(),
//                         None,
//                     );
//                 }
//                 RenderCommand::Text {
//                     bounds,
//                     color,
//                     opacity,
//                     ..
//                 } => {
//                     let mut paint = tiny_skia::Paint::default();
//                     let color = Color {
//                         a: color.a * opacity, // TODO: This is an escape hatch and needs rework!
//                         ..*color
//                     };
//                     paint.set_color(color.into());

//                     self.pixmap.fill_rect(
//                         (*bounds).into(),
//                         &paint,
//                         tiny_skia::Transform::identity(),
//                         None,
//                     );
//                 }
//             }
//         }

//         Ok(())
//     }

//     fn resize(&mut self, size: Size<u32>) -> Result<()> {
//         self.pixmap = TinySkiaRenderer::create_pixmap(size)?;
//         Ok(())
//     }

//     fn get_size(&self) -> Size<u32> {
//         Size::wh(self.pixmap.width(), self.pixmap.height())
//     }
// }
