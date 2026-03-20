use bevy::prelude::*;
use bevy::winit::WinitWindows;

#[derive(Resource)]
struct Paused(pub bool);

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

#[derive(Resource)]
struct PetTexture(Handle<Image>);

#[derive(Component)]
struct PauseButton;

#[derive(Component)]
struct QuitButton;

const WINDOW_W: f32 = 320.;
const WINDOW_H: f32 = 320.;
const PET_SIZE: f32 = 128.;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "pet-rs (Bevy prototype)".into(),
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
        .add_system(spin_pet)
        .add_system(start_drag)
        .add_system(drag_move)
        .add_system(end_drag)
        .add_system(right_click_menu_system)
        .add_system(menu_interaction)
        .run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.spawn(Camera2dBundle::default());

    // generate a simple circular RGBA image for the pet
    let size = PET_SIZE as usize;
    let mut data = Vec::with_capacity(size * size * 4);
    let cx = (size / 2) as i32;
    let cy = (size / 2) as i32;
    let radius = (size as f32 * 0.45) as i32;
    for y in 0..size {
        for x in 0..size {
            let dx = x as i32 - cx;
            let dy = y as i32 - cy;
            if dx * dx + dy * dy <= radius * radius {
                // simple gradient color
                let r = ((x as f32 / size as f32) * 255.0) as u8;
                let g = ((y as f32 / size as f32) * 200.0) as u8;
                let b = 180u8;
                data.push(r);
                data.push(g);
                data.push(b);
                data.push(255);
            } else {
                data.push(0); data.push(0); data.push(0); data.push(0);
            }
        }
    }

    let image = Image::new_fill(
        Extent3d::new(size as u32, size as u32, 1),
        TextureDimension::D2,
        &data,
        TextureFormat::Rgba8UnormSrgb,
    );
    let handle = images.add(image);
    // store pet texture handle for hit-testing/drag decisions
    commands.insert_resource(PetTexture(handle.clone()));

    // spawn pet sprite in center
    commands
        .spawn(SpriteBundle {
            texture: handle.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)).with_scale(Vec3::splat(1.0)),
            sprite: Sprite { custom_size: Some(Vec2::new(PET_SIZE, PET_SIZE)), ..Default::default() },
            ..Default::default()
        })
        .insert(Pet);

    // UI root exists but menu spawned on demand
}

fn spin_pet(time: Res<Time>, paused: Res<Paused>, mut q: Query<&mut Transform, With<Pet>>) {
    if paused.0 { return; }
    for mut t in q.iter_mut() {
        t.rotate_z(0.5 * time.delta_seconds());
    }
}

fn start_drag(
    mouse: Res<Input<MouseButton>>,
    windows: Query<&Window>,
    mut drag: ResMut<DragState>,
    pet_q: Query<(Entity, &Transform), With<Pet>>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        let w = windows.iter().next().unwrap();
        if let Some(cursor) = w.cursor_position() {
            // convert cursor (window coords) to world coords
            let pos = Vec2::new(
                cursor.x as f32 - (w.resolution.width() as f32) / 2.0,
                cursor.y as f32 - (w.resolution.height() as f32) / 2.0,
            );
            for (ent, t) in pet_q.iter() {
                let pet_pos = t.translation.truncate();
                let half = PET_SIZE / 2.0;
                let min = pet_pos - Vec2::splat(half);
                let max = pet_pos + Vec2::splat(half);
                if pos.x >= min.x && pos.x <= max.x && pos.y >= min.y && pos.y <= max.y {
                    // simple circular hit test relative to pet center
                    let local = pos - pet_pos;
                    if local.length() <= half * 0.9 {
                        drag.entity = Some(ent);
                        drag.offset = pet_pos - pos;
                        break;
                    }
                }
            }
        }
    }
}

fn drag_move(
    windows: Query<&Window>,
    mouse: Res<Input<MouseButton>>,
    mut drag: ResMut<DragState>,
    mut pet_q: Query<&mut Transform, With<Pet>>,
) {
    if drag.entity.is_none() { return; }
    if mouse.pressed(MouseButton::Left) {
        let w = windows.iter().next().unwrap();
        if let Some(cursor) = w.cursor_position() {
            let pos = Vec2::new(
                cursor.x as f32 - (w.resolution.width() as f32) / 2.0,
                cursor.y as f32 - (w.resolution.height() as f32) / 2.0,
            );
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

fn right_click_menu_system(
    mut commands: Commands,
    mouse: Res<Input<MouseButton>>,
    windows: Query<&Window>,
    mut menu: ResMut<MenuState>,
) {
    if mouse.just_pressed(MouseButton::Right) {
        // toggle menu: if open, close; else open at cursor
        if menu.root.is_some() {
            if let Some(root) = menu.root.take() {
                commands.entity(root).despawn_recursive();
            }
        } else {
            let w = windows.iter().next().unwrap();
            let cursor = w.cursor_position().unwrap_or(Vec2::new(WINDOW_W/2.0, WINDOW_H/2.0));
            // spawn UI menu
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
                            .with_children(|b| {
                                b.spawn(NodeBundle { ..Default::default() });
                            });
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: Style { size: Size::new(Val::Percent(100.0), Val::Px(40.0)), ..Default::default() },
                                    background_color: BackgroundColor(Color::DARK_RED),
                                    ..Default::default()
                                },
                                QuitButton,
                            ))
                            .with_children(|b| {
                                b.spawn(NodeBundle { ..Default::default() });
                            });
                    })
                    .id();
            menu.root = Some(root);
        }
    }
}

fn menu_interaction(
    mut commands: Commands,
    mut interactions: Query<(Entity, &Interaction, Option<&PauseButton>, Option<&QuitButton>), (Changed<Interaction>, With<Button>)>,
    mut paused: ResMut<Paused>,
    mut menu: ResMut<MenuState>,
    mut exit: EventWriter<AppExit>,
) {
    for (entity, interaction, pause_opt, quit_opt) in interactions.iter_mut() {
        if *interaction == Interaction::Clicked {
            if pause_opt.is_some() {
                paused.0 = !paused.0;
                println!("Bevy prototype: paused = {}", paused.0);
            } else if quit_opt.is_some() {
                println!("Bevy prototype: Quit selected");
                exit.send(AppExit);
            }
            // close menu
            if let Some(root) = menu.root.take() {
                commands.entity(root).despawn_recursive();
            }
        }
    }
}

// If user clicks/drag on transparent area (outside pet), request native window drag via winit
fn native_window_drag(
    mouse: Res<Input<MouseButton>>,
    windows: Query<&Window>,
    winit_windows: NonSend<WinitWindows>,
    pet_tex: Res<PetTexture>,
    images: Res<Assets<Image>>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        let w = windows.iter().next().unwrap();
        if let Some(cursor) = w.cursor_position() {
            // convert cursor to image pixel coords
            let ix = ((cursor.x as f32 / w.resolution.width()) * PET_SIZE) as usize;
            let iy = ((cursor.y as f32 / w.resolution.height()) * PET_SIZE) as usize;
            if let Some(img) = images.get(&pet_tex.0) {
                let px = ix.min(img.texture_descriptor.size.width as usize - 1);
                let py = iy.min(img.texture_descriptor.size.height as usize - 1);
                // image data in rgba8
                let idx = (py * img.texture_descriptor.size.width as usize + px) * 4;
                if idx + 3 < img.data.len() {
                    let alpha = img.data[idx + 3];
                    if alpha == 0 {
                        // transparent pixel -> start native drag
                        if let Some(wnd) = winit_windows.get_window(w.id()) {
                            let _ = wnd.drag_window();
                        }
                    }
                }
            }
        }
    }
}
