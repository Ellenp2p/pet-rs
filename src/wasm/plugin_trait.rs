pub trait WasmPetPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn on_tick(&self, pet_id: u64);
    fn on_feed(&self, pet_id: u64, amount: f32);
    fn on_state_change(&self, pet_id: u64, old_state: &str, new_state: &str);
    fn on_network_sync(&self, pet_id: u64);
}
