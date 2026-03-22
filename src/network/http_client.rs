use super::dto::{PetStateDto, ServerResponse};
use super::NetworkConfig;

pub async fn poll_server(config: &NetworkConfig) -> Result<Vec<PetStateDto>, String> {
    let url = format!("{}/api/pets", config.server_url);
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let server_response: ServerResponse = response
        .json()
        .await
        .map_err(|e| format!("JSON parse failed: {}", e))?;

    Ok(server_response.pets)
}

pub async fn upload_pet_state(
    config: &NetworkConfig,
    dto: &PetStateDto,
) -> Result<(), String> {
    let url = format!("{}/api/pets/{}", config.server_url, dto.id);
    let client = reqwest::Client::new();
    client
        .put(&url)
        .json(dto)
        .send()
        .await
        .map_err(|e| format!("Upload failed: {}", e))?;

    Ok(())
}
