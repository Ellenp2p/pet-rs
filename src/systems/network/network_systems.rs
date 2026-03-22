use crate::components::*;
use crate::events::*;
use crate::hooks::{HookContext, HookRegistry, HookType};
use crate::network::NetworkChannel;
use bevy::prelude::*;

pub fn apply_external_sync_system(
    mut sync_events: EventReader<ExternalSyncEvent>,
    mut pet_query: Query<(Entity, &mut Hunger, &mut Health, Option<&NetworkId>), With<Pet>>,
) {
    for event in sync_events.read() {
        for (_entity, mut hunger, mut health, net_id) in pet_query.iter_mut() {
            if let Some(nid) = net_id {
                if nid.id == event.pet_id {
                    hunger.value = event.hunger.clamp(0.0, hunger.max);
                    health.value = event.health.clamp(0.0, health.max);
                    info!(
                        "Synced pet {} from server: hunger={:.1}, health={:.1}",
                        event.pet_id, hunger.value, health.value
                    );
                }
            }
        }
    }
}

pub fn detect_changes_system(
    pet_query: Query<
        (Entity, Option<&NetworkId>),
        (With<Pet>, Or<(Changed<Hunger>, Changed<Health>)>),
    >,
    mut upload_events: EventWriter<UploadPetEvent>,
) {
    for (entity, _net_id) in pet_query.iter() {
        upload_events.send(UploadPetEvent { entity });
    }
}

pub fn process_upload_queue_system(
    mut upload_events: EventReader<UploadPetEvent>,
    pet_query: Query<(&Hunger, &Health, Option<&NetworkId>), With<Pet>>,
    network_channel: Res<NetworkChannel>,
    hooks: Res<HookRegistry>,
) {
    for event in upload_events.read() {
        if let Ok((hunger, health, net_id)) = pet_query.get(event.entity) {
            let dto = crate::network::PetStateDto {
                id: net_id.map(|n| n.id).unwrap_or(0),
                hunger: hunger.value,
                health: health.value,
            };

            let _ = network_channel.send_update(dto);

            if let Some(nid) = net_id {
                let ctx = HookContext {
                    entity: event.entity,
                    pet_id: Some(nid.id),
                };
                hooks.trigger(HookType::OnNetworkSync, &ctx);
            }
        }
    }
}
