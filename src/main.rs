use std::env;
use std::time::{Duration, Instant};

use image::{Rgba, RgbaImage};
mod platform;
use pixels::{Pixels, SurfaceTexture};
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::VirtualKeyCode;
use winit::event::{ElementState, Event, MouseButton, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

fn generate_frames(w: u32, h: u32, count: usize) -> Vec<RgbaImage> {
    let mut frames = Vec::with_capacity(count);
    for i in 0..count {
        let mut img = RgbaImage::new(w, h);
        let cx = (w / 2) as i32;
        let cy = (h / 2) as i32;
        let radius = (w.min(h) as f32 * 0.4) as i32;
        for y in 0..(h as i32) {
            for x in 0..(w as i32) {
                let dx = x - cx;
                let dy = y - cy;
                let dist2 = dx * dx + dy * dy;
                if dist2 <= radius * radius {
                    // simple animated color/pattern
                    let t = (i as f32) / (count as f32);
                    let r = ((0.5 + 0.5 * (t * 10.0).cos()) * 255.0) as u8;
                    let g = ((0.5 + 0.5 * (t * 10.0).sin()) * 255.0) as u8;
                    let b = (200u8.saturating_sub((dx.abs() as u8).saturating_mul(2))) as u8;
                    img.put_pixel(x as u32, y as u32, Rgba([r, g, b, 255]));
                } else {
                    img.put_pixel(x as u32, y as u32, Rgba([0, 0, 0, 0]));
                }
            }
        }
        frames.push(img);
    }
    frames
}

fn try_load_spritesheet(path: &str) -> Option<Vec<RgbaImage>> {
    use std::path::Path;
    if !Path::new(path).exists() {
        return None;
    }
    match image::open(path) {
        Ok(imgdyn) => {
            let img = imgdyn.to_rgba8();
            let w = img.width();
            let h = img.height();

            // Infer frames: if width is multiple of height treat as horizontal strip
            if w >= h && w % h == 0 {
                let count = (w / h) as usize;
                let mut frames = Vec::with_capacity(count);
                for i in 0..count {
                    let mut frame = RgbaImage::new(h, h);
                    for y in 0..(h as u32) {
                        for x in 0..(h as u32) {
                            let sx = i as u32 * h + x;
                            let p = img.get_pixel(sx, y);
                            frame.put_pixel(x, y, *p);
                        }
                    }
                    frames.push(frame);
                }
                Some(frames)
            } else if h >= w && h % w == 0 {
                // vertical strip
                let count = (h / w) as usize;
                let mut frames = Vec::with_capacity(count);
                for i in 0..count {
                    let mut frame = RgbaImage::new(w, w);
                    for y in 0..(w as u32) {
                        for x in 0..(w as u32) {
                            let sy = i as u32 * w + y;
                            let p = img.get_pixel(x, sy);
                            frame.put_pixel(x, y, *p);
                        }
                    }
                    frames.push(frame);
                }
                Some(frames)
            } else {
                // Not a simple strip; return single-frame fallback
                let mut single = Vec::new();
                single.push(img);
                Some(single)
            }
        }
        Err(_) => None,
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new();

    let sprite_w = 256u32;
    let sprite_h = 256u32;

    let window = WindowBuilder::new()
        .with_title("pet-rs")
        .with_decorations(false)
        .with_transparent(true)
        .with_inner_size(LogicalSize::new(sprite_w as f64, sprite_h as f64))
        .build(&event_loop)
        .unwrap();

    // NOTE: always-on-top not set here to keep compatibility across winit versions

    // Create the pixel buffer surface (physical size)
    let size = window.inner_size();
    let surface_texture = SurfaceTexture::new(size.width, size.height, &window);
    let mut pixels = Pixels::new(sprite_w, sprite_h, surface_texture).expect("Pixels init");

    // allow generating a sample spritesheet: run with `--gen-spritesheet` to create assets/spritesheet.png then exit
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "--gen-spritesheet") {
        let count = 8usize;
        let w = sprite_w;
        let h = sprite_h;
        let mut sheet = RgbaImage::new(w * count as u32, h);
        let frames_proc = generate_frames(w, h, count);
        for i in 0..count {
            let frame = &frames_proc[i];
            for y in 0..h {
                for x in 0..w {
                    let p = frame.get_pixel(x, y);
                    sheet.put_pixel(i as u32 * w + x, y, *p);
                }
            }
        }
        std::fs::create_dir_all("assets").ok();
        let _ = sheet.save("assets/spritesheet.png");
        println!("Generated assets/spritesheet.png");
        return;
    }

    // try to load frames from assets/spritesheet.png (horizontal/vertical strip).
    // if not present, fall back to procedural frames.
    let frames = if let Some(frames) = try_load_spritesheet("assets/spritesheet.png") {
        // If loaded frames don't match expected size, resize or crop to sprite_w,sprite_h
        let mut adjusted = Vec::new();
        for f in frames.into_iter() {
            let f = if f.width() != sprite_w || f.height() != sprite_h {
                image::imageops::resize(
                    &f,
                    sprite_w,
                    sprite_h,
                    image::imageops::FilterType::Lanczos3,
                )
            } else {
                f
            };
            adjusted.push(f);
        }
        adjusted
    } else {
        generate_frames(sprite_w, sprite_h, 8)
    };
    let mut frame_idx = 0usize;
    let frame_duration = Duration::from_millis(100);
    let mut last_frame = Instant::now();

    // drag state (we use the OS-level drag helper)
    let mut dragging = false;
    let mut last_cursor_pos: PhysicalPosition<f64> = PhysicalPosition::new(0.0, 0.0);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput {
                    input,
                    ..
                } => {
                    if input.state == ElementState::Pressed {
                        if let Some(VirtualKeyCode::Escape) = input.virtual_keycode {
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    // keep track of the last cursor position (physical coordinates)
                    last_cursor_pos = position;
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    match (button, state) {
                        (MouseButton::Left, ElementState::Pressed) => {
                            // map last known cursor position to image pixel coords
                            let pos = last_cursor_pos;
                            let phys = window.inner_size(); // physical size in pixels
                            let img_x = ((pos.x * (sprite_w as f64)) / (phys.width as f64)).round() as i32;
                            let img_y = ((pos.y * (sprite_h as f64)) / (phys.height as f64)).round() as i32;
                            if img_x >= 0
                                && img_y >= 0
                                && (img_x as u32) < sprite_w
                                && (img_y as u32) < sprite_h
                            {
                                let pixel = frames[frame_idx].get_pixel(img_x as u32, img_y as u32);
                                if pixel.0[3] > 10 {
                                    // start OS-level dragging — this hands drag control to the OS
                                    // and gives a much smoother, correct drag behavior.
                                    let _ = window.drag_window();
                                    dragging = true;
                                }
                            }
                        }
                        (MouseButton::Left, ElementState::Released) => {
                            dragging = false;
                        }
                        _ => {}
                    }
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                // update animation
                if last_frame.elapsed() >= frame_duration {
                    frame_idx = (frame_idx + 1) % frames.len();
                    last_frame = Instant::now();
                    window.request_redraw();
                }
            }
            Event::RedrawRequested(_) => {
                // copy frame RGBA into pixels frame (which is RGBA)
                let frame: &mut [u8] = pixels.frame_mut();
                let img = &frames[frame_idx];
                // update native window region for clickable area (Windows implementation)
                platform::set_window_region_from_image(&window, img);
                for y in 0..(sprite_h as usize) {
                    for x in 0..(sprite_w as usize) {
                        let p = img.get_pixel(x as u32, y as u32).0;
                        let i = (y * sprite_w as usize + x) * 4;
                        frame[i] = p[0];
                        frame[i + 1] = p[1];
                        frame[i + 2] = p[2];
                        frame[i + 3] = p[3];
                    }
                }

                if pixels.render().is_err() {
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => {}
        }
    });
}
