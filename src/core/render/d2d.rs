use std::{collections::HashMap, f32, sync::Arc};

/*
 * Main problems:
 * 1. too slow
 * 2. too pixelated for small font (Maybe antialiasing needed?)
 *
 *
 * TODO: need to handle when dpi changes
 */

use windows::{
    Win32::{
        Foundation::*,
        Graphics::{
            Direct2D::{Common::*, *},
            Direct3D::*,
            Direct3D11::*,
            DirectWrite::*,
            Dxgi::{Common::*, *},
        },
        UI::HiDpi::*,
    },
    core::{HSTRING, Interface},
};
use windows_numerics::Vector2;

use crate::{
    core::{
        error::*,
        layout::{AvailableSpace, Point, Rect, Size},
        render::{RenderCommand, Renderer, d2d::cache::D2DCache},
        style::Color,
    },
    elements::text::TextProps,
};

pub mod cache;
pub mod types;

pub use types::*;

pub struct D2DRendererFactory {
    d3d_device: D3DDevice,
    dxgi_device: DXGIDevice,
    d2d_factory: D2DFactory,
    d2d_device: D2DDevice,
    dwrite_factory: DWriteFactory,
}

impl From<windows_result::Error> for Error {
    #[inline(always)]
    fn from(value: windows_result::Error) -> Self {
        Error::D2DRendererError(value)
    }
}

impl From<Color> for D2D1_COLOR_F {
    #[inline(always)]
    fn from(value: Color) -> Self {
        Self {
            r: value.r,
            g: value.g,
            b: value.b,
            a: value.a,
        }
    }
}

impl From<Rect<f32>> for D2D_RECT_F {
    fn from(value: Rect<f32>) -> Self {
        Self {
            left: value.location.x,
            top: value.location.y,
            right: value.location.x + value.size.width,
            bottom: value.location.y + value.size.height,
        }
    }
}

impl From<Point<f32>> for Vector2 {
    fn from(value: Point<f32>) -> Self {
        Self {
            X: value.x,
            Y: value.y,
        }
    }
}

impl D2DRendererFactory {
    pub fn new() -> Result<Self> {
        // TODO: check if coinitializeex needed

        let d3d_device = Self::create_d3d_device()?;
        let dxgi_device = Self::create_dxgi_device(&d3d_device)?;
        let d2d_factory = Self::create_d2d_factory()?;
        let d2d_device = Self::create_d2d_device(&d2d_factory, &dxgi_device)?;
        let dwrite_factory = Self::create_dwrite_factory()?;

        Ok(Self {
            d3d_device,
            dxgi_device,
            d2d_factory,
            d2d_device,
            dwrite_factory,
        })
    }

    pub fn rebuild(&mut self) -> Result<()> {
        self.d3d_device = Self::create_d3d_device()?;
        self.dxgi_device = Self::create_dxgi_device(&self.d3d_device)?;
        self.d2d_device = Self::create_d2d_device(&self.d2d_factory, &self.dxgi_device)?;
        Ok(())
    }
}

impl D2DRendererFactory {
    pub fn create_renderer_for_hwnd(&self, hwnd: HWND, size: Size<usize>) -> Result<D2DRenderer> {
        let dpi = unsafe {
            let raw = GetDpiForWindow(hwnd);
            if raw == 0 { 96.0 } else { raw as f32 }
        };

        let swapchain = self.create_swapchain(hwnd, &size)?;
        let d2d_device_context = self.create_device_context()?;

        unsafe {
            d2d_device_context.SetTextAntialiasMode(D2D1_TEXT_ANTIALIAS_MODE_CLEARTYPE);
            d2d_device_context.SetDpi(dpi, dpi);
        }

        let bitmap = Self::create_target_bitmap(&d2d_device_context, &swapchain, dpi)?;

        Ok(D2DRenderer {
            cache: D2DCache::new(self.dwrite_factory.clone(), d2d_device_context.clone()),
            dwrite_factory: self.dwrite_factory.clone(),
            swapchain,
            d2d_device_context,
            target_bitmap: Some(bitmap),
            size,
            dpi,
        })
    }

    fn create_d3d_device() -> Result<D3DDevice> {
        let mut d3d_device = None;

        let feature_levels = [D3D_FEATURE_LEVEL_11_1, D3D_FEATURE_LEVEL_11_0];

        unsafe {
            D3D11CreateDevice(
                None,
                D3D_DRIVER_TYPE_HARDWARE,
                HMODULE::default(),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT | D3D11_CREATE_DEVICE_SINGLETHREADED,
                Some(&feature_levels),
                D3D11_SDK_VERSION,
                Some(&mut d3d_device),
                None,
                None,
            )?;
        }

        let Some(d3d_device) = d3d_device else {
            return Err(Error::GenericRendererError(
                "D3D11CreateDevice failed creating ID3D11Device".into(),
            ));
        };

        Ok(d3d_device.cast()?)
    }

    fn create_dxgi_device(d3d_device: &D3DDevice) -> Result<DXGIDevice> {
        let dxgi_device: DXGIDevice = d3d_device.cast()?;
        Ok(dxgi_device)
    }

    fn create_d2d_factory() -> Result<D2DFactory> {
        #[cfg(debug_assertions)]
        let options = D2D1_FACTORY_OPTIONS {
            debugLevel: windows::Win32::Graphics::Direct2D::D2D1_DEBUG_LEVEL_INFORMATION,
        };

        #[cfg(not(debug_assertions))]
        let options = D2D1_FACTORY_OPTIONS {
            debugLevel: windows::Win32::Graphics::Direct2D::D2D1_DEBUG_LEVEL_NONE,
        };

        let d2d_factory: D2DFactory =
            unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, Some(&options))? };

        Ok(d2d_factory)
    }

    fn create_d2d_device(d2d_factory: &D2DFactory, dxgi_device: &DXGIDevice) -> Result<D2DDevice> {
        let d2d_device = unsafe { d2d_factory.CreateDevice(dxgi_device)? };

        Ok(d2d_device)
    }

    fn create_dwrite_factory() -> Result<DWriteFactory> {
        let dwrite_factory = unsafe { DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)? };

        Ok(dwrite_factory)
    }

    fn create_device_context(&self) -> Result<D2DDeviceContext> {
        Ok(unsafe {
            self.d2d_device
                .CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)? // TODO: change to multithreading
        })
    }

    fn create_swapchain(&self, hwnd: HWND, size: &Size<usize>) -> Result<DXGISwapChain> {
        let adapter: DXGIAdapter = unsafe { self.dxgi_device.GetAdapter()?.cast()? };
        let factory: DXGIFactory = unsafe { adapter.GetParent()? };

        let desc = DXGI_SWAP_CHAIN_DESC1 {
            Width: size.width as u32,
            Height: size.height as u32,
            Format: DXGI_FORMAT_B8G8R8A8_UNORM,
            Stereo: FALSE,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            }, // TODO: Check if correct
            BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
            BufferCount: 2,
            Scaling: DXGI_SCALING_NONE,
            SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
            AlphaMode: DXGI_ALPHA_MODE_IGNORE, // TODO: if i set premultiplied it crashes why?
            Flags: 0,
        };

        let swapchain: DXGISwapChain = unsafe {
            factory
                .CreateSwapChainForHwnd(&self.d3d_device, hwnd, &desc, None, None)?
                .cast()?
        };

        Ok(swapchain)
    }

    fn create_target_bitmap(
        d2d_device_context: &D2DDeviceContext,
        swapchain: &DXGISwapChain,
        dpi: f32,
    ) -> Result<D2DBitmap> {
        let surface: DXGISurface = unsafe { swapchain.GetBuffer(0)? }; // TODO: check correctness

        let bitmap_properties = D2D1_BITMAP_PROPERTIES1 {
            pixelFormat: D2D1_PIXEL_FORMAT {
                format: DXGI_FORMAT_B8G8R8A8_UNORM,
                alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
            },
            dpiX: dpi,
            dpiY: dpi,
            bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET | D2D1_BITMAP_OPTIONS_CANNOT_DRAW, // TODO: Check correctness
            colorContext: std::mem::ManuallyDrop::new(None), // TODO: Check correctness
        };

        let bitmap = unsafe {
            d2d_device_context.CreateBitmapFromDxgiSurface(&surface, Some(&bitmap_properties))?
        };

        Ok(bitmap)
    }
}

// to be stored per window
pub struct D2DRenderer {
    dwrite_factory: DWriteFactory,

    swapchain: DXGISwapChain,
    d2d_device_context: D2DDeviceContext,
    target_bitmap: Option<D2DBitmap>,
    size: Size<usize>,
    dpi: f32,

    cache: D2DCache,
}

impl D2DRenderer {
    pub fn recreate_swapchain(&mut self, new_size: Size<usize>) -> Result<()> {
        unsafe {
            self.d2d_device_context.SetTarget(None);
        };

        self.target_bitmap = None;

        unsafe {
            self.swapchain.ResizeBuffers(
                0,
                new_size.width as u32,
                new_size.height as u32,
                DXGI_FORMAT_UNKNOWN,
                DXGI_SWAP_CHAIN_FLAG(0),
            )?;
        }

        self.target_bitmap = Some(D2DRendererFactory::create_target_bitmap(
            &self.d2d_device_context,
            &self.swapchain,
            self.dpi,
        )?);

        self.size = new_size;

        Ok(())
    }
}

impl Renderer for D2DRenderer {
    fn render(&mut self, commands: &[RenderCommand]) -> Result<()> {
        if let Some(target_bitmap) = &self.target_bitmap {
            unsafe {
                self.d2d_device_context.SetTarget(&*target_bitmap); // TODO: &* wtf
                self.d2d_device_context.BeginDraw();
                self.d2d_device_context
                    .Clear(Some(&Color::TRANSPARENT.into()));

                for command in commands {
                    match command {
                        RenderCommand::Rect {
                            bounds,
                            corner_radius,
                            color,
                            ..
                        } => {
                            let brush = self.cache.get_solid_color_brush(color)?;

                            let rect = D2D1_ROUNDED_RECT {
                                rect: (*bounds).into(),
                                radiusX: *corner_radius, // TODO: extend to xy
                                radiusY: *corner_radius,
                            };

                            self.d2d_device_context.FillRoundedRectangle(&rect, &brush);
                        }
                        RenderCommand::Text { bounds, props, .. } => {
                            // TODO: needs caching
                            let fmt = self.cache.get_text_format(props)?;
                            let utf16: Vec<u16> = props.content.encode_utf16().collect();
                            let layout = self.dwrite_factory.CreateTextLayout(
                                &utf16,
                                &fmt,
                                bounds.size.width,
                                bounds.size.height,
                            )?;

                            let brush = self.cache.get_solid_color_brush(&props.color)?;

                            self.d2d_device_context.DrawTextLayout(
                                bounds.location.into(),
                                &layout,
                                &brush,
                                None, // TODO: check correctness
                                0,    // TODO: check correctness
                                D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT,
                            );
                        }
                    }
                }

                let result = self.d2d_device_context.EndDraw(None, None);

                if let Err(e) = result
                    && e.code() == D2DERR_RECREATE_TARGET
                {
                    return Err(Error::DeviceLost);
                }

                // TODO: add vsync once everything is perfect
                let result = self.swapchain.Present(0, DXGI_PRESENT(0));

                if matches!(result, DXGI_ERROR_DEVICE_REMOVED | DXGI_ERROR_DEVICE_RESET) {
                    return Err(Error::DeviceLost);
                }
            }
        } else {
            return Err(Error::DeviceLost); // TODO: check correctness
        }

        Ok(())
    }

    fn resize(&mut self, size: Size<usize>) -> Result<()> {
        self.recreate_swapchain(size)?;

        Ok(())
    }

    fn measure_text(
        &mut self,
        text_props: &TextProps,
        available_size: Size<AvailableSpace>,
    ) -> Result<Size<f32>> {
        let max_width = match available_size.width {
            AvailableSpace::Definite(px) => px,
            _ => f32::MAX,
        };

        let max_height = match available_size.height {
            AvailableSpace::Definite(px) => px,
            _ => f32::MAX,
        };

        let fmt = self.cache.get_text_format(text_props)?;
        let utf16: Vec<u16> = text_props.content.encode_utf16().collect();
        let layout = unsafe {
            self.dwrite_factory
                .CreateTextLayout(&utf16, &fmt, max_width, max_height)?
        };

        let mut metrics = DWRITE_TEXT_METRICS::default();

        unsafe { layout.GetMetrics(&mut metrics)? };

        Ok(Size::wh(metrics.width, metrics.height))
    }
}

// TODO: To cache
// #[derive(Hash, Eq, PartialEq)]
// struct TextLayouutKey {
//     content_ptr: usize,
//     format_key: TextFormatKey,
//     max_width_bits: u32,
//     max_height_bits: u32,
// }
