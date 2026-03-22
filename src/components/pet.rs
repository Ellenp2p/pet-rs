use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, Reflect)]
pub struct Pet;

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize, Reflect)]
pub struct Hunger {
    pub value: f32,
    pub max: f32,
    pub decay_rate: f32,
}

impl Default for Hunger {
    fn default() -> Self {
        Self {
            value: 100.0,
            max: 100.0,
            decay_rate: 1.0,
        }
    }
}

impl Hunger {
    pub fn clamp(&mut self) {
        self.value = self.value.clamp(0.0, self.max);
    }

    pub fn ratio(&self) -> f32 {
        if self.max <= 0.0 {
            0.0
        } else {
            self.value / self.max
        }
    }
}

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize, Reflect)]
pub struct Health {
    pub value: f32,
    pub max: f32,
}

impl Default for Health {
    fn default() -> Self {
        Self {
            value: 100.0,
            max: 100.0,
        }
    }
}

impl Health {
    pub fn clamp(&mut self) {
        self.value = self.value.clamp(0.0, self.max);
    }

    pub fn ratio(&self) -> f32 {
        if self.max <= 0.0 {
            0.0
        } else {
            self.value / self.max
        }
    }
}

#[derive(
    Component, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Reflect, Default,
)]
pub enum Mood {
    Happy,
    #[default]
    Neutral,
    Sad,
    Sick,
    Dead,
}

#[derive(
    Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect, Default,
)]
pub enum PetState {
    #[default]
    Idle,
    Hungry,
    Eating,
    Sick,
    Dead,
}

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize, Reflect)]
pub struct NetworkId {
    pub id: u64,
}

impl NetworkId {
    pub fn new(id: u64) -> Self {
        Self { id }
    }
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct PetName(pub String);

impl Default for PetName {
    fn default() -> Self {
        Self(String::from("Unnamed Pet"))
    }
}
