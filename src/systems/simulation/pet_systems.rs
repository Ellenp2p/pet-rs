use crate::components::*;
use bevy::prelude::*;

pub fn hunger_decay_system(time: Res<Time>, mut pet_query: Query<&mut Hunger, With<Pet>>) {
    for mut hunger in pet_query.iter_mut() {
        hunger.value -= hunger.decay_rate * time.delta_seconds();
        hunger.clamp();
    }
}

pub fn health_from_hunger_system(
    mut pet_query: Query<(&Hunger, &mut Health), (With<Pet>, Changed<Hunger>)>,
) {
    for (hunger, mut health) in pet_query.iter_mut() {
        if hunger.value <= 0.0 {
            health.value -= 2.0 * hunger.decay_rate;
            health.clamp();
        } else if hunger.ratio() < 0.2 {
            health.value -= 0.5 * hunger.decay_rate;
            health.clamp();
        }
    }
}
