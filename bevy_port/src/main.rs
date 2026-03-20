use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "pet-rs (Bevy port)".into(),
                resolution: (320., 320.).into(),
                resizable: false,
                decorations: false,
                transparent: true,
                ..Default::default()
            }),
            ..default()
        }))
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    // This port will be extended to reproduce main.rs behavior (pixels/framebuffer, per-pixel hit-test,
    // native region update). For now it's a scaffold; I'll continue implementing features next.
}
