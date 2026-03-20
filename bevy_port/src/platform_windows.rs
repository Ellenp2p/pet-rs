#![cfg(target_os = "windows")]

use raw_window_handle::HasRawWindowHandle;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{CombineRgn, CreateRectRgn, SetWindowRgn, HRGN, RGN_OR};

pub fn set_window_region_from_image<W: HasRawWindowHandle + ?Sized>(window: &W, img: &image::RgbaImage) {
    use raw_window_handle::RawWindowHandle;

    let raw = window.raw_window_handle();
    let hwnd = match raw {
        RawWindowHandle::Win32(h) => {
            if h.hwnd.is_null() {
                return;
            }
            HWND(h.hwnd as isize)
        }
        _ => return,
    };

    unsafe {
        let mut dest: HRGN = CreateRectRgn(0, 0, 0, 0);
        let w = img.width() as i32;
        let h = img.height() as i32;
        let threshold = 10u8;
        for y in 0..h {
            let mut x = 0;
            while x < w {
                let mut found = false;
                let mut start = x;
                while start < w {
                    let p = img.get_pixel(start as u32, y as u32).0[3];
                    if p > threshold {
                        found = true;
                        break;
                    }
                    start += 1;
                }
                if !found { break; }
                let mut end = start + 1;
                while end < w {
                    let p = img.get_pixel(end as u32, y as u32).0[3];
                    if p <= threshold { break; }
                    end += 1;
                }
                let r = CreateRectRgn(start, y, end, y + 1);
                CombineRgn(dest, dest, r, RGN_OR);
                x = end + 1;
            }
        }
        let _ = SetWindowRgn(hwnd, dest, true);
    }
}
