use crate::components::*;
use crate::events::*;
use crate::hooks::{HookContext, HookRegistry, HookType};
use bevy::prelude::*;

pub fn spawn_pet_system(
    mut commands: Commands,
    mut spawn_events: EventReader<SpawnPetEvent>,
    hooks: Res<HookRegistry>,
) {
    for event in spawn_events.read() {
        let entity = commands
            .spawn((
                Pet,
                PetName(event.name.clone()),
                Hunger::default(),
                Health::default(),
                Mood::default(),
                PetState::default(),
            ))
            .id();

        if let Some(net_id) = event.network_id {
            commands.entity(entity).insert(NetworkId::new(net_id));
        }

        let ctx = HookContext {
            entity,
            pet_id: event.network_id,
        };
        hooks.trigger(HookType::OnPetSpawn, &ctx);

        info!("Spawned pet '{}' with entity {:?}", event.name, entity);
    }
}

pub fn feed_pet_system(
    mut feed_events: EventReader<FeedEvent>,
    mut pet_query: Query<(&mut Hunger, &mut PetState, Option<&NetworkId>), With<Pet>>,
    hooks: Res<HookRegistry>,
) {
    for event in feed_events.read() {
        if let Ok((mut hunger, _state, net_id)) = pet_query.get_mut(event.entity) {
            hunger.value = (hunger.value + event.amount).clamp(0.0, hunger.max);

            let ctx = HookContext {
                entity: event.entity,
                pet_id: net_id.map(|n| n.id),
            };
            hooks.trigger(HookType::OnFeed, &ctx);

            info!(
                "Fed pet {:?} with amount {:.1}, hunger now {:.1}",
                event.entity, event.amount, hunger.value
            );
        }
    }
}

pub fn heal_pet_system(
    mut heal_events: EventReader<HealEvent>,
    mut pet_query: Query<&mut Health, With<Pet>>,
) {
    for event in heal_events.read() {
        if let Ok(mut health) = pet_query.get_mut(event.entity) {
            health.value = (health.value + event.amount).clamp(0.0, health.max);
            info!(
                "Healed pet {:?} by {:.1}, health now {:.1}",
                event.entity, event.amount, health.value
            );
        }
    }
}
