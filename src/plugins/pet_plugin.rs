use bevy::prelude::*;

use crate::components::*;
use crate::events::*;
use crate::hooks::HookRegistry;
use crate::systems::{
    input::input_systems::*, simulation::pet_systems::*, state::state_systems::*,
};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PetSet {
    Input,
    Simulation,
    StateDerivation,
    Output,
}

pub struct PetPlugin;

impl Plugin for PetPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Pet>()
            .register_type::<Hunger>()
            .register_type::<Health>()
            .register_type::<Mood>()
            .register_type::<PetState>()
            .register_type::<NetworkId>()
            .register_type::<PetName>()
            .add_event::<FeedEvent>()
            .add_event::<StateChangedEvent>()
            .add_event::<SpawnPetEvent>()
            .add_event::<HealEvent>()
            .add_event::<ExternalSyncEvent>()
            .add_event::<UploadPetEvent>()
            .init_resource::<HookRegistry>()
            .configure_sets(
                Update,
                (
                    PetSet::Input,
                    PetSet::Simulation,
                    PetSet::StateDerivation,
                    PetSet::Output,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    spawn_pet_system.in_set(PetSet::Input),
                    feed_pet_system.in_set(PetSet::Input),
                    heal_pet_system.in_set(PetSet::Input),
                    hunger_decay_system.in_set(PetSet::Simulation),
                    health_from_hunger_system.in_set(PetSet::Simulation),
                    derive_pet_state_system.in_set(PetSet::StateDerivation),
                    emit_state_change_system.in_set(PetSet::StateDerivation),
                ),
            );
    }
}
