use crate::components::PetState;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Event, Debug, Clone)]
pub struct FeedEvent {
    pub entity: Entity,
    pub amount: f32,
}

#[derive(Event, Debug, Clone)]
pub struct StateChangedEvent {
    pub entity: Entity,
    pub old_state: PetState,
    pub new_state: PetState,
}

#[derive(Event, Debug, Clone, Serialize, Deserialize)]
pub struct ExternalSyncEvent {
    pub pet_id: u64,
    pub hunger: f32,
    pub health: f32,
}

#[derive(Event, Debug, Clone)]
pub struct UploadPetEvent {
    pub entity: Entity,
}

#[derive(Event, Debug, Clone)]
pub struct SpawnPetEvent {
    pub name: String,
    pub network_id: Option<u64>,
}

#[derive(Event, Debug, Clone)]
pub struct HealEvent {
    pub entity: Entity,
    pub amount: f32,
}
