use windows::Win32::Graphics::{
    Direct2D::*,
    Direct3D11::*,
    DirectComposition::*,
    DirectWrite::*,
    Dxgi::*,
};

pub type D3DDevice = ID3D11Device5;

pub type D2DFactory = ID2D1Factory8;
pub type D2DDevice = ID2D1Device7;
pub type D2DDeviceContext = ID2D1DeviceContext7;
pub type D2DBitmap = ID2D1Bitmap1;
pub type D2DSolidColorBrush = ID2D1SolidColorBrush;

pub type DWriteFactory = IDWriteFactory8;
pub type DWriteTextFormat = IDWriteTextFormat3;
pub type DWriteTextLayout = IDWriteTextLayout4;

pub type DXGIAdapter = IDXGIAdapter4;
pub type DXGIDevice = IDXGIDevice4;
pub type DXGIFactory = IDXGIFactory7;
pub type DXGISwapChain = IDXGISwapChain3;
pub type DXGISurface = IDXGISurface2;

pub type DCompDevice = IDCompositionDevice;
pub type DCompTarget = IDCompositionTarget;
pub type DCompVisual = IDCompositionVisual;
