pub mod dto;
pub mod http_client;
pub mod websocket;

pub use dto::*;

use bevy::prelude::*;
use tokio::sync::mpsc;
use std::sync::Arc;

#[derive(Resource, Clone)]
pub struct NetworkConfig {
    pub server_url: String,
    pub poll_interval_secs: f32,
    pub use_websocket: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            server_url: String::from("http://localhost:3000"),
            poll_interval_secs: 5.0,
            use_websocket: false,
        }
    }
}

#[derive(Resource)]
pub struct NetworkChannel {
    tx: Arc<mpsc::UnboundedSender<PetStateDto>>,
    rx: Arc<std::sync::Mutex<mpsc::UnboundedReceiver<PetStateDto>>>,
    incoming_tx: Arc<mpsc::UnboundedSender<PetStateDto>>,
    incoming_rx: Arc<std::sync::Mutex<mpsc::UnboundedReceiver<PetStateDto>>>,
}

impl Default for NetworkChannel {
    fn default() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let (incoming_tx, incoming_rx) = mpsc::unbounded_channel();
        Self {
            tx: Arc::new(tx),
            rx: Arc::new(std::sync::Mutex::new(rx)),
            incoming_tx: Arc::new(incoming_tx),
            incoming_rx: Arc::new(std::sync::Mutex::new(incoming_rx)),
        }
    }
}

impl NetworkChannel {
    pub fn send_update(&self, dto: PetStateDto) -> Result<(), String> {
        self.tx
            .send(dto)
            .map_err(|e| format!("Failed to send: {}", e))
    }

    pub fn receive_updates(&self) -> Vec<PetStateDto> {
        let mut rx = self.rx.lock().unwrap();
        let mut updates = Vec::new();
        while let Ok(dto) = rx.try_recv() {
            updates.push(dto);
        }
        updates
    }

    pub fn inject_incoming(&self, dto: PetStateDto) -> Result<(), String> {
        self.incoming_tx
            .send(dto)
            .map_err(|e| format!("Failed to inject: {}", e))
    }

    pub fn receive_incoming(&self) -> Vec<PetStateDto> {
        let mut rx = self.incoming_rx.lock().unwrap();
        let mut updates = Vec::new();
        while let Ok(dto) = rx.try_recv() {
            updates.push(dto);
        }
        updates
    }
}
