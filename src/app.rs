use windows::Win32::{
    System::LibraryLoader::GetModuleHandleW,
    UI::WindowsAndMessaging::{DispatchMessageW, GetMessageW, MSG, TranslateMessage},
};

use crate::{error::*, types::*};

pub struct App {
    pub instance: foundation::Instance,
}

impl App {
    pub fn new() -> Result<Self> {
        let instance = unsafe { GetModuleHandleW(None)?.into() };

        Ok(Self { instance: instance })
    }

    pub fn instance(&self) -> foundation::Instance {
        self.instance
    }

    pub fn run(&self) {
        let mut msg = MSG::default();
        while unsafe { GetMessageW(&mut msg, None, 0, 0).as_bool() } {
            let _ = unsafe { TranslateMessage(&mut msg) }; // TODO: Handle error
            unsafe { DispatchMessageW(&msg) };
        }
    }
}
