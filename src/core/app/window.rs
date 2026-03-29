use crate::{
    core::{
        app::engine::Engine,
        error::*,
        event::{ EngineEvent, MouseButton },
        layout::types::{ point::Point, rect::Rect, size::Size },
        render::{ Dpi, d2d::{ D2DRenderer, D2DRendererFactory } },
    },
    elements::{ Element, div::div },
};

use windows::{
    Win32::{
        Foundation::*,
        Graphics::{ Dwm::*, Gdi::* },
        UI::{ HiDpi::*, WindowsAndMessaging::* },
    },
    core::{ HSTRING, PCWSTR },
};
use windows_result::BOOL;

pub unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM
) -> LRESULT {
    let window_ptr = (unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) }) as *mut WindowState;

    if window_ptr.is_null() {
        return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
    }

    let window = unsafe { &mut *window_ptr };

    // TODO: Handle errors
    match msg {
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            unsafe {
                BeginPaint(hwnd, &mut ps);
            }
            unsafe {
                let _ = EndPaint(hwnd, &ps); // TODO: handle error
            }

            LRESULT(0)
        }
        WM_SIZE => {
            // WM_SIZE lparam gives physical pixels for per-monitor DPI aware windows.
            // Convert to logical DIPs since the engine and layout work in DIP units.
            let dpi_scale = ((unsafe { GetDpiForWindow(hwnd) }) as f32) / 96.0;
            let dpi_scale = if dpi_scale == 0.0 { 1.0 } else { dpi_scale };

            let size = Size::wh(
                ((((lparam.0 as u32) & 0xffff) as f32) / dpi_scale) as usize,
                (((((lparam.0 as u32) >> 16) & 0xffff) as f32) / dpi_scale) as usize
            );

            window.engine.dispatch_event(EngineEvent::WindowResized { size });

            LRESULT(0)
        }
        WM_MOUSEMOVE => {
            let dpi_scale = ((unsafe { GetDpiForWindow(hwnd) }) as f32) / 96.0;
            let dpi_scale = if dpi_scale == 0.0 { 1.0 } else { dpi_scale };

            let position = Point::xy(
                ((lparam.0 & 0xffff) as f32) / dpi_scale,
                (((lparam.0 >> 16) & 0xffff) as f32) / dpi_scale
            );

            window.engine.dispatch_event(EngineEvent::MouseMove { position });

            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            let dpi_scale = ((unsafe { GetDpiForWindow(hwnd) }) as f32) / 96.0;
            let dpi_scale = if dpi_scale == 0.0 { 1.0 } else { dpi_scale };

            let position = Point::xy(
                ((lparam.0 & 0xffff) as f32) / dpi_scale,
                (((lparam.0 >> 16) & 0xffff) as f32) / dpi_scale
            );

            window.engine.dispatch_event(EngineEvent::MouseDown {
                position,
                button: MouseButton::Left,
            });

            LRESULT(0)
        }
        WM_LBUTTONUP => {
            let dpi_scale = ((unsafe { GetDpiForWindow(hwnd) }) as f32) / 96.0;
            let dpi_scale = if dpi_scale == 0.0 { 1.0 } else { dpi_scale };

            let position = Point::xy(
                ((lparam.0 & 0xffff) as f32) / dpi_scale,
                (((lparam.0 >> 16) & 0xffff) as f32) / dpi_scale
            );

            window.engine.dispatch_event(EngineEvent::MouseUp {
                position,
                button: MouseButton::Left,
            });

            LRESULT(0)
        }
        WM_DPICHANGED => {
            let dpi = Dpi::new((wparam.0 & 0xffff) as f32, ((wparam.0 >> 16) & 0xffff) as f32);

            let rect = Rect::from(unsafe { *(lparam.0 as *const RECT) });

            window.engine.dispatch_event(EngineEvent::DpiChanged(rect, dpi));

            unsafe {
                let _ = SetWindowPos(
                    hwnd,
                    None,
                    rect.location.x,
                    rect.location.y,
                    rect.size.width,
                    rect.size.height,
                    SWP_NOZORDER | SWP_NOACTIVATE
                );
            }

            LRESULT(0)
        }
        WM_DESTROY => {
            window.engine.dispatch_event(EngineEvent::WindowDestroyed);
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
    }
}

pub struct WindowState {
    hwnd: HWND,
    engine: Engine<D2DRenderer>,
}

impl WindowState {
    pub fn build(
        hinstance: HINSTANCE,
        title: String,
        size: Size<usize>,
        position: Option<Point<usize>>,
        mode: WindowMode,
        backdrop: WindowBackdrop,
        factory: &D2DRendererFactory,
        root: Box<dyn Element>
    ) -> Result<Box<WindowState>> {
        // Scale logical size to physical pixels for the current system DPI.
        // CreateWindowExW takes physical pixels when per-monitor DPI aware.
        let system_dpi = unsafe { GetDpiForSystem() };
        let dpi_scale = (system_dpi as f32) / 96.0;

        let physical_width = ((size.width as f32) * dpi_scale) as i32;
        let physical_height = ((size.height as f32) * dpi_scale) as i32;

        let mut rect = RECT {
            left: 0,
            top: 0,
            right: physical_width,
            bottom: physical_height,
        };

        unsafe {
            AdjustWindowRectExForDpi(
                &mut rect,
                WS_OVERLAPPEDWINDOW,
                false,
                WS_EX_NOREDIRECTIONBITMAP,
                system_dpi
            )?;
        }

        let window_width = rect.right - rect.left;
        let window_height = rect.bottom - rect.top;

        let title = HSTRING::from(title);
        let class_name = HSTRING::from("window_class");

        let hwnd = unsafe {
            CreateWindowExW(
                WS_EX_NOREDIRECTIONBITMAP,
                PCWSTR(class_name.as_ptr()),
                PCWSTR(title.as_ptr()),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                match position {
                    Some(point) => point.x as i32,
                    None => CW_USEDEFAULT,
                },
                match position {
                    Some(point) => point.y as i32,
                    None => CW_USEDEFAULT,
                },
                window_width as i32,
                window_height as i32,
                None,
                None,
                Some(hinstance),
                None
            )?
        };

        let renderer = factory.create_renderer_for_hwnd(hwnd, size)?;
        let mut engine = Engine::new(renderer, root, size)?;

        engine.dispatch_event(EngineEvent::WindowCreated);

        let window = Box::new(WindowState { hwnd, engine });

        let raw = Box::into_raw(window);

        unsafe {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, raw as isize);
        }
        setup_window(hwnd, mode, backdrop)?;

        Ok(unsafe { Box::from_raw(raw) })
    }

    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    pub fn engine_mut(&mut self) -> &mut Engine<D2DRenderer> {
        &mut self.engine
    }
}

pub enum WindowBackdrop {
    None,
    Auto,
    Mica,
    Acrylic,
    MicaAlt,
}

pub enum WindowMode {
    Light,
    Dark,
    System,
}

pub struct Window {
    title: String,
    size: Size<usize>,
    position: Option<Point<usize>>,
    mode: WindowMode,
    backdrop: WindowBackdrop,
    root: Box<dyn Element>,
}

impl Window {
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn size(mut self, size: Size<usize>) -> Self {
        self.size = size;
        self
    }

    pub fn position(mut self, position: Point<usize>) -> Self {
        self.position = Some(position);
        self
    }

    pub fn mode(mut self, mode: WindowMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn backdrop(mut self, backdrop: WindowBackdrop) -> Self {
        self.backdrop = backdrop;
        self
    }

    pub fn root(mut self, root: impl Element + 'static) -> Self {
        self.root = Box::new(root);
        self
    }

    pub fn build(
        self,
        hinstance: HINSTANCE,
        factory: &D2DRendererFactory
    ) -> Result<Box<WindowState>> {
        WindowState::build(
            hinstance,
            self.title,
            self.size,
            self.position,
            self.mode,
            self.backdrop,
            factory,
            self.root
        )
    }
}

impl Default for Window {
    fn default() -> Self {
        Self {
            title: String::from("window"),
            size: Size::wh(800, 600),
            position: None,
            mode: WindowMode::Light,
            backdrop: WindowBackdrop::None,
            root: Box::new(div()),
        }
    }
}

pub fn setup_window(hwnd: HWND, mode: WindowMode, backdrop: WindowBackdrop) -> Result<()> {
    let mode_type = match mode {
        WindowMode::Light => FALSE,
        WindowMode::Dark => TRUE,
        WindowMode::System => unimplemented!(), // TODO: implement
    };

    unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            &mode_type as *const _ as *const _,
            std::mem::size_of::<BOOL>() as u32
        )?;
    }

    let backdrop_type = match backdrop {
        WindowBackdrop::Auto => DWMSBT_AUTO,
        WindowBackdrop::None => DWMSBT_NONE,
        WindowBackdrop::Mica => DWMSBT_MAINWINDOW,
        WindowBackdrop::Acrylic => DWMSBT_TRANSIENTWINDOW,
        WindowBackdrop::MicaAlt => DWMSBT_TABBEDWINDOW,
    };

    unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_SYSTEMBACKDROP_TYPE,
            &backdrop_type as *const _ as *const _,
            std::mem::size_of::<DWMWINDOWATTRIBUTE>() as u32
        )?;
    }

    Ok(())
}

pub fn window() -> Window {
    Window::default()
}
