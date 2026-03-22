use bevy::prelude::*;
use pet_rs::prelude::*;

fn main() {
    configure_backend();
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PetFrameworkPlugin)
        .add_plugins(PetAnimationPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (keyboard_input, update_ui))
        .run();
}

fn setup(mut commands: Commands, mut spawn_events: EventWriter<SpawnPetEvent>) {
    commands.spawn(Camera2dBundle::default());

    spawn_events.send(SpawnPetEvent {
        name: String::from("Buddy"),
        network_id: None,
    });

    commands.spawn(
        TextBundle::from_section(
            "Loading...",
            TextStyle {
                font_size: 28.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(30.0),
            left: Val::Px(30.0),
            ..default()
        }),
    );
}

fn keyboard_input(
    keys: Res<ButtonInput<KeyCode>>,
    pet_query: Query<Entity, With<Pet>>,
    mut feed_events: EventWriter<FeedEvent>,
    mut heal_events: EventWriter<HealEvent>,
) {
    for entity in pet_query.iter() {
        if keys.just_pressed(KeyCode::KeyF) {
            feed_events.send(FeedEvent {
                entity,
                amount: 20.0,
            });
        }
        if keys.just_pressed(KeyCode::KeyH) {
            heal_events.send(HealEvent {
                entity,
                amount: 15.0,
            });
        }
    }
}

fn update_ui(
    pet_query: Query<(&PetName, &Hunger, &Health, &PetState, &Mood), With<Pet>>,
    mut text_query: Query<&mut Text>,
) {
    for (name, hunger, health, state, mood) in pet_query.iter() {
        for mut text in text_query.iter_mut() {
            let lines = vec![
                "===========================".into(),
                String::new(),
                format!("      {}", name.0),
                String::new(),
                "===========================".into(),
                String::new(),
                format!("  Hunger: {:.0} / {:.0}", hunger.value, hunger.max),
                format!("  Health: {:.0} / {:.0}", health.value, health.max),
                format!("  State:  {:?}", state),
                format!("  Mood:   {:?}", mood),
                String::new(),
                "  [F] Feed  |  [H] Heal".into(),
                "===========================".into(),
            ];
            text.sections[0].value = lines.join("\n");
        }
    }
}
