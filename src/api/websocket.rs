//! WebSocket module for the Nox agent ecosystem
//! 
//! This module handles WebSocket connections and message broadcasting.

use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_ws::{Message, MessageStream, Session};
use futures::StreamExt;
use log::{info, warn, error, debug};
use serde::{Serialize, Deserialize};
use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, Mutex};
use tokio::time::sleep;
use crate::types::{Agent, AgentStatus, Task, TaskStatus};

// Constants for WebSocket stability
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(60);
const MAX_QUEUE_SIZE: usize = 1000;

// Singleton instance of the WebSocket manager
lazy_static::lazy_static! {
    static ref WEBSOCKET_MANAGER: Arc<WebSocketManager> = {
        let (tx, _) = broadcast::channel(MAX_QUEUE_SIZE);
        Arc::new(WebSocketManager::new(tx))
    };
}

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebSocketMessageType {
    /// Agent status update
    AgentStatus,
    /// Task update
    TaskUpdate,
    /// System event
    SystemEvent,
    /// Heartbeat message
    Heartbeat,
}

/// WebSocket message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    /// Message type
    pub message_type: WebSocketMessageType,
    /// Timestamp
    pub timestamp: String,
    /// Message payload
    pub payload: serde_json::Value,
}

/// WebSocket manager
pub struct WebSocketManager {
    /// Broadcast channel sender
    tx: broadcast::Sender<String>,
    /// Connected clients count
    clients: Arc<Mutex<usize>>,
}

impl WebSocketManager {
    /// Create a new WebSocket manager
    fn new(tx: broadcast::Sender<String>) -> Self {
        Self {
            tx,
            clients: Arc::new(Mutex::new(0)),
        }
    }

    /// Get the broadcast channel sender
    pub fn sender(&self) -> broadcast::Sender<String> {
        self.tx.clone()
    }

    /// Get the number of connected clients
    pub async fn client_count(&self) -> usize {
        *self.clients.lock().await
    }

    /// Increment the client count
    async fn increment_clients(&self) {
        let mut clients = self.clients.lock().await;
        *clients += 1;
        debug!("Client connected. Total clients: {}", *clients);
    }

    /// Decrement the client count
    async fn decrement_clients(&self) {
        let mut clients = self.clients.lock().await;
        if *clients > 0 {
            *clients -= 1;
        }
        debug!("Client disconnected. Total clients: {}", *clients);
    }
}

/// WebSocket connection handler
pub async fn ws_handler(
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    let (res, session, msg_stream) = actix_ws::handle(&req, stream)?;

    // Get the WebSocket manager
    let manager = WEBSOCKET_MANAGER.clone();
    
    // Increment the client count
    manager.increment_clients().await;
    
    // Spawn a task to handle the WebSocket connection
    actix_web::rt::spawn(ws_client(session, msg_stream, manager));

    Ok(res)
}

/// WebSocket client handler
async fn ws_client(
    mut session: Session,
    mut msg_stream: MessageStream,
    manager: Arc<WebSocketManager>,
) {
    // Subscribe to the broadcast channel
    let mut rx = manager.sender().subscribe();
    
    // Set up heartbeat
    let mut last_heartbeat = Instant::now();
    let mut heartbeat_interval = tokio::time::interval(HEARTBEAT_INTERVAL);
    
    // Message queue for handling backpressure
    let message_queue = Arc::new(Mutex::new(Vec::new()));
    let queue_processor = message_queue.clone();
    
    // Create a task to process the message queue
    let queue_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(100));
        
        loop {
            interval.tick().await;
            
            let mut queue = queue_processor.lock().await;
            if !queue.is_empty() {
                let msg = queue.remove(0);
                if session.send(Message::Text(msg.into())).await.is_err() {
                    break;
                }
            }
        }
    });

    // Create a task to forward broadcast messages to the WebSocket
    let broadcast_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let mut queue = message_queue.lock().await;
            
            // Handle backpressure by limiting queue size
            if queue.len() < MAX_QUEUE_SIZE {
                queue.push(msg);
            } else {
                warn!("Message queue full, dropping message");
            }
        }
    });

    // Handle incoming WebSocket messages
    while let Some(Ok(msg)) = msg_stream.next().await {
        match msg {
            Message::Text(text) => {
                debug!("Received message: {}", text);
                // Echo the message back for now
                if session.send(Message::Text(text)).await.is_err() {
                    break;
                }
            }
            Message::Ping(bytes) => {
                last_heartbeat = Instant::now();
                if session.send(Message::Pong(bytes)).await.is_err() {
                    break;
                }
            }
            Message::Pong(_) => {
                last_heartbeat = Instant::now();
            }
            Message::Close(reason) => {
                if let Some(reason) = reason {
                    debug!("Connection closed with code {}: {}", reason.code, reason.description);
                } else {
                    debug!("Connection closed");
                }
                break;
            }
            _ => {}
        }

        // Check for client timeout
        if Instant::now().duration_since(last_heartbeat) > CLIENT_TIMEOUT {
            warn!("Client timed out");
            break;
        }

        // Send heartbeat
        if heartbeat_interval.tick().await.is_elapsed() {
            debug!("Sending heartbeat");
            let heartbeat_msg = json!({
                "message_type": "Heartbeat",
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "payload": {
                    "status": "ok"
                }
            });
            
            if session.send(Message::Text(heartbeat_msg.to_string())).await.is_err() {
                break;
            }
            
            // Also check for client timeout
            if Instant::now().duration_since(last_heartbeat) > CLIENT_TIMEOUT {
                warn!("Client timed out");
                break;
            }
        }
    }

    // Cancel the broadcast task when the WebSocket connection is closed
    broadcast_task.abort();
    queue_task.abort();
    
    // Decrement the client count
    manager.decrement_clients().await;
}

/// Broadcast an agent status update
pub async fn broadcast_agent_status(
    agent: &Agent,
    previous_status: AgentStatus,
) -> Result<(), broadcast::error::SendError<String>> {
    let message = json!({
        "message_type": "AgentStatus",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "payload": {
            "agent_id": agent.id,
            "name": agent.name,
            "status": agent.status,
            "previous_status": previous_status
        }
    });

    WEBSOCKET_MANAGER.sender().send(message.to_string())
}

/// Broadcast a task update
pub async fn broadcast_task_update(
    task: &Task,
    previous_status: Option<TaskStatus>,
) -> Result<(), broadcast::error::SendError<String>> {
    let message = json!({
        "message_type": "TaskUpdate",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "payload": {
            "task_id": task.id,
            "agent_id": task.agent_id,
            "title": task.title,
            "status": task.status,
            "previous_status": previous_status,
            "progress": task.progress
        }
    });

    WEBSOCKET_MANAGER.sender().send(message.to_string())
}

/// Broadcast a system event
pub async fn broadcast_system_event(
    event_type: &str,
    details: serde_json::Value,
) -> Result<(), broadcast::error::SendError<String>> {
    let message = json!({
        "message_type": "SystemEvent",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "payload": {
            "event_type": event_type,
            "details": details
        }
    });

    WEBSOCKET_MANAGER.sender().send(message.to_string())
}

/// Configure the WebSocket routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/ws", web::get().to(ws_handler));
}