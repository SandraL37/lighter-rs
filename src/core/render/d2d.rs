// use std::f32;

// use windows::{
//     Win32::{
//         Foundation::*,
//         Graphics::{
//             Direct2D::{Common::*, *},
//             Direct3D::*,
//             Direct3D11::*,
//             DirectComposition::*,
//             DirectWrite::*,
//             Dxgi::{Common::*, *},
//         },
//         System::Threading::*,
//         UI::HiDpi::*,
//     },
//     core::{IUnknown, Interface},
// };
// use windows_numerics::{Matrix3x2, Vector2};

// use crate::{
//     core::{
//         error::*,
//         layout::{
//             AvailableSpace,
//             types::{point::Point, rect::Rect, size::Size},
//         },
//         render::{RenderCommand, Renderer, d2d::cache::D2DCache},
//         style::Color,
//     },
//     elements::text::TextProps,
// };

// pub mod cache;
// pub mod types;
// pub use types::*;

// // ── Additional type aliases (add these to types.rs) ──────────────────────────
// //
// //   pub type DCompDevice = IDCompositionDesktopDevice;
// //   pub type DCompTarget = IDCompositionTarget;
// //   pub type DCompVisual = IDCompositionVisual;
// //   pub type DXGISwapChain = IDXGISwapChain2;   // upgrade from IDXGISwapChain1
// //
// // IDCompositionDesktopDevice inherits IDCompositionDevice2, which adds
// // WaitForCommitCompletion and async-batch commits. It also adds
// // CreateTargetForHwnd, which IDCompositionDevice lacks.
// //
// // IDXGISwapChain2 adds GetFrameLatencyWaitableObject / SetMaximumFrameLatency,
// // the core of the low-latency present path.

// // ── Conversions ──────────────────────────────────────────────────────────────

// impl From<windows_result::Error> for Error {
//     #[inline(always)]
//     fn from(value: windows_result::Error) -> Self {
//         Error::D2DRendererError(value)
//     }
// }

// impl From<Color> for D2D1_COLOR_F {
//     #[inline(always)]
//     fn from(value: Color) -> Self {
//         Self {
//             r: value.r,
//             g: value.g,
//             b: value.b,
//             a: value.a,
//         }
//     }
// }

// impl From<Rect<f32>> for D2D_RECT_F {
//     fn from(value: Rect<f32>) -> Self {
//         Self {
//             left: value.location.x,
//             top: value.location.y,
//             right: value.location.x + value.size.width,
//             bottom: value.location.y + value.size.height,
//         }
//     }
// }

// impl From<Point<f32>> for Vector2 {
//     fn from(value: Point<f32>) -> Self {
//         Self {
//             X: value.x,
//             Y: value.y,
//         }
//     }
// }

// // ── Factory ───────────────────────────────────────────────────────────────────
// //
// // One factory per process. All per-window renderers share the D3D/D2D/DComp
// // devices housed here.

// pub struct D2DRendererFactory {
//     d3d_device: D3DDevice,
//     dxgi_device: DXGIDevice,
//     d2d_factory: D2DFactory,
//     d2d_device: D2DDevice,
//     dwrite_factory: DWriteFactory,
//     // IDCompositionDesktopDevice backs the composition tree for every HWND
//     // created by this factory. Shared across windows on the same GPU timeline.
//     dcomp_device: IDCompositionDesktopDevice,
// }

// impl D2DRendererFactory {
//     pub fn new() -> Result<Self> {
//         // NOTE: If your app uses COM elsewhere, call CoInitializeEx before this.

//         let d3d_device = Self::create_d3d_device()?;
//         let dxgi_device = Self::create_dxgi_device(&d3d_device)?;
//         let d2d_factory = Self::create_d2d_factory()?;
//         let d2d_device = Self::create_d2d_device(&d2d_factory, &dxgi_device)?;
//         let dwrite_factory = Self::create_dwrite_factory()?;
//         let dcomp_device = Self::create_dcomp_device(&dxgi_device)?;

//         unsafe {
//             SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)?;
//         }

//         Ok(Self {
//             d3d_device,
//             dxgi_device,
//             d2d_factory,
//             d2d_device,
//             dwrite_factory,
//             dcomp_device,
//         })
//     }

//     /// Recreate GPU resources after `Error::DeviceLost`.
//     ///
//     /// The D2D factory and DWrite factory survive device loss and must NOT be
//     /// recreated — they are CPU-side objects. Only the D3D/DXGI/D2D/DComp
//     /// device objects need rebuilding.
//     pub fn rebuild(&mut self) -> Result<()> {
//         self.d3d_device = Self::create_d3d_device()?;
//         self.dxgi_device = Self::create_dxgi_device(&self.d3d_device)?;
//         self.d2d_device = Self::create_d2d_device(&self.d2d_factory, &self.dxgi_device)?;
//         // DComp device is backed by the DXGI device — must be recreated with it.
//         self.dcomp_device = Self::create_dcomp_device(&self.dxgi_device)?;
//         Ok(())
//     }
// }

// impl D2DRendererFactory {
//     /// Create a renderer for the given HWND.
//     ///
//     /// # Window requirements
//     ///
//     /// For zero-copy DComp compositing the HWND should be created with the
//     /// `WS_EX_NOREDIRECTIONBITMAP` extended style. This tells DWM not to
//     /// allocate its own redirection surface for the window, saving a full-frame
//     /// GPU blit on every present. Without it DComp still works but DWM keeps an
//     /// unused shadow copy of the framebuffer.
//     pub fn create_renderer_for_hwnd(&self, hwnd: HWND, size: Size<usize>) -> Result<D2DRenderer> {
//         // Physical DPI for this monitor.
//         let dpi = unsafe {
//             let raw = GetDpiForWindow(hwnd);
//             if raw == 0 { 96.0 } else { raw as f32 }
//         };

//         // The swapchain is sized in physical pixels (as DXGI expects).
//         // D2D's coordinate system is DIPs. SetDpi maps 1 DIP → dpi/96 px.
//         // Our layout engine emits coordinates in physical pixels, so we apply
//         // an inverse scale (96/dpi) to cancel D2D's upscale. When you migrate
//         // the layout to DIP-native coordinates, remove this transform and the
//         // SetDpi call becomes sufficient on its own.
//         let scale = 96.0 / dpi;

//         let (swapchain, frame_latency_waitable) = self.create_swapchain(&size)?;
//         let d2d_device_context = self.create_device_context()?;

//         unsafe {
//             // ClearType sub-pixel rendering for crisp text at all DPI values.
//             d2d_device_context.SetTextAntialiasMode(D2D1_TEXT_ANTIALIAS_MODE_CLEARTYPE);
//             d2d_device_context.SetDpi(dpi, dpi);
//             d2d_device_context.SetTransform(&Matrix3x2::scale(scale, scale));
//         }

//         let target_bitmap = Self::create_target_bitmap(&d2d_device_context, &swapchain, dpi)?;

//         let (dcomp_target, dcomp_visual) =
//             Self::commit_dcomp_tree(&self.dcomp_device, hwnd, &swapchain)?;

//         Ok(D2DRenderer {
//             cache: D2DCache::new(self.dwrite_factory.clone(), d2d_device_context.clone()),
//             dwrite_factory: self.dwrite_factory.clone(),
//             // Drop order: visual → target → device (child before parent).
//             _dcomp_visual: dcomp_visual,
//             _dcomp_target: dcomp_target,
//             _dcomp_device: self.dcomp_device.clone(),
//             swapchain,
//             frame_latency_waitable,
//             d2d_device_context,
//             target_bitmap: Some(target_bitmap),
//             size,
//             dpi,
//         })
//     }

//     // ── Device-creation helpers ───────────────────────────────────────────────

//     fn create_d3d_device() -> Result<D3DDevice> {
//         let mut d3d_device = None;
//         let feature_levels = [D3D_FEATURE_LEVEL_11_1, D3D_FEATURE_LEVEL_11_0];

//         unsafe {
//             #[cfg(debug_assertions)]
//             D3D11CreateDevice(
//                 None,
//                 D3D_DRIVER_TYPE_HARDWARE,
//                 HMODULE::default(),
//                 D3D11_CREATE_DEVICE_BGRA_SUPPORT
//                     | D3D11_CREATE_DEVICE_SINGLETHREADED
//                     | D3D11_CREATE_DEVICE_DEBUG,
//                 Some(&feature_levels),
//                 D3D11_SDK_VERSION,
//                 Some(&mut d3d_device),
//                 None,
//                 None,
//             )?;
//             #[cfg(not(debug_assertions))]
//             D3D11CreateDevice(
//                 None,
//                 D3D_DRIVER_TYPE_HARDWARE,
//                 HMODULE::default(),
//                 D3D11_CREATE_DEVICE_BGRA_SUPPORT | D3D11_CREATE_DEVICE_SINGLETHREADED,
//                 Some(&feature_levels),
//                 D3D11_SDK_VERSION,
//                 Some(&mut d3d_device),
//                 None,
//                 None,
//             )?;
//         }

//         d3d_device
//             .ok_or_else(|| {
//                 Error::GenericRendererError("D3D11CreateDevice failed creating ID3D11Device".into())
//             })?
//             .cast()
//             .map_err(Into::into)
//     }

//     fn create_dxgi_device(d3d_device: &D3DDevice) -> Result<DXGIDevice> {
//         Ok(d3d_device.cast()?)
//     }

//     fn create_d2d_factory() -> Result<D2DFactory> {
//         #[cfg(debug_assertions)]
//         let options = D2D1_FACTORY_OPTIONS {
//             debugLevel: D2D1_DEBUG_LEVEL_INFORMATION,
//         };
//         #[cfg(not(debug_assertions))]
//         let options = D2D1_FACTORY_OPTIONS {
//             debugLevel: D2D1_DEBUG_LEVEL_NONE,
//         };

//         Ok(unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, Some(&options))? })
//     }

//     fn create_d2d_device(d2d_factory: &D2DFactory, dxgi_device: &DXGIDevice) -> Result<D2DDevice> {
//         Ok(unsafe { d2d_factory.CreateDevice(dxgi_device)? })
//     }

//     fn create_dwrite_factory() -> Result<DWriteFactory> {
//         Ok(unsafe { DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)? })
//     }

//     fn create_device_context(&self) -> Result<D2DDeviceContext> {
//         Ok(unsafe {
//             self.d2d_device
//                 .CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)?
//         })
//     }

//     fn create_dcomp_device(dxgi_device: &DXGIDevice) -> Result<IDCompositionDesktopDevice> {
//         // Share the GPU timeline with our D3D/DXGI stack so DComp compositing
//         // is zero-copy — no readback or intermediate surface is allocated.
//         //
//         // DCompositionCreateDevice2 is generic over its output interface.
//         // Requesting IDCompositionDesktopDevice gives us:
//         //   • IDCompositionDevice  — CreateVisual, CreateSurface, Commit …
//         //   • IDCompositionDevice2 — WaitForCommitCompletion, batch commits
//         //   • IDCompositionDesktopDevice — CreateTargetForHwnd (needed for HWND binding)
//         let unk: IUnknown = dxgi_device.cast()?;
//         Ok(unsafe { DCompositionCreateDevice2(Some(&unk))? })
//     }

//     /// Returns `(swapchain, frame_latency_waitable_handle)`.
//     ///
//     /// ## Why `CreateSwapChainForComposition`?
//     /// `ForHwnd` hands the present path to DWM's legacy blit model.
//     /// `ForComposition` routes every frame through the DComp visual tree,
//     /// enabling tear-free atomic commits, per-visual transforms, and
//     /// `WS_EX_NOREDIRECTIONBITMAP` support.
//     ///
//     /// ## Why `FRAME_LATENCY_WAITABLE_OBJECT`?
//     /// Without it, `Present(1, 0)` blocks on the *previous* frame completing,
//     /// meaning the CPU stalls inside the present call and can't process input.
//     /// With the waitable, you instead call `wait_for_present_ready()` at the
//     /// *top* of your frame loop so the CPU is idle and OS input is coalesced
//     /// while waiting — then you immediately render + present without blocking.
//     fn create_swapchain(&self, size: &Size<usize>) -> Result<(IDXGISwapChain2, HANDLE)> {
//         let adapter: DXGIAdapter = unsafe { self.dxgi_device.GetAdapter()?.cast()? };
//         let factory: DXGIFactory = unsafe { adapter.GetParent()? };

//         let desc = DXGI_SWAP_CHAIN_DESC1 {
//             Width: size.width as u32,
//             Height: size.height as u32,
//             Format: DXGI_FORMAT_B8G8R8A8_UNORM,
//             Stereo: FALSE,
//             SampleDesc: DXGI_SAMPLE_DESC {
//                 Count: 1,
//                 Quality: 0,
//             },
//             BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
//             BufferCount: 2,
//             // DXGI_SCALING_STRETCH is mandatory for ForComposition.
//             // DXGI_SCALING_NONE is only valid for ForHwnd on feature-level 11.1+.
//             Scaling: DXGI_SCALING_STRETCH,
//             SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
//             // PREMULTIPLIED lets the DComp visual blend against the desktop and
//             // other visuals with correct per-pixel alpha — required for
//             // transparent windows with WS_EX_NOREDIRECTIONBITMAP.
//             AlphaMode: DXGI_ALPHA_MODE_PREMULTIPLIED,
//             // FRAME_LATENCY_WAITABLE_OBJECT: enables GetFrameLatencyWaitableObject.
//             Flags: DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT.0 as u32,
//         };

//         // No HWND — DComp owns the connection to the window.
//         let swapchain: IDXGISwapChain2 = unsafe {
//             factory
//                 .CreateSwapChainForComposition(&self.d3d_device, &desc, None)?
//                 .cast()?
//         };

//         unsafe {
//             // Default maximum frame latency is 3. Setting it to 1 ensures the
//             // GPU queue is never more than one frame deep, trading throughput
//             // for the lowest possible end-to-end input latency.
//             swapchain.SetMaximumFrameLatency(1)?;
//         }

//         // This Win32 handle is owned by us; close it in Drop.
//         let waitable = unsafe { swapchain.GetFrameLatencyWaitableObject() };
//         if waitable.is_invalid() {
//             return Err(Error::GenericRendererError(
//                 "GetFrameLatencyWaitableObject returned an invalid handle".into(),
//             ));
//         }

//         Ok((swapchain, waitable))
//     }

//     /// Build the DComp visual tree for one HWND and issue the first Commit.
//     ///
//     /// The tree is intentionally minimal: one target → one visual → swapchain.
//     /// Add child visuals here later if you need layered sub-surfaces (e.g. for
//     /// video overlays or independent-flip panels).
//     fn commit_dcomp_tree(
//         device: &IDCompositionDesktopDevice,
//         hwnd: HWND,
//         swapchain: &IDXGISwapChain2,
//     ) -> Result<(IDCompositionTarget, IDCompositionVisual)> {
//         unsafe {
//             // `topmost = TRUE`: our visual is the root of this window's
//             // composition tree, rendered below any system chrome (title bar etc).
//             let target = device.CreateTargetForHwnd(hwnd, true)?;
//             let visual: IDCompositionVisual = device.CreateVisual()?.cast()?;

//             // Bind the swapchain as the visual's pixel source. IDXGISwapChain2
//             // implements IUnknown, which SetContent accepts.
//             visual.SetContent(swapchain)?;

//             // Promote the visual to root of the composition tree.
//             target.SetRoot(&visual)?;

//             // Atomically flush the visual tree to the compositor. After this,
//             // subsequent frames are driven solely by Present() — no further
//             // Commit() calls are needed per frame.
//             device.Commit()?;

//             Ok((target, visual))
//         }
//     }

//     fn create_target_bitmap(
//         ctx: &D2DDeviceContext,
//         swapchain: &IDXGISwapChain2,
//         dpi: f32,
//     ) -> Result<D2DBitmap> {
//         // GetBuffer(0) wraps the front buffer of the swapchain as a DXGI surface.
//         let surface: DXGISurface = unsafe { swapchain.GetBuffer(0)? };

//         let props = D2D1_BITMAP_PROPERTIES1 {
//             pixelFormat: D2D1_PIXEL_FORMAT {
//                 format: DXGI_FORMAT_B8G8R8A8_UNORM,
//                 // Must match the swapchain's AlphaMode so D2D doesn't insert a
//                 // conversion pass when writing to the render target.
//                 alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
//             },
//             dpiX: dpi,
//             dpiY: dpi,
//             // TARGET: writable render target.
//             // CANNOT_DRAW: prevents using this bitmap as a brush source, which
//             // would require an extra copy; we only ever draw *to* it.
//             bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET | D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
//             colorContext: std::mem::ManuallyDrop::new(None),
//         };

//         Ok(unsafe { ctx.CreateBitmapFromDxgiSurface(&surface, Some(&props))? })
//     }
// }

// // ── Per-window renderer ───────────────────────────────────────────────────────

// pub struct D2DRenderer {
//     dwrite_factory: DWriteFactory,

//     // ── DComp objects ─────────────────────────────────────────────────────────
//     // Field declaration order == drop order (last field drops first).
//     // Visual must drop before Target (it references the target's tree).
//     // Target must drop before Device (it holds a reference into the device).
//     // TODO: MUST CHECK
//     _dcomp_visual: IDCompositionVisual,
//     _dcomp_target: IDCompositionTarget,
//     _dcomp_device: IDCompositionDesktopDevice,

//     // ── DXGI / D2D ────────────────────────────────────────────────────────────
//     swapchain: IDXGISwapChain2,
//     /// Win32 HANDLE; closed on drop. Invalid if the device was lost.
//     frame_latency_waitable: HANDLE,
//     d2d_device_context: D2DDeviceContext,
//     /// `None` only during `recreate_swapchain` or after a device-lost error.
//     target_bitmap: Option<D2DBitmap>,
//     size: Size<usize>,
//     dpi: f32,

//     cache: D2DCache,
// }

// impl Drop for D2DRenderer {
//     fn drop(&mut self) {
//         // Detach D2D from the DXGI surface first so there are no live
//         // references when the swapchain (and its surfaces) are released.
//         unsafe { self.d2d_device_context.SetTarget(None) };
//         self.target_bitmap = None;

//         if !self.frame_latency_waitable.is_invalid() {
//             // Safety: we own this handle; nobody else holds a copy.
//             unsafe {
//                 let _ = CloseHandle(self.frame_latency_waitable);
//             }
//         }
//         // DComp COM objects drop in field-declaration order after this.
//     }
// }

// impl D2DRenderer {
//     /// Block until the DWM compositor is ready to accept the next frame.
//     ///
//     /// Call this **before** `render()` in your frame loop. While waiting the
//     /// thread is suspended at the OS level, giving the scheduler a chance to
//     /// process input events accumulated during the previous frame — this is the
//     /// source of the perceived "snappiness" vs. the legacy Present(1, 0) model.
//     ///
//     /// ```rust
//     /// loop {
//     ///     renderer.wait_for_present_ready();   // yields CPU; coalesces input
//     ///     let commands = app.build_frame();
//     ///     renderer.render(&commands)?;
//     /// }
//     /// ```
//     ///
//     /// The 1 000 ms timeout is a safety net. `WAIT_TIMEOUT` at this scale
//     /// indicates a hung compositor or a lost device; you should treat it the
//     /// same as `Error::DeviceLost` and rebuild.
//     #[inline]
//     pub fn wait_for_present_ready(&self) {
//         if !self.frame_latency_waitable.is_invalid() {
//             unsafe {
//                 // `bAlertable = FALSE`: we don't need APC delivery here.
//                 WaitForSingleObjectEx(self.frame_latency_waitable, 1000, false);
//             }
//         }
//     }

//     pub fn recreate_swapchain(&mut self, new_size: Size<usize>) -> Result<()> {
//         // Must clear the D2D render target before ResizeBuffers, otherwise
//         // DXGI will return DXGI_ERROR_INVALID_CALL (live references to the surface).
//         unsafe { self.d2d_device_context.SetTarget(None) };
//         self.target_bitmap = None;

//         unsafe {
//             self.swapchain.ResizeBuffers(
//                 0, // 0 = preserve buffer count
//                 new_size.width as u32,
//                 new_size.height as u32,
//                 DXGI_FORMAT_UNKNOWN, // preserve format
//                 // Preserve the waitable flag — dropping it here would invalidate
//                 // `frame_latency_waitable` and cause undefined behaviour on the
//                 // next wait.
//                 DXGI_SWAP_CHAIN_FLAG(DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT.0),
//             )?;
//         }

//         // ResizeBuffers preserves the IDXGISwapChain2 object identity, so:
//         //   • The DComp visual's SetContent binding remains valid — no rebind.
//         //   • The frame-latency waitable handle remains valid — no re-fetch.
//         //   • MaximumFrameLatency is preserved — no re-set.
//         // A new Commit() is therefore not required.

//         self.target_bitmap = Some(D2DRendererFactory::create_target_bitmap(
//             &self.d2d_device_context,
//             &self.swapchain,
//             self.dpi,
//         )?);

//         self.size = new_size;
//         Ok(())
//     }
// }

// impl Renderer for D2DRenderer {
//     fn render(&mut self, commands: &[RenderCommand]) -> Result<()> {
//         let Some(target_bitmap) = &self.target_bitmap else {
//             // target_bitmap is None only during recreate_swapchain or after
//             // device loss; the caller should have rebuilt before calling render.
//             return Err(Error::DeviceLost);
//         };

//         unsafe {
//             self.d2d_device_context.SetTarget(&*target_bitmap);
//             self.d2d_device_context.BeginDraw();
//             self.d2d_device_context
//                 .Clear(Some(&Color::TRANSPARENT.into()));

//             for command in commands {
//                 match command {
//                     RenderCommand::Rect {
//                         bounds,
//                         corner_radius,
//                         color,
//                         ..
//                     } => {
//                         let brush = self.cache.get_solid_color_brush(color)?;
//                         let rect = D2D1_ROUNDED_RECT {
//                             rect: (*bounds).into(),
//                             radiusX: *corner_radius,
//                             radiusY: *corner_radius,
//                         };
//                         self.d2d_device_context.FillRoundedRectangle(&rect, &brush);
//                     }
//                     RenderCommand::Text { bounds, props, .. } => {
//                         let layout = self.cache.get_text_layout(props, bounds.size)?;
//                         let brush = self.cache.get_solid_color_brush(&props.color)?;
//                         self.d2d_device_context.DrawTextLayout(
//                             bounds.location.into(),
//                             &layout,
//                             &brush,
//                             None,
//                             0,
//                             D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT,
//                         );
//                     }
//                 }
//             }

//             let result = self.d2d_device_context.EndDraw(None, None);
//             if let Err(e) = result {
//                 if e.code() == D2DERR_RECREATE_TARGET {
//                     return Err(Error::DeviceLost);
//                 }
//                 return Err(e.into());
//             }

//             // DXGI_PRESENT_DO_NOT_WAIT: non-blocking present. The frame-latency
//             // waitable already guaranteed the compositor is ready, so this call
//             // returns immediately (no internal stall). Using Present(1, 0) here
//             // would re-introduce the latency we eliminated with the waitable.
//             //
//             // SyncInterval = 0: vsync pacing is handled by the DComp compositor
//             // using the waitable object. Passing 1 here would add an extra
//             // frame of queuing on top of the waitable, defeating its purpose.
//             let result = self.swapchain.Present(0, DXGI_PRESENT_DO_NOT_WAIT);

//             if matches!(result, DXGI_ERROR_DEVICE_REMOVED | DXGI_ERROR_DEVICE_RESET) {
//                 return Err(Error::DeviceLost);
//             }

//             // DXGI_ERROR_WAS_STILL_DRAWING means the compositor wasn't ready yet
//             // (shouldn't happen if wait_for_present_ready was called, but handle
//             // it gracefully by dropping the frame rather than propagating an error).
//             if result == DXGI_ERROR_WAS_STILL_DRAWING {
//                 return Ok(());
//             }
//         }

//         Ok(())
//     }

//     fn resize(&mut self, size: Size<usize>) -> Result<()> {
//         self.recreate_swapchain(size)
//     }

//     fn measure_text(
//         &mut self,
//         text_props: &TextProps,
//         available_size: Size<AvailableSpace>,
//     ) -> Result<Size<f32>> {
//         let max_width = match available_size.width {
//             AvailableSpace::Definite(px) => px,
//             _ => f32::MAX,
//         };
//         let max_height = match available_size.height {
//             AvailableSpace::Definite(px) => px,
//             _ => f32::MAX,
//         };

//         let fmt = self.cache.get_text_format(text_props)?;
//         let utf16: Vec<u16> = text_props.content.encode_utf16().collect();
//         let layout = unsafe {
//             self.dwrite_factory
//                 .CreateTextLayout(&utf16, &fmt, max_width, max_height)?
//         };

//         let mut metrics = DWRITE_TEXT_METRICS::default();
//         unsafe { layout.GetMetrics(&mut metrics)? };

//         Ok(Size::wh(metrics.width, metrics.height))
//     }
// }

use std::f32;

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
    core::Interface,
};
use windows_numerics::{Matrix3x2, Vector2};

use crate::{
    core::{
        error::*,
        layout::{
            AvailableSpace,
            types::{point::Point, rect::Rect, size::Size},
        },
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

        unsafe {
            SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)?;
        }

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

        let scale = 96.0 / dpi; // TODO: shouldn't this be dpi/96.0? why it works?

        let swapchain = self.create_swapchain(hwnd, &size)?;
        let d2d_device_context = self.create_device_context()?;

        unsafe {
            d2d_device_context.SetTextAntialiasMode(D2D1_TEXT_ANTIALIAS_MODE_CLEARTYPE);
            d2d_device_context.SetDpi(dpi, dpi);
        }

        unsafe {
            d2d_device_context.SetTransform(&Matrix3x2::scale(scale, scale)); // TODO: this is an escape hatch, fix in future
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
            #[cfg(debug_assertions)]
            D3D11CreateDevice(
                None,
                D3D_DRIVER_TYPE_HARDWARE,
                HMODULE::default(),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT
                    | D3D11_CREATE_DEVICE_SINGLETHREADED
                    | D3D11_CREATE_DEVICE_DEBUG,
                Some(&feature_levels),
                D3D11_SDK_VERSION,
                Some(&mut d3d_device),
                None,
                None,
            )?;
            #[cfg(not(debug_assertions))]
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
            AlphaMode: DXGI_ALPHA_MODE_UNSPECIFIED, // TODO: if i set premultiplied it crashes why?
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
                            let layout = self.cache.get_text_layout(props, bounds.size)?;

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
                let result = self.swapchain.Present(1, DXGI_PRESENT(0));

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
