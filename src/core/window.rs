use crate::{
    core::{
        engine::Engine,
        error::*,
        layout::{Point, Size},
        render::d2d::{D2DRenderer, D2DRendererFactory},
    },
    elements::{Element, div::div},
};

use windows::{
    Win32::{Foundation::*, UI::WindowsAndMessaging::*},
    core::{HSTRING, PCWSTR},
};

pub unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let window_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut Window;

    if window_ptr.is_null() {
        return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
    }

    let window = unsafe { &mut *window_ptr };

    match msg {
        WM_PAINT => {
            let _ = window.engine.frame(); // TODO: change
            LRESULT(0)
        }
        WM_SIZE => {
            let width = (lparam.0 as u32 & 0xFFFF) as usize;
            let height = ((lparam.0 as u32 >> 16) & 0xFFFF) as usize;

            let old_size = window.engine.get_size();

            if !(old_size.width == width && old_size.height == height) {
                window.engine.resize(Size::wh(width, height)).unwrap();

                let result = window.engine.frame(); // TODO: change
                if matches!(result, Err(Error::DeviceLost)) {
                    todo!()
                }
            }

            LRESULT(0)
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

pub struct Window {
    hwnd: HWND,
    engine: Engine<D2DRenderer>,
}

impl Window {
    pub fn build(
        hinstance: HINSTANCE,
        title: String,
        size: Size<usize>,
        position: Option<Point<usize>>,
        factory: &D2DRendererFactory,
        root: Box<dyn Element>,
    ) -> Result<Box<Window>> {
        let mut rect = RECT {
            left: 0,
            top: 0,
            right: size.width as i32,
            bottom: size.height as i32,
        };
        unsafe { AdjustWindowRect(&mut rect, WS_OVERLAPPEDWINDOW, false)? };

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
                None,
            )?
        };

        let renderer = factory.create_renderer_for_hwnd(hwnd, size)?;
        let engine = Engine::new(renderer, root, size)?;

        let window = Box::new(Window { hwnd, engine });
        let raw = Box::into_raw(window);

        unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, raw as isize) };

        Ok(unsafe { Box::from_raw(raw) })
    }
}

pub struct WindowBuilder {
    title: String,
    size: Size<usize>,
    position: Option<Point<usize>>,
    root: Box<dyn Element>,
}

impl WindowBuilder {
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

    pub fn root(mut self, root: impl Element + 'static) -> Self {
        self.root = Box::new(root);
        self
    }

    pub fn build(self, hinstance: HINSTANCE, factory: &D2DRendererFactory) -> Result<Box<Window>> {
        Window::build(
            hinstance,
            self.title,
            self.size,
            self.position,
            factory,
            self.root,
        )
    }
}

impl Default for WindowBuilder {
    fn default() -> Self {
        Self {
            title: String::from("window"),
            size: Size::wh(800, 600),
            position: None,
            root: Box::new(div()),
        }
    }
}

pub fn window() -> WindowBuilder {
    WindowBuilder::default()
}
