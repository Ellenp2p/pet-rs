#[cfg(target_os = "windows")]
mod windows_impl;
#[cfg(target_os = "windows")]
pub use windows_impl::*;

#[cfg(not(target_os = "windows"))]
pub fn set_window_region_from_image<W>(_window: &W, _img: &image::RgbaImage) {
    // no-op on non-Windows platforms for now
}
