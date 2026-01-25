use windows::{
    Win32::{
        Graphics::{
            Direct2D::{
                Common::{D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_PIXEL_FORMAT},
                D2D1_BITMAP_OPTIONS_CANNOT_DRAW, D2D1_BITMAP_OPTIONS_TARGET,
                D2D1_BITMAP_PROPERTIES1, D2D1_DEVICE_CONTEXT_OPTIONS_NONE, D2D1_FACTORY_OPTIONS,
                D2D1_FACTORY_TYPE_MULTI_THREADED, D2D1CreateFactory,
            },
            Direct3D::D3D_DRIVER_TYPE_HARDWARE,
            Direct3D11::{
                D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_CREATE_DEVICE_SINGLETHREADED,
                D3D11_SDK_VERSION, D3D11CreateDevice,
            },
            DirectComposition::DCompositionCreateDevice3,
        },
        System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize},
    },
    core::Interface,
};

use crate::{core::*, error::*, types::*};

pub struct WindowContext {
    _dcomp_target: dcomp::Target, // TODO: Check why not used
    root_visual: dcomp::Visual,
}

impl WindowContext {
    pub fn new(dcomp_device: &dcomp::Device, hwnd: foundation::Hwnd) -> Result<Self> {
        let dcomp_desktop_device: dcomp::DesktopDevice = dcomp_device.cast()?;
        let dcomp_target = unsafe { dcomp_desktop_device.CreateTargetForHwnd(hwnd, true)? };
        let root_visual = unsafe { dcomp_device.CreateVisual()?.cast()? };

        unsafe { dcomp_target.SetRoot(&root_visual)? };

        Ok(Self {
            _dcomp_target: dcomp_target,
            root_visual,
        })
    }

    pub fn root_visual(&self) -> &dcomp::Visual {
        &self.root_visual
    }

    pub fn set_root_content(&self, surface: &RenderSurface) -> Result<()> {
        unsafe { self.root_visual.SetContent(surface.surface())? };
        Ok(())
    }

    pub fn add_layer(&self, layer: &CompositionLayer, insert_above: bool) -> Result<()> {
        unsafe {
            self.root_visual
                .AddVisual(layer.visual(), insert_above, None)?
        };

        Ok(())
    }

    pub fn remove_layer(&self, layer: &CompositionLayer) -> Result<()> {
        unsafe {
            self.root_visual.RemoveVisual(layer.visual())?;
        };

        Ok(())
    }

    pub fn clear_layers(&self) -> Result<()> {
        unsafe {
            self.root_visual.RemoveAllVisuals()?;
        };

        Ok(())
    }
}

pub struct Renderer {
    _dxgi_device: dxgi::Device, // TODO: Check why not used

    d2d_factory: d2d::Factory,
    d2d_device: d2d::Device,

    dcomp_device: dcomp::Device,
}

impl Renderer {
    pub fn new() -> Result<Self> {
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED)
                .ok()
                .map_err(|e| EngineError::WindowsApiError(e))?;
        }

        let dxgi_device = Self::create_dxgi_device()?;
        let d2d_factory = Self::create_d2d_factory()?;
        let d2d_device = Self::create_d2d_device(&d2d_factory, &dxgi_device)?;
        let dcomp_device = Self::create_dcomp_device(&dxgi_device)?;

        Ok(Self {
            _dxgi_device: dxgi_device,
            d2d_factory,
            d2d_device,
            dcomp_device,
        })
    }

    fn create_dxgi_device() -> Result<dxgi::Device> {
        let mut d3d_device = None;

        // let feature_levels = [
        //     D3D_FEATURE_LEVEL_11_1,
        //     D3D_FEATURE_LEVEL_11_0,
        //     D3D_FEATURE_LEVEL_10_1,
        // ]; // TODO: check if needed

        unsafe {
            D3D11CreateDevice(
                None,
                D3D_DRIVER_TYPE_HARDWARE,
                foundation::Hmodule::default(),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT | D3D11_CREATE_DEVICE_SINGLETHREADED,
                None,
                D3D11_SDK_VERSION,
                Some(&mut d3d_device),
                None,
                None,
            )?;
        }

        let Some(d3d_device) = d3d_device else {
            return Err(EngineError::UnknownWindowsError(
                "D3D11CreateDevice failed creating ID3D11Device".into(),
            ));
        };

        Ok(d3d_device.cast()?)
    }

    fn create_d2d_factory() -> Result<d2d::Factory> {
        #[cfg(debug_assertions)]
        let options = D2D1_FACTORY_OPTIONS {
            debugLevel: windows::Win32::Graphics::Direct2D::D2D1_DEBUG_LEVEL_INFORMATION,
        };

        #[cfg(not(debug_assertions))]
        let options = D2D1_FACTORY_OPTIONS {
            debugLevel: windows::Win32::Graphics::Direct2D::D2D1_DEBUG_LEVEL_INFORMATION,
        };

        let d2d_factory: d2d::Factory =
            unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_MULTI_THREADED, Some(&options))? };

        Ok(d2d_factory)
    }

    fn create_d2d_device(
        d2d_factory: &d2d::Factory,
        dxgi_device: &dxgi::Device,
    ) -> Result<d2d::Device> {
        let d2d_device = unsafe { d2d_factory.CreateDevice(dxgi_device)? };

        Ok(d2d_device)
    }

    fn create_dcomp_device(dxgi_device: &dxgi::Device) -> Result<dcomp::Device> {
        let dcomp_device = unsafe { DCompositionCreateDevice3(dxgi_device)? };

        Ok(dcomp_device)
    }

    pub fn create_window_context(&self, hwnd: foundation::Hwnd) -> Result<WindowContext> {
        WindowContext::new(&self.dcomp_device, hwnd)
    }

    pub fn create_surface(
        &self,
        size: Size,
        pixel_format: dxgi::common::PixelFormat,
        alpha_mode: dxgi::common::AlphaMode,
    ) -> Result<RenderSurface> {
        RenderSurface::new(&self.dcomp_device, size, pixel_format, alpha_mode)
    }

    /// Create a device context for drawing
    ///
    /// Device contexts are lightweight and should be created per-frame
    pub fn create_d2d_device_context(&self) -> Result<d2d::DeviceContext> {
        let d2d_device_context = unsafe {
            self.d2d_device
                .CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)? // TODO: should be modified when the MULTITHREADING is enabled
        };

        Ok(d2d_device_context)
    }

    pub fn create_visual(&self) -> Result<dcomp::Visual> {
        let visual = unsafe { self.dcomp_device.CreateVisual()? };

        Ok(visual.cast()?)
    }

    /// Commit all pending composition commands
    ///
    /// This is when DirectComposition actually updates the screen
    /// Synchronizes with vsync for tear-free rendering
    ///
    /// CRITICAL: This commits ALL windows atomically
    /// All window updates appear on screen simultaneously
    pub fn commit(&self) -> Result<()> {
        unsafe {
            self.dcomp_device.Commit()?;
        }

        Ok(())
    }

    pub fn d2d_factory(&self) -> &d2d::Factory {
        &self.d2d_factory
    }

    pub fn dcomp_device(&self) -> &dcomp::Device {
        &self.dcomp_device
    }

    pub fn d2d_device(&self) -> &d2d::Device {
        &self.d2d_device
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}

pub struct RenderSurface {
    surface: dcomp::Surface,
    size: Size,
    pixel_format: dxgi::common::PixelFormat,
}

impl RenderSurface {
    pub fn new(
        dcomp_device: &dcomp::Device,
        size: Size,
        pixel_format: dxgi::common::PixelFormat,
        alpha_mode: dxgi::common::AlphaMode,
    ) -> Result<Self> {
        let surface = unsafe {
            dcomp_device.CreateVirtualSurface(
                size.width as u32,
                size.height as u32,
                pixel_format,
                alpha_mode,
            )?
        };

        Ok(Self {
            surface,
            size,
            pixel_format,
        })
    }

    pub fn begin_draw(&self, update_rect: Option<Rect>) -> Result<(dxgi::Surface, Point)> {
        let mut offset = foundation::Point::default();

        let raw_rect;
        let rect_ptr = if let Some(rect) = update_rect {
            raw_rect = rect.win32();
            Some(&raw_rect as *const _)
        } else {
            None
        };

        let dxgi_surface: dxgi::Surface = unsafe { self.surface.BeginDraw(rect_ptr, &mut offset)? };

        Ok((dxgi_surface, Point::from(offset)))
    }

    pub fn end_draw(&self) -> Result<()> {
        unsafe {
            self.surface.EndDraw()?;
        }

        Ok(())
    }

    pub fn size(&self) -> &Size {
        &self.size
    }

    pub fn surface(&self) -> &dcomp::Surface {
        &self.surface
    }
}

pub struct CompositionLayer {
    visual: dcomp::Visual,
}

impl CompositionLayer {
    pub fn new(renderer: &Renderer) -> Result<Self> {
        let visual = renderer.create_visual()?;
        Ok(Self { visual })
    }

    pub fn set_content(&self, surface: &RenderSurface) -> Result<()> {
        unsafe {
            self.visual.SetContent(surface.surface())?;
        }

        Ok(())
    }

    pub fn set_offset(&self, offset: Point) -> Result<()> {
        unsafe {
            self.visual.SetOffsetX2(offset.x)?;
            self.visual.SetOffsetY2(offset.y)?;
        }

        Ok(())
    }

    pub fn add_child(&self, child: &CompositionLayer, insert_above: bool) -> Result<()> {
        unsafe {
            self.visual.AddVisual(&child.visual, insert_above, None)?;
        }

        Ok(())
    }

    pub fn remove_child(&self, child: &CompositionLayer) -> Result<()> {
        unsafe {
            self.visual.RemoveVisual(&child.visual)?;
        }

        Ok(())
    }

    pub fn set_opacity(&self, opacity: f32) -> Result<()> {
        unsafe {
            self.visual.SetOpacity2(opacity)?;
        }

        Ok(())
    }

    pub fn visual(&self) -> &dcomp::Visual {
        &self.visual
    }
}

pub struct DrawContext {
    device_context: d2d::DeviceContext,
    offset: Point,
    surface: Option<dcomp::Surface>,

    _bitmap: d2d::Bitmap,
}

impl DrawContext {
    pub fn new(
        surface: &RenderSurface,
        d2d_device: &d2d::Device,
        update_rect: Option<Rect>,
    ) -> Result<Self> {
        let (dxgi_surface, offset) = surface.begin_draw(update_rect)?;

        let device_context =
            unsafe { d2d_device.CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)? }; // TODO: check

        let bitmap_properties = D2D1_BITMAP_PROPERTIES1 {
            pixelFormat: D2D1_PIXEL_FORMAT {
                format: surface.pixel_format,
                alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
            },
            bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET | D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
            ..Default::default()
        };

        let bitmap = unsafe {
            device_context.CreateBitmapFromDxgiSurface(&dxgi_surface, Some(&bitmap_properties))?
        };

        unsafe { device_context.SetTarget(&bitmap) };
        unsafe { device_context.BeginDraw() };

        Ok(Self {
            device_context,
            offset,
            surface: Some(surface.surface.clone()),
            _bitmap: bitmap,
        })
    }

    pub fn context(&self) -> &d2d::DeviceContext {
        &self.device_context
    }

    pub fn offset(&self) -> &Point {
        &self.offset
    }

    pub fn clear(&self, color: Color) {
        unsafe {
            self.device_context.Clear(Some(&color.into()));
        }
    }

    pub fn submit(&self, command: &Command) -> Result<()> {
        match command {
            Command::DrawLine(line) => line.draw(self.context()),
            Command::DrawFilledRectangle(rect) => rect.draw(self.context()),
        }
    }

    pub fn end(mut self) -> Result<()> {
        self.finish()?;

        Ok(())
    }

    fn finish(&mut self) -> Result<()> {
        if let Some(surface) = self.surface.take() {
            unsafe { self.device_context.EndDraw(None, None)? };
            unsafe { surface.EndDraw()? };
        }
        Ok(())
    }
}

impl Drop for DrawContext {
    fn drop(&mut self) {
        let _ = self.finish();
    }
}

pub enum Command {
    DrawLine(Line),
    DrawFilledRectangle(FilledRectangle),
}

pub struct Line {
    pub start: Point,
    pub end: Point,
    pub color: Color,
    pub stroke_width: f32,
}

impl Line {
    fn draw(&self, context: &d2d::DeviceContext) -> Result<()> {
        unsafe {
            let brush = context.CreateSolidColorBrush(&self.color.d2d(), None)?;

            context.DrawLine(
                self.start.into(),
                self.end.into(),
                &brush,
                self.stroke_width,
                None,
            );
        }

        Ok(())
    }
}

pub struct FilledRectangle {
    pub rect: Rect,
    pub color: Color,
}

impl FilledRectangle {
    fn draw(&self, context: &d2d::DeviceContext) -> Result<()> {
        unsafe {
            let brush = context.CreateSolidColorBrush(&self.color.d2d(), None)?;

            context.FillRectangle(&self.rect.d2d(), &brush);
        }

        Ok(())
    }
}
