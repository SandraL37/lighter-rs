pub mod engine;
pub mod window;

use windows::{
    Win32::{
        Foundation::*, Graphics::Gdi::HBRUSH, System::LibraryLoader::*, UI::WindowsAndMessaging::*,
    },
    core::{HSTRING, PCWSTR},
};

use crate::core::{
    app::window::{Window, WindowState, wnd_proc},
    error::*,
    render::d2d::D2DRendererFactory,
};

// TODO: abstract more
pub struct App {
    hinstance: HINSTANCE,
    windows: Vec<Box<WindowState>>,
    factory: D2DRendererFactory,
}

impl App {
    pub fn new() -> Result<Self> {
        let hinstance = unsafe { GetModuleHandleW(None)?.into() };

        let class_name = HSTRING::from("window_class");

        let class = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            style: WNDCLASS_STYLES::default(),
            lpfnWndProc: Some(wnd_proc),
            hInstance: hinstance,
            hbrBackground: HBRUSH(std::ptr::null_mut()),
            hCursor: unsafe { LoadCursorW(None, IDC_ARROW) }?,
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        unsafe { RegisterClassExW(&class) };

        let factory = D2DRendererFactory::new()?;

        Ok(Self {
            hinstance,
            factory,
            windows: vec![],
        })
    }

    pub fn add(mut self, window: Window) -> Result<Self> {
        let window = window.build(self.hinstance, &self.factory)?;
        self.windows.push(window);
        Ok(self)
    }

    fn check_windows(&mut self) {
        self.windows
            .retain(|window| unsafe { IsWindow(Some(window.hwnd())) }.as_bool());

        if self.windows.len() == 0 {
            unsafe { PostQuitMessage(0) };
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let mut msg = MSG::default();
        while unsafe { GetMessageW(&mut msg, None, 0, 0).0 } > 0 {
            if msg.message == custom_messages::WINDOWCLOSED {
                self.check_windows();
            }

            let _ = unsafe { TranslateMessage(&msg) }; // TODO: handle error
            unsafe { DispatchMessageW(&msg) };
        }
        Ok(())
    }
}

pub fn app() -> Result<App> {
    App::new()
}

mod custom_messages {
    pub const WINDOWCLOSED: u32 = 10001;
}
