pub mod foundation {
    use windows::Win32::Foundation::*;

    pub type Instance = HINSTANCE;
    pub type Hwnd = HWND;
    pub type Hmodule = HMODULE;
    pub type Wparam = WPARAM;
    pub type Lparam = LPARAM;
    pub type Lresult = LRESULT;

    pub type Point = POINT;
    pub type Rect = RECT;
}

pub mod window {
    pub use windows::Win32::UI::WindowsAndMessaging::*;

    pub type Wndclass = WNDCLASSEXW;
}

pub mod dcomp {
    use windows::Win32::Graphics::DirectComposition::*;

    pub type Target = IDCompositionTarget;
    pub type Visual = IDCompositionVisual3;
    pub type Device = IDCompositionDevice4;
    pub type DesktopDevice = IDCompositionDesktopDevice;
    pub type Surface = IDCompositionVirtualSurface;
}

pub mod dxgi {
    use windows::Win32::Graphics::Dxgi::*;

    pub type Factory = IDXGIFactory7;
    pub type Device = IDXGIDevice4;
    pub type Surface = IDXGISurface2;

    pub mod common {
        use windows::Win32::Graphics::Dxgi::Common::*;

        pub type PixelFormat = DXGI_FORMAT;
        pub type AlphaMode = DXGI_ALPHA_MODE;
    }
}

pub mod d2d {
    use windows::Win32::Graphics::Direct2D::*;

    pub type Factory = ID2D1Factory8;
    pub type Device = ID2D1Device7;
    pub type DeviceContext = ID2D1DeviceContext7;
    pub type Bitmap = ID2D1Bitmap1;

    pub mod common {
        use windows::Win32::Graphics::Direct2D::Common::*;

        pub type ColorF = D2D1_COLOR_F;
        pub type RectF = D2D_RECT_F;
    }
}
