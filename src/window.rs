use std::sync::atomic::{AtomicUsize, Ordering};

use windows::{
    Win32::{
        Foundation::LRESULT,
        UI::WindowsAndMessaging::{
            CS_OWNDC, CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, GWLP_USERDATA,
            GetWindowLongPtrW, IDC_ARROW, LoadCursorW, PostQuitMessage, RegisterClassExW,
            SetWindowLongPtrW, WM_CLOSE, WS_EX_NOREDIRECTIONBITMAP, WS_OVERLAPPEDWINDOW,
            WS_VISIBLE,
        },
    },
    core::{HSTRING, PCWSTR},
};

use crate::{app::*, core::*, error::*, types::*};

pub struct WindowStyle {
    pub title: String,
    pub position: Point,
    pub size: Size,
}

impl Default for WindowStyle {
    fn default() -> Self {
        Self {
            position: Point::new(CW_USEDEFAULT as f32, CW_USEDEFAULT as f32),
            size: Size::new(640.0, 480.0),
            title: "Window".to_string(),
        }
    }
}

pub struct Window {
    pub hwnd: foundation::Hwnd,
}

impl Window {
    pub fn new(app: &App, style: WindowStyle) -> Result<Box<Self>> {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let count = COUNTER.fetch_add(1, Ordering::Relaxed);

        let title = HSTRING::from(style.title);
        let title_ptr = PCWSTR(title.as_ptr());

        let class_name = HSTRING::from(format!("Window - {}", count));
        let class_name_ptr = PCWSTR(class_name.as_ptr());

        let wc = window::Wndclass {
            cbSize: std::mem::size_of::<window::Wndclass>() as u32,
            style: CS_OWNDC,
            lpfnWndProc: Some(Self::window_proc),
            // cbClsExtra: todo!(),
            // cbWndExtra: todo!(),
            hInstance: app.instance(),
            // hIcon: unsafe { LoadIconW(None, IDI_APPLICATION)? },
            hCursor: unsafe { LoadCursorW(None, IDC_ARROW)? },
            // hbrBackground: todo!(),
            // lpszMenuName: todo!(),
            lpszClassName: class_name_ptr,
            // hIconSm: todo!(),
            ..Default::default()
        };

        unsafe {
            RegisterClassExW(&wc);
        }

        let hwnd = unsafe {
            CreateWindowExW(
                WS_EX_NOREDIRECTIONBITMAP,
                class_name_ptr,
                title_ptr,
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                style.position.x as i32,
                style.position.y as i32,
                style.size.width as i32,
                style.size.height as i32,
                None,
                None,
                Some(app.instance()),
                None,
            )?
        };

        let mut window = Box::new(Self { hwnd });

        unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, window.as_mut() as *mut _ as isize) };

        Ok(window)
    }

    fn handle_message(
        &mut self,
        msg: u32,
        wparam: foundation::Wparam,
        lparam: foundation::Lparam,
    ) -> LRESULT {
        match msg {
            WM_CLOSE => {
                unsafe { PostQuitMessage(0) };
                LRESULT(0)
            }
            _ => unsafe { DefWindowProcW(self.hwnd, msg, wparam, lparam) },
        }
    }

    unsafe extern "system" fn window_proc(
        hwnd: foundation::Hwnd,
        msg: u32,
        wparam: foundation::Wparam,
        lparam: foundation::Lparam,
    ) -> foundation::Lresult {
        let ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Self };

        if ptr.is_null() {
            return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
        }

        let window = unsafe { &mut *ptr };

        window.handle_message(msg, wparam, lparam)
    }

    pub fn hwnd(&self) -> foundation::Hwnd {
        self.hwnd
    }
}
