use piet_common::{
    BitmapTarget, Color as PietColor, Device, ImageFormat, RenderContext, Text, TextAttribute,
    TextLayoutBuilder,
    kurbo::{Affine, Rect as PietRect},
};

use crate::core::{
    error::*,
    layout::{Rect, Size},
    style::{Color, Transform},
};

use super::{RenderCommand, Renderer};

/// A software renderer backed by piet-common's bitmap target.
///
/// # Self-referential struct — safety contract
///
/// `BitmapTarget<'device>` holds a reference into the `Device` that created it.
/// We store both in the same struct, which Rust does not support natively.
///
/// Our solution:
///   1. `device` is heap-allocated behind a `Box` and **never moved** after
///      `bitmap` is created.  `Box<T>` guarantees a stable heap address.
///   2. We lie to the compiler about the bitmap's lifetime, using `'static`.
///      The real invariant is upheld manually: **`bitmap` is always dropped
///      before `device`**, which Rust guarantees because struct fields are
///      dropped in declaration order (top to bottom).
///   3. No code path moves `device` while `bitmap` is live.  The only
///      mutation is `resize()`, which drops the old bitmap first (by
///      overwriting `self.bitmap`), then never touches `device`.
///
/// This is the same pattern used by `ouroboros` / `self_cell` internally,
/// made explicit here to avoid the extra dependency.
pub struct PietRenderer {
    // IMPORTANT: declaration order matters for drop order.
    // `bitmap` must be declared BEFORE `device` so it is dropped first.
    //
    // Rust drops struct fields in declaration order (first → last).
    // If device were declared first, it would drop first, leaving bitmap
    // with a dangling reference during its own drop — use-after-free.
    bitmap: BitmapTarget<'static>, // 'static is a lie; see safety contract above
    device: Box<Device>,           // never moved after bitmap is created
    size: Size<u32>,
}

impl PietRenderer {
    pub fn new(size: Size<u32>) -> Result<Self> {
        // Heap-allocate the Device so it has a stable address.
        let mut device = Box::new(
            Device::new().map_err(|_| Error::RendererError("Failed to create device".into()))?,
        );

        // Create the bitmap, borrowing from the heap-allocated device.
        // SAFETY: We extend the lifetime to 'static here. The actual lifetime
        // is "as long as `device` lives and doesn't move", which we guarantee
        // by (a) storing device in a Box on the heap, (b) never moving it,
        // and (c) ensuring bitmap is dropped before device via field order.
        let bitmap: BitmapTarget<'static> = unsafe {
            let device_ref: &'static mut Device = &mut *(device.as_mut() as *mut Device);
            device_ref
                .bitmap_target(size.width as usize, size.height as usize, 1.0)
                .map_err(|_| Error::RendererError("Failed to create bitmap target".into()))?
        };

        Ok(Self {
            bitmap,
            device,
            size,
        })
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    /// Convert a `Color` (0–255 channels, 0.0–1.0 alpha) to piet's RGBA.
    ///
    /// Piet expects all channels in 0.0–1.0. We divide r/g/b by 255.
    fn color_with_opacity(color: Color, opacity: f32) -> PietColor {
        PietColor::rgba(
            color.r as f64,
            color.g as f64,
            color.b as f64,
            (color.a * opacity) as f64,
        )
    }

    fn rect_to_piet(rect: Rect<f32>) -> PietRect {
        PietRect::new(
            rect.location.x as f64,
            rect.location.y as f64,
            (rect.location.x + rect.size.width) as f64,
            (rect.location.y + rect.size.height) as f64,
        )
    }

    /// Convert a 4×4 column-major `Transform` matrix to piet's 2D `Affine`.
    ///
    /// We extract the top-left 2×2 rotation/scale block and the translation
    /// column.  The Z and W rows/columns are ignored (this is a 2D renderer).
    ///
    /// Matrix layout assumed (column-major, matching typical GPU convention):
    /// ```text
    /// [ m[0][0]  m[1][0]  0  m[3][0] ]   [ a  c  0  tx ]
    /// [ m[0][1]  m[1][1]  0  m[3][1] ] = [ b  d  0  ty ]
    /// [    0        0     1     0    ]   [ 0  0  1   0 ]
    /// [    0        0     0     1    ]   [ 0  0  0   1 ]
    /// ```
    fn transform_to_affine(transform: &Transform) -> Affine {
        let m = &transform.matrix;
        Affine::new([
            m[0][0] as f64, // a — x scale / cos(θ)
            m[0][1] as f64, // b — y shear / sin(θ)
            m[1][0] as f64, // c — x shear / -sin(θ)
            m[1][1] as f64, // d — y scale / cos(θ)
            m[3][0] as f64, // tx — x translation
            m[3][1] as f64, // ty — y translation
        ])
    }

    /// Save the current bitmap to a PNG file.
    pub fn save_png(&mut self, path: &str) -> Result<()> {
        let image_buf = self
            .bitmap
            .to_image_buf(ImageFormat::RgbaPremul)
            .map_err(|e| {
                Error::RendererError(format!("Failed to get image buffer for '{}': {}", path, e))
            })?;

        let width = self.size.width;
        let height = self.size.height;
        let raw = image_buf.raw_pixels();

        image::save_buffer(path, raw, width, height, image::ColorType::Rgba8)
            .map_err(|e| Error::RendererError(format!("Failed to save PNG '{}': {}", path, e)))
    }
}

impl Renderer for PietRenderer {
    fn render(&mut self, commands: &[RenderCommand]) -> Result<()> {
        let mut ctx = self.bitmap.render_context();

        // Clear to transparent before drawing.
        ctx.clear(
            Some(PietRect::new(
                0.0,
                0.0,
                self.size.width as f64,
                self.size.height as f64,
            )),
            PietColor::TRANSPARENT,
        );

        for command in commands {
            match command {
                RenderCommand::Rect {
                    bounds,
                    color,
                    opacity,
                    transform,
                    ..
                } => {
                    ctx.save()
                        .map_err(|_| Error::RendererError("ctx.save() failed".into()))?;

                    ctx.transform(Self::transform_to_affine(transform));
                    ctx.fill(
                        Self::rect_to_piet(*bounds),
                        &Self::color_with_opacity(*color, *opacity),
                    );

                    ctx.restore()
                        .map_err(|_| Error::RendererError("ctx.restore() failed".into()))?;
                }

                RenderCommand::Text {
                    bounds,
                    content,
                    color,
                    font_size,
                    opacity,
                    transform,
                    ..
                } => {
                    ctx.save()
                        .map_err(|_| Error::RendererError("ctx.save() failed".into()))?;

                    ctx.transform(Self::transform_to_affine(transform));

                    let layout = ctx
                        .text()
                        .new_text_layout(content.to_string())
                        .default_attribute(TextAttribute::FontSize(*font_size as f64))
                        .text_color(Self::color_with_opacity(*color, *opacity))
                        .build()
                        .map_err(|_| Error::RendererError("Text layout failed".into()))?;

                    ctx.draw_text(
                        &layout,
                        (bounds.location.x as f64, bounds.location.y as f64),
                    );

                    ctx.restore()
                        .map_err(|_| Error::RendererError("ctx.restore() failed".into()))?;
                }
            }
        }

        ctx.finish()
            .map_err(|_| Error::RendererError("ctx.finish() failed".into()))?;

        Ok(())
    }

    fn resize(&mut self, size: Size<u32>) -> Result<()> {
        // Create the new bitmap first. If this fails, self is unchanged.
        // SAFETY: same contract as new(). device is still heap-allocated and
        // not moved. The old bitmap (self.bitmap) is dropped by the assignment
        // below BEFORE device could ever be affected.
        let new_bitmap: BitmapTarget<'static> = unsafe {
            let device_ref: &'static mut Device = &mut *(self.device.as_mut() as *mut Device);
            device_ref
                .bitmap_target(size.width as usize, size.height as usize, 1.0)
                .map_err(|_| {
                    Error::RendererError("Resize: failed to create bitmap target".into())
                })?
        };

        // Drop old bitmap here, install new one.
        self.bitmap = new_bitmap;
        self.size = size;

        Ok(())
    }

    fn get_size(&self) -> Size<u32> {
        self.size
    }
}
