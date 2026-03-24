use crate::error::FrameworkError;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Network configuration.
#[derive(Clone)]
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

/// Generic bidirectional channel for bridging async network tasks.
///
/// ## Usage
///
/// - `send()` / `inject_incoming()` — safe to call from any context (async or sync).
/// - `drain_outgoing()` / `drain_incoming()` — **only call from synchronous systems**.
///   These methods use `std::sync::Mutex` with very short critical sections (just `try_recv`).
///   Do NOT call them across an `.await`.
///
/// ## Design
///
/// Uses `tokio::sync::mpsc` for the channel (async-safe producer side) and
/// `std::sync::Mutex` to guard the receiver (sync consumer side). This avoids
/// the overhead of `tokio::sync::Mutex` while keeping the producer side usable
/// from tokio tasks.
pub struct NetworkChannel<T>
where
    T: Send + Sync + 'static,
{
    outgoing_tx: Arc<mpsc::UnboundedSender<T>>,
    outgoing_rx: Arc<std::sync::Mutex<mpsc::UnboundedReceiver<T>>>,
    incoming_tx: Arc<mpsc::UnboundedSender<T>>,
    incoming_rx: Arc<std::sync::Mutex<mpsc::UnboundedReceiver<T>>>,
    _marker: PhantomData<T>,
}

impl<T> Default for NetworkChannel<T>
where
    T: Send + Sync + 'static,
{
    fn default() -> Self {
        let (out_tx, out_rx) = mpsc::unbounded_channel();
        let (in_tx, in_rx) = mpsc::unbounded_channel();
        Self {
            outgoing_tx: Arc::new(out_tx),
            outgoing_rx: Arc::new(std::sync::Mutex::new(out_rx)),
            incoming_tx: Arc::new(in_tx),
            incoming_rx: Arc::new(std::sync::Mutex::new(in_rx)),
            _marker: PhantomData,
        }
    }
}

impl<T> NetworkChannel<T>
where
    T: Send + Sync + 'static,
{
    /// Queue a message for outgoing transmission.
    ///
    /// Safe to call from both sync Bevy systems and async tokio tasks.
    pub fn send(&self, msg: T) -> Result<(), FrameworkError> {
        self.outgoing_tx
            .send(msg)
            .map_err(|e| FrameworkError::ChannelClosed(e.to_string()))
    }

    /// Drain all queued outgoing messages.
    ///
    /// **Must be called from a synchronous Bevy system only.**
    pub fn drain_outgoing(&self) -> Result<Vec<T>, FrameworkError> {
        let mut rx = self
            .outgoing_rx
            .lock()
            .map_err(|_| FrameworkError::LockPoisoned)?;
        let mut msgs = Vec::new();
        while let Ok(msg) = rx.try_recv() {
            msgs.push(msg);
        }
        Ok(msgs)
    }

    /// Inject an incoming message (e.g., received from a WebSocket or HTTP poll).
    ///
    /// Safe to call from both sync Bevy systems and async tokio tasks.
    pub fn inject_incoming(&self, msg: T) -> Result<(), FrameworkError> {
        self.incoming_tx
            .send(msg)
            .map_err(|e| FrameworkError::ChannelClosed(e.to_string()))
    }

    /// Drain all queued incoming messages.
    ///
    /// **Must be called from a synchronous Bevy system only.**
    pub fn drain_incoming(&self) -> Result<Vec<T>, FrameworkError> {
        let mut rx = self
            .incoming_rx
            .lock()
            .map_err(|_| FrameworkError::LockPoisoned)?;
        let mut msgs = Vec::new();
        while let Ok(msg) = rx.try_recv() {
            msgs.push(msg);
        }
        Ok(msgs)
    }
}
