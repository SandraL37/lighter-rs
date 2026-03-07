use windows::{
    Win32::{
        Foundation::*, Graphics::Gdi::HBRUSH, System::LibraryLoader::*, UI::WindowsAndMessaging::*,
    },
    core::{HSTRING, PCWSTR},
};

use crate::core::{
    error::*,
    render::d2d::D2DRendererFactory,
    window::{Window, WindowBuilder, wnd_proc},
};

// TODO: abstract more
pub struct Application {
    hinstance: HINSTANCE,
    windows: Vec<Box<Window>>,
    factory: D2DRendererFactory,
}

impl Application {
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

    pub fn add_window(&mut self, builder: WindowBuilder) -> Result<()> {
        let window = builder.build(self.hinstance, &self.factory)?;
        self.windows.push(window);
        Ok(())
    }

    pub fn run(&self) {
        let mut msg = MSG::default();
        while unsafe { GetMessageW(&mut msg, None, 0, 0).0 } > 0 {
            unsafe { TranslateMessage(&msg) }; // TODO: handle error
            unsafe { DispatchMessageW(&msg) };
        }
    }
}
