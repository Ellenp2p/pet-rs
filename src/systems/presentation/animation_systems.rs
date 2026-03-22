use crate::components::*;
use crate::events::StateChangedEvent;
use bevy::prelude::*;

#[derive(Component, Debug, Clone, Reflect)]
pub struct PetAnimator {
    pub current_animation: String,
}

impl Default for PetAnimator {
    fn default() -> Self {
        Self {
            current_animation: String::from("idle"),
        }
    }
}

pub fn animation_state_system(
    mut anim_query: Query<(&mut PetAnimator, &PetState)>,
    mut state_events: EventReader<StateChangedEvent>,
) {
    for state_event in state_events.read() {
        if let Ok((mut animator, _state)) = anim_query.get_mut(state_event.entity) {
            let anim_name = match state_event.new_state {
                PetState::Idle => "idle",
                PetState::Hungry => "hungry",
                PetState::Eating => "eating",
                PetState::Sick => "sick",
                PetState::Dead => "dead",
            };
            animator.current_animation = anim_name.to_string();
            info!(
                "Pet {:?} animation set to '{}'",
                state_event.entity, anim_name
            );
        }
    }
}

pub struct PetAnimationPlugin;

impl Plugin for PetAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PetAnimator>()
            .add_systems(Update, animation_state_system);
    }
}
