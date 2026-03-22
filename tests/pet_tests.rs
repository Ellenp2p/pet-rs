use bevy::prelude::*;
use pet_rs::prelude::*;

#[cfg(test)]
mod component_tests {
    use super::*;

    #[test]
    fn test_hunger_default() {
        let hunger = Hunger::default();
        assert_eq!(hunger.value, 100.0);
        assert_eq!(hunger.max, 100.0);
        assert!(hunger.ratio() > 0.99);
    }

    #[test]
    fn test_hunger_clamp() {
        let mut hunger = Hunger {
            value: 150.0,
            max: 100.0,
            decay_rate: 1.0,
        };
        hunger.clamp();
        assert_eq!(hunger.value, 100.0);

        hunger.value = -10.0;
        hunger.clamp();
        assert_eq!(hunger.value, 0.0);
    }

    #[test]
    fn test_health_ratio() {
        let health = Health {
            value: 50.0,
            max: 100.0,
        };
        assert!((health.ratio() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_pet_state_variants() {
        assert_eq!(PetState::default(), PetState::Idle);
        assert_ne!(PetState::Idle, PetState::Dead);
    }

    #[test]
    fn test_network_id() {
        let nid = NetworkId::new(42);
        assert_eq!(nid.id, 42);
    }
}

#[cfg(test)]
mod hook_tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_hook_registration() {
        let registry = HookRegistry::default();
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        registry.register_fn(HookType::OnFeed, move |_ctx| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        let ctx = HookContext {
            entity: Entity::from_raw(0),
            pet_id: None,
        };
        registry.trigger(HookType::OnFeed, &ctx);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_multiple_hooks() {
        let registry = HookRegistry::default();
        let counter = Arc::new(AtomicU32::new(0));

        for _ in 0..3 {
            let c = counter.clone();
            registry.register_fn(HookType::OnTick, move |_ctx| {
                c.fetch_add(1, Ordering::SeqCst);
            });
        }

        let ctx = HookContext {
            entity: Entity::from_raw(0),
            pet_id: None,
        };
        registry.trigger(HookType::OnTick, &ctx);
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }
}

#[cfg(test)]
mod event_tests {
    use super::*;

    #[test]
    fn test_feed_event_creation() {
        let entity = Entity::from_raw(1);
        let event = FeedEvent {
            entity,
            amount: 25.0,
        };
        assert_eq!(event.amount, 25.0);
    }

    #[test]
    fn test_state_changed_event() {
        let entity = Entity::from_raw(1);
        let event = StateChangedEvent {
            entity,
            old_state: PetState::Idle,
            new_state: PetState::Hungry,
        };
        assert_ne!(event.old_state, event.new_state);
    }
}

#[cfg(test)]
mod dto_tests {
    use pet_rs::network::PetStateDto;

    #[test]
    fn test_dto_serialization() {
        let dto = PetStateDto {
            id: 1,
            hunger: 75.0,
            health: 90.0,
        };
        let json = serde_json::to_string(&dto).unwrap();
        let parsed: PetStateDto = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, 1);
        assert!((parsed.hunger - 75.0).abs() < f32::EPSILON);
    }
}

#[cfg(test)]
mod network_channel_tests {
    use pet_rs::network::{NetworkChannel, PetStateDto};

    #[test]
    fn test_channel_send_receive() {
        let channel = NetworkChannel::default();
        let dto = PetStateDto {
            id: 1,
            hunger: 50.0,
            health: 80.0,
        };
        channel.send_update(dto).unwrap();
        let updates = channel.receive_updates();
        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0].id, 1);
    }

    #[test]
    fn test_channel_incoming() {
        let channel = NetworkChannel::default();
        let dto = PetStateDto {
            id: 2,
            hunger: 30.0,
            health: 60.0,
        };
        channel.inject_incoming(dto).unwrap();
        let incoming = channel.receive_incoming();
        assert_eq!(incoming.len(), 1);
        assert_eq!(incoming[0].id, 2);
    }
}
