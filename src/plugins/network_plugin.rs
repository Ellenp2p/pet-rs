use bevy::prelude::*;

use crate::network::{NetworkChannel, NetworkConfig};
use crate::plugins::pet_plugin::PetSet;
use crate::systems::network::network_systems::{
    apply_external_sync_system, detect_changes_system, process_upload_queue_system,
};

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NetworkConfig::default())
            .init_resource::<NetworkChannel>()
            .add_systems(
                Update,
                (
                    apply_external_sync_system.in_set(PetSet::Simulation),
                    detect_changes_system.in_set(PetSet::Output),
                    process_upload_queue_system.in_set(PetSet::Output),
                ),
            );
    }
}
