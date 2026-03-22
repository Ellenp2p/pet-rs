use super::dto::PetStateDto;
use super::NetworkConfig;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

pub type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub async fn connect_ws(
    config: &NetworkConfig,
) -> Result<WsStream, String> {
    let ws_url = config.server_url.replace("http://", "ws://").replace("https://", "wss://");
    let url = format!("{}/ws", ws_url);
    let (ws_stream, _) = connect_async(&url)
        .await
        .map_err(|e| format!("WebSocket connect failed: {}", e))?;
    Ok(ws_stream)
}

pub async fn send_ws_message(
    ws: &mut WsStream,
    dto: &PetStateDto,
) -> Result<(), String> {
    let json = serde_json::to_string(dto).map_err(|e| format!("Serialize failed: {}", e))?;
    ws.send(Message::Text(json))
        .await
        .map_err(|e| format!("WS send failed: {}", e))?;
    Ok(())
}

pub async fn receive_ws_message(
    ws: &mut WsStream,
) -> Result<Option<PetStateDto>, String> {
    if let Some(msg) = ws.next().await {
        let msg = msg.map_err(|e| format!("WS receive failed: {}", e))?;
        match msg {
            Message::Text(text) => {
                let dto: PetStateDto = serde_json::from_str(&text)
                    .map_err(|e| format!("JSON parse failed: {}", e))?;
                Ok(Some(dto))
            }
            _ => Ok(None),
        }
    } else {
        Ok(None)
    }
}
