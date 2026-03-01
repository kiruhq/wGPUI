use anyhow::{Context as _, Result};
use gpui::{
    DevicePixels, GpuSpecs, PlatformAtlas, Scene, Size, WindowBackgroundAppearance, size,
};
use gpui_wgpu::{WgpuContext, WgpuRenderer, WgpuSurfaceConfig};
use raw_window_handle as rwh;
use std::num::NonZeroIsize;
use std::sync::Arc;
use windows::Win32::{
    Foundation::HWND,
    Graphics::Gdi::GetClientRect,
    UI::WindowsAndMessaging::{GWLP_HINSTANCE, GetWindowLongPtrW},
};

use crate::DirectXDevices;

pub(crate) struct WgpuWindowRenderer {
    renderer: WgpuRenderer,
    supports_subpixel_rendering: bool,
}

impl WgpuWindowRenderer {
    pub(crate) fn new(
        hwnd: HWND,
        _directx_devices: &DirectXDevices,
        _disable_direct_composition: bool,
    ) -> Result<Self> {
        let mut gpu_context = None;
        let raw_window = RawWindow { hwnd };
        let renderer = WgpuRenderer::new(
            &mut gpu_context,
            &raw_window,
            WgpuSurfaceConfig {
                size: window_size(hwnd),
                transparent: false,
            },
        )
        .context("Creating wgpu renderer for Windows window")?;
        let supports_subpixel_rendering = renderer.supports_dual_source_blending();

        Ok(Self {
            renderer,
            supports_subpixel_rendering,
        })
    }

    pub(crate) fn draw(
        &mut self,
        scene: &Scene,
        _background_appearance: WindowBackgroundAppearance,
    ) -> Result<()> {
        self.renderer.draw(scene);
        Ok(())
    }

    pub(crate) fn resize(&mut self, new_size: Size<DevicePixels>) -> Result<()> {
        self.renderer.update_drawable_size(new_size);
        Ok(())
    }

    pub(crate) fn sprite_atlas(&self) -> Arc<dyn PlatformAtlas> {
        self.renderer.sprite_atlas().clone()
    }

    pub(crate) fn gpu_specs(&self) -> Result<GpuSpecs> {
        Ok(self.renderer.gpu_specs())
    }

    pub(crate) fn update_transparency(&mut self, transparent: bool) {
        self.renderer.update_transparency(transparent);
    }

    pub(crate) fn supports_subpixel_rendering(&self) -> bool {
        self.supports_subpixel_rendering
    }

    pub(crate) fn handle_device_lost(&mut self, _directx_devices: &DirectXDevices) -> Result<()> {
        Ok(())
    }

    pub(crate) fn mark_drawable(&mut self) {}
}

struct RawWindow {
    hwnd: HWND,
}

impl rwh::HasWindowHandle for RawWindow {
    fn window_handle(&self) -> std::result::Result<rwh::WindowHandle<'_>, rwh::HandleError> {
        let Some(hwnd) = NonZeroIsize::new(self.hwnd.0 as isize) else {
            return Err(rwh::HandleError::Unavailable);
        };
        let mut raw = rwh::Win32WindowHandle::new(hwnd);
        raw.hinstance = NonZeroIsize::new(unsafe { GetWindowLongPtrW(self.hwnd, GWLP_HINSTANCE) });
        // SAFETY: The HWND/HINSTANCE are borrowed from a live window and remain valid while
        // this handle is used to create the surface.
        Ok(unsafe { rwh::WindowHandle::borrow_raw(rwh::RawWindowHandle::Win32(raw)) })
    }
}

impl rwh::HasDisplayHandle for RawWindow {
    fn display_handle(&self) -> std::result::Result<rwh::DisplayHandle<'_>, rwh::HandleError> {
        // SAFETY: Windows display handles do not borrow external data.
        Ok(unsafe {
            rwh::DisplayHandle::borrow_raw(rwh::RawDisplayHandle::Windows(
                rwh::WindowsDisplayHandle::new(),
            ))
        })
    }
}

fn window_size(hwnd: HWND) -> Size<DevicePixels> {
    let mut rect = windows::Win32::Foundation::RECT::default();
    if unsafe { GetClientRect(hwnd, &mut rect).is_err() } {
        return size(DevicePixels(1), DevicePixels(1));
    }
    size(
        DevicePixels((rect.right - rect.left).max(1)),
        DevicePixels((rect.bottom - rect.top).max(1)),
    )
}
