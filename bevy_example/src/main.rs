use bevy::prelude::*;

#[derive(Resource)]
struct Paused(bool);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "pet-rs (Bevy prototype)".into(),
                resolution: (300., 300.).into(),
                resizable: false,
                decorations: false,
                transparent: true,
                ..Default::default()
            }),
            ..default()
        }))
        .insert_resource(Paused(false))
        .add_startup_system(setup)
        .add_system(spin_sprite)
        .add_system(right_click_menu)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    // A simple colored square as a sprite (use a built-in image if available)
    let texture_handle = asset_server.load("branding/icon.png");
    commands.spawn(SpriteBundle {
        texture: texture_handle,
        transform: Transform::from_scale(Vec3::splat(0.5)),
        ..Default::default()
    });
}

fn spin_sprite(time: Res<Time>, paused: Res<Paused>, mut query: Query<&mut Transform, With<Sprite>>) {
    if paused.0 {
        return;
    }
    for mut t in query.iter_mut() {
        t.rotate_z(1.0 * time.delta_seconds());
    }
}

fn right_click_menu(
    mouse_buttons: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
    mut paused: ResMut<Paused>,
    mut exit: EventWriter<AppExit>,
) {
    if mouse_buttons.just_pressed(MouseButton::Right) {
        if keyboard.pressed(KeyCode::LShift) || keyboard.pressed(KeyCode::RShift) {
            // Shift + Right => Quit
            println!("Bevy prototype: Quit requested (Shift+Right)");
            exit.send(AppExit);
        } else {
            // Right-click toggles pause
            paused.0 = !paused.0;
            println!("Bevy prototype: paused = {}", paused.0);
        }
    }
}
