use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PetStateDto {
    pub id: u64,
    pub hunger: f32,
    pub health: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerResponse {
    pub pets: Vec<PetStateDto>,
    pub timestamp: u64,
}
