use crate::components::*;
use crate::events::StateChangedEvent;
use crate::hooks::{HookContext, HookRegistry, HookType};
use bevy::prelude::*;

pub fn derive_pet_state_system(
    mut pet_query: Query<
        (
            &Hunger,
            &Health,
            &mut PetState,
            &mut Mood,
            Option<&NetworkId>,
        ),
        With<Pet>,
    >,
) {
    for (hunger, health, mut state, mut mood, _net_id) in pet_query.iter_mut() {
        let new_state = if health.value <= 0.0 {
            PetState::Dead
        } else if health.ratio() < 0.25 {
            PetState::Sick
        } else if hunger.ratio() < 0.2 {
            PetState::Hungry
        } else {
            PetState::Idle
        };

        *state = new_state;

        *mood = match new_state {
            PetState::Dead => Mood::Dead,
            PetState::Sick => Mood::Sick,
            PetState::Hungry => Mood::Sad,
            PetState::Idle => {
                if hunger.ratio() > 0.6 && health.ratio() > 0.6 {
                    Mood::Happy
                } else {
                    Mood::Neutral
                }
            }
            PetState::Eating => Mood::Happy,
        };
    }
}

pub fn emit_state_change_system(
    pet_query: Query<(Entity, &PetState, Option<&NetworkId>), (With<Pet>, Changed<PetState>)>,
    mut state_events: EventWriter<StateChangedEvent>,
    hooks: Res<HookRegistry>,
    mut previous_states: Local<std::collections::HashMap<Entity, PetState>>,
) {
    for (entity, &new_state, net_id) in pet_query.iter() {
        let old_state = previous_states
            .get(&entity)
            .copied()
            .unwrap_or(PetState::Idle);

        if old_state != new_state {
            state_events.send(StateChangedEvent {
                entity,
                old_state,
                new_state,
            });

            let ctx = HookContext {
                entity,
                pet_id: net_id.map(|n| n.id),
            };
            hooks.trigger(HookType::OnStateChange, &ctx);

            info!(
                "Pet {:?} state changed: {:?} -> {:?}",
                entity, old_state, new_state
            );
        }

        previous_states.insert(entity, new_state);
    }
}
