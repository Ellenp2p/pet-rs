use bevy::prelude::*;
use bevy::winit::WinitWindows;
mod platform_windows;
use image::RgbaImage;

const SPRITE_W: u32 = 256;
const SPRITE_H: u32 = 256;
const WINDOW_W: f32 = SPRITE_W as f32;
const WINDOW_H: f32 = SPRITE_H as f32;

#[derive(Resource)]
struct Frames {
    handles: Vec<Handle<Image>>,
    raws: Vec<RgbaImage>,
    idx: usize,
}

#[derive(Resource)]
struct Paused(bool);

#[derive(Component)]
struct Pet;

#[derive(Resource, Default)]
struct DragState {
    entity: Option<Entity>,
    offset: Vec2,
}

#[derive(Resource, Default)]
struct MenuState {
    root: Option<Entity>,
}

#[derive(Component)]
struct PauseButton;
#[derive(Component)]
struct QuitButton;

#[derive(Resource)]
struct FontHandle(Handle<Font>);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "pet-rs (Bevy port)".into(),
                resolution: (WINDOW_W, WINDOW_H).into(),
                resizable: false,
                decorations: false,
                transparent: true,
                ..Default::default()
            }),
            ..default()
        }))
        .insert_resource(Paused(false))
        .insert_resource(DragState::default())
        .insert_resource(MenuState::default())
        .add_startup_system(setup)
        .add_system(animate_frames)
        .add_system(start_drag)
        .add_system(drag_move)
        .add_system(end_drag)
        .add_system(native_window_drag)
        .add_system(right_click_menu_system)
        .add_system(menu_interaction)
        .run();
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());

    // try load spritesheet
    let frames_rgba = if let Ok(img) = image::open("assets/spritesheet.png") {
        let img = img.to_rgba8();
        let w = img.width();
        let h = img.height();
        if w >= h && w % h == 0 {
            let count = (w / h) as usize;
            let mut frames = Vec::with_capacity(count);
            for i in 0..count {
                let mut frame = RgbaImage::new(h, h);
                for y in 0..h {
                    for x in 0..h {
                        let sx = i as u32 * h + x;
                        let p = img.get_pixel(sx, y);
                        frame.put_pixel(x, y, *p);
                    }
                }
                frames.push(frame);
            }
            frames
        } else if h >= w && h % w == 0 {
            let count = (h / w) as usize;
            let mut frames = Vec::with_capacity(count);
            for i in 0..count {
                let mut frame = RgbaImage::new(w, w);
                for y in 0..w {
                    for x in 0..w {
                        let sy = i as u32 * w + y;
                        let p = img.get_pixel(x, sy);
                        frame.put_pixel(x, y, *p);
                    }
                }
                frames.push(frame);
            }
            frames
        } else {
            vec![img]
        }
    } else {
        // procedural fallback (8 frames)
        let mut frames = Vec::new();
        for i in 0..8usize {
            let mut img = RgbaImage::new(SPRITE_W, SPRITE_H);
            let cx = (SPRITE_W / 2) as i32;
            let cy = (SPRITE_H / 2) as i32;
            let radius = (SPRITE_W.min(SPRITE_H) as f32 * 0.4) as i32;
            for y in 0..(SPRITE_H as i32) {
                for x in 0..(SPRITE_W as i32) {
                    let dx = x - cx;
                    let dy = y - cy;
                    let dist2 = dx * dx + dy * dy;
                    if dist2 <= radius * radius {
                        let t = (i as f32) / 8.0;
                        let r = ((0.5 + 0.5 * (t * 10.0).cos()) * 255.0) as u8;
                        let g = ((0.5 + 0.5 * (t * 10.0).sin()) * 255.0) as u8;
                        let b = 200u8.saturating_sub((dx.abs() as u8).saturating_mul(2));
                        img.put_pixel(x as u32, y as u32, image::Rgba([r, g, b, 255]));
                    } else {
                        img.put_pixel(x as u32, y as u32, image::Rgba([0, 0, 0, 0]));
                    }
                }
            }
            frames.push(img);
        }
        frames
    };

    // create Bevy images and handles
    let mut handles = Vec::new();
    for f in frames_rgba.iter() {
        let data = f.clone().into_raw();
        let image = Image::new_fill(
            Extent3d::new(f.width(), f.height(), 1),
            TextureDimension::D2,
            &data,
            TextureFormat::Rgba8UnormSrgb,
        );
        handles.push(images.add(image));
    }

    let frames_res = Frames { handles, raws: frames_rgba, idx: 0 };
    let first = frames_res.handles[0].clone();
    commands.insert_resource(frames_res);
    // load font for menu UI
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    commands.insert_resource(FontHandle(font));

    // spawn pet sprite
    commands
        .spawn(SpriteBundle {
            texture: first,
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            sprite: Sprite { custom_size: Some(Vec2::new(SPRITE_W as f32, SPRITE_H as f32)), ..Default::default() },
            ..Default::default()
        })
        .insert(Pet);
}

fn animate_frames(
    time: Res<Time>,
    mut frames: ResMut<Frames>,
    mut q: Query<&mut Handle<Image>, With<Pet>>,
    windows: Query<&Window>,
    winit_windows: NonSend<WinitWindows>,
) {
    // simple timer-based frame advance
    // advance every 0.1s
    const D: f32 = 0.1;
    static mut ACC: f32 = 0.0;
    unsafe { ACC += time.delta_seconds();
        if ACC >= D {
            ACC = 0.0;
            frames.idx = (frames.idx + 1) % frames.handles.len();
            for mut h in q.iter_mut() {
                *h = frames.handles[frames.idx].clone();
            }
            // Update native window region for clickable area on Windows using the raw frame
            #[cfg(target_os = "windows")]
            {
                        if let Some(w) = windows.iter().next() {
                            if let Some(wnd) = winit_windows.get_window(w.id()) {
                                // call platform-specific region update
                                platform_windows::set_window_region_from_image(wnd, &frames.raws[frames.idx]);
                            }
                        }
            }
        }
    }
}

fn start_drag(
    mouse: Res<Input<MouseButton>>,
    windows: Query<&Window>,
    mut drag: ResMut<DragState>,
    pet_q: Query<(Entity, &Transform), With<Pet>>,
    frames: Res<Frames>,
    images: Res<Assets<Image>>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        let w = windows.iter().next().unwrap();
        if let Some(cursor) = w.cursor_position() {
            // convert cursor to world coords
            let pos = Vec2::new(cursor.x as f32 - w.resolution.width() as f32 / 2.0, cursor.y as f32 - w.resolution.height() as f32 / 2.0);
            // map to image pixel coords
            let img_x = (((cursor.x / w.resolution.width()) * SPRITE_W as f32) as u32).min(SPRITE_W - 1) as usize;
            let img_y = (((cursor.y / w.resolution.height()) * SPRITE_H as f32) as u32).min(SPRITE_H - 1) as usize;
            if let Some(img) = images.get(&frames.handles[frames.idx]) {
                let idx = (img_y * img.texture_descriptor.size.width as usize + img_x) * 4;
                if idx + 3 < img.data.len() {
                    let alpha = img.data[idx + 3];
                    if alpha > 10 {
                        for (ent, t) in pet_q.iter() {
                            let pet_pos = t.translation.truncate();
                            drag.entity = Some(ent);
                            drag.offset = pet_pos - pos;
                            break;
                        }
                    }
                }
            }
        }
    }
}

fn drag_move(
    mouse: Res<Input<MouseButton>>,
    windows: Query<&Window>,
    mut drag: ResMut<DragState>,
    mut pet_q: Query<&mut Transform, With<Pet>>,
) {
    if drag.entity.is_none() { return; }
    if mouse.pressed(MouseButton::Left) {
        let w = windows.iter().next().unwrap();
        if let Some(cursor) = w.cursor_position() {
            let pos = Vec2::new(cursor.x as f32 - w.resolution.width() as f32 / 2.0, cursor.y as f32 - w.resolution.height() as f32 / 2.0);
            if let Some(ent) = drag.entity {
                if let Ok(mut t) = pet_q.get_mut(ent) {
                    t.translation.x = (pos + drag.offset).x;
                    t.translation.y = (pos + drag.offset).y;
                }
            }
        }
    }
}

fn end_drag(mouse: Res<Input<MouseButton>>, mut drag: ResMut<DragState>) {
    if mouse.just_released(MouseButton::Left) {
        drag.entity = None;
    }
}

fn native_window_drag(
    mouse: Res<Input<MouseButton>>,
    windows: Query<&Window>,
    winit_windows: NonSend<WinitWindows>,
    frames: Res<Frames>,
    images: Res<Assets<Image>>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        let w = windows.iter().next().unwrap();
        if let Some(cursor) = w.cursor_position() {
            let img_x = (((cursor.x / w.resolution.width()) * SPRITE_W as f32) as u32).min(SPRITE_W - 1) as usize;
            let img_y = (((cursor.y / w.resolution.height()) * SPRITE_H as f32) as u32).min(SPRITE_H - 1) as usize;
            if let Some(img) = images.get(&frames.handles[frames.idx]) {
                let idx = (img_y * img.texture_descriptor.size.width as usize + img_x) * 4;
                if idx + 3 < img.data.len() {
                    let alpha = img.data[idx + 3];
                    if alpha == 0 {
                        if let Some(wnd) = winit_windows.get_window(w.id()) {
                            let _ = wnd.drag_window();
                        }
                    }
                }
            }
        }
    }
}

fn right_click_menu_system(
    mut commands: Commands,
    mouse: Res<Input<MouseButton>>,
    windows: Query<&Window>,
    mut menu: ResMut<MenuState>,
    font: Res<FontHandle>,
) {
    if mouse.just_pressed(MouseButton::Right) {
        if menu.root.is_some() {
            if let Some(root) = menu.root.take() {
                commands.entity(root).despawn_recursive();
            }
        } else {
            let w = windows.iter().next().unwrap();
            let cursor = w.cursor_position().unwrap_or(Vec2::new(WINDOW_W/2.0, WINDOW_H/2.0));
            let root = commands
                .spawn(NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        position: UiRect {
                            left: Val::Px(cursor.x as f32),
                            bottom: Val::Px(cursor.y as f32),
                            ..Default::default()
                        },
                        size: Size::new(Val::Px(160.0), Val::Px(80.0)),
                        flex_direction: FlexDirection::Column,
                        ..Default::default()
                    },
                    background_color: BackgroundColor(Color::rgba(0.1,0.1,0.1,0.85)),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent
                        .spawn((
                            ButtonBundle {
                                style: Style { size: Size::new(Val::Percent(100.0), Val::Px(40.0)), ..Default::default() },
                                background_color: BackgroundColor(Color::GRAY),
                                ..Default::default()
                            },
                            PauseButton,
                        ))
                        .with_children(|b| { b.spawn(TextBundle::from_section("Pause/Resume", TextStyle { font: font.0.clone(), font_size: 16.0, color: Color::WHITE })); });
                    parent
                        .spawn((
                            ButtonBundle {
                                style: Style { size: Size::new(Val::Percent(100.0), Val::Px(40.0)), ..Default::default() },
                                background_color: BackgroundColor(Color::DARK_RED),
                                ..Default::default()
                            },
                            QuitButton,
                        ))
                        .with_children(|b| { b.spawn(TextBundle::from_section("Quit", TextStyle { font: font.0.clone(), font_size: 16.0, color: Color::WHITE })); });
                })
                .id();
            menu.root = Some(root);
        }
    }
}

fn menu_interaction(
    mut commands: Commands,
    mut interactions: Query<(&Interaction, Option<&PauseButton>, Option<&QuitButton>), (Changed<Interaction>, With<Button>)>,
    mut paused: ResMut<Paused>,
    mut menu: ResMut<MenuState>,
    mut exit: EventWriter<AppExit>,
) {
    for (interaction, pause_opt, quit_opt) in interactions.iter_mut() {
        if *interaction == Interaction::Clicked {
            if pause_opt.is_some() {
                paused.0 = !paused.0;
                println!("Bevy port: paused = {}", paused.0);
            } else if quit_opt.is_some() {
                println!("Bevy port: Quit selected");
                exit.send(AppExit);
            }
            if let Some(root) = menu.root.take() {
                commands.entity(root).despawn_recursive();
            }
        }
    }
}
