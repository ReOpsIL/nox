//! Message broker module for the Nox agent ecosystem
//! 
//! This module implements a message broker system for complex agent collaboration and discovery.

use anyhow::{Result, anyhow};
use log::{info, warn, error, debug};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::types::Agent;

// Singleton instance of the message broker
lazy_static::lazy_static! {
    static ref MESSAGE_BROKER: Arc<MessageBroker> = Arc::new(MessageBroker::new());
}

/// Message priority enum
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    /// Low priority message
    Low = 0,
    /// Normal priority message
    Normal = 1,
    /// High priority message
    High = 2,
    /// Critical priority message
    Critical = 3,
}

/// Message status enum
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageStatus {
    /// Message is pending delivery
    Pending,
    /// Message has been delivered
    Delivered,
    /// Message has been read
    Read,
    /// Message has been processed
    Processed,
    /// Message delivery failed
    Failed,
}

/// Message struct representing a message between agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique identifier for the message
    pub id: String,
    /// ID of the sender agent
    pub sender_id: String,
    /// ID of the recipient agent
    pub recipient_id: String,
    /// Message subject
    pub subject: String,
    /// Message content
    pub content: String,
    /// Message priority
    pub priority: MessagePriority,
    /// Message status
    pub status: MessageStatus,
    /// When the message was created
    pub created_at: DateTime<Utc>,
    /// When the message was delivered (if applicable)
    pub delivered_at: Option<DateTime<Utc>>,
    /// When the message was read (if applicable)
    pub read_at: Option<DateTime<Utc>>,
    /// Message metadata
    pub metadata: HashMap<String, String>,
}

impl Message {
    /// Create a new message
    pub fn new(sender_id: String, recipient_id: String, subject: String, content: String) -> Self {
        let id = format!("msg-{}", Uuid::new_v4().as_u128());
        Self {
            id,
            sender_id,
            recipient_id,
            subject,
            content,
            priority: MessagePriority::Normal,
            status: MessageStatus::Pending,
            created_at: Utc::now(),
            delivered_at: None,
            read_at: None,
            metadata: HashMap::new(),
        }
    }

    /// Set the message priority
    pub fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.priority = priority;
        self
    }

    /// Add metadata to the message
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// Topic struct representing a message topic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topic {
    /// Unique identifier for the topic
    pub id: String,
    /// Name of the topic
    pub name: String,
    /// Description of the topic
    pub description: String,
    /// List of agent IDs subscribed to this topic
    pub subscribers: Vec<String>,
    /// When the topic was created
    pub created_at: DateTime<Utc>,
}

impl Topic {
    /// Create a new topic
    pub fn new(name: String, description: String) -> Self {
        let id = format!("topic-{}", Uuid::new_v4().as_u128());
        Self {
            id,
            name,
            description,
            subscribers: Vec::new(),
            created_at: Utc::now(),
        }
    }
}

/// Message queue for an agent
struct AgentQueue {
    /// Queue of messages for the agent
    messages: VecDeque<Message>,
    /// Last time the agent checked for messages
    last_check: DateTime<Utc>,
}

impl AgentQueue {
    /// Create a new agent queue
    fn new() -> Self {
        Self {
            messages: VecDeque::new(),
            last_check: Utc::now(),
        }
    }

    /// Add a message to the queue
    fn add_message(&mut self, message: Message) {
        // Insert the message in the correct position based on priority
        let priority = message.priority as usize;
        let position = self.messages.iter()
            .position(|m| (m.priority as usize) < priority)
            .unwrap_or(self.messages.len());
        
        self.messages.insert(position, message);
    }

    /// Get the next message from the queue
    fn get_next_message(&mut self) -> Option<Message> {
        self.messages.pop_front()
    }

    /// Get all messages from the queue
    fn get_all_messages(&mut self) -> Vec<Message> {
        let messages: Vec<Message> = self.messages.drain(..).collect();
        self.last_check = Utc::now();
        messages
    }

    /// Update the last check time
    fn update_last_check(&mut self) {
        self.last_check = Utc::now();
    }
}

/// Message broker struct
pub struct MessageBroker {
    /// Map of agent IDs to message queues
    agent_queues: RwLock<HashMap<String, Arc<Mutex<AgentQueue>>>>,
    /// Map of topic IDs to topics
    topics: RwLock<HashMap<String, Topic>>,
}

impl MessageBroker {
    /// Create a new message broker
    fn new() -> Self {
        Self {
            agent_queues: RwLock::new(HashMap::new()),
            topics: RwLock::new(HashMap::new()),
        }
    }

    /// Register an agent with the message broker
    async fn register_agent(&self, agent_id: &str) -> Result<()> {
        let mut agent_queues = self.agent_queues.write().await;
        if !agent_queues.contains_key(agent_id) {
            agent_queues.insert(agent_id.to_string(), Arc::new(Mutex::new(AgentQueue::new())));
            info!("Agent registered with message broker: {}", agent_id);
        }
        Ok(())
    }

    /// Unregister an agent from the message broker
    async fn unregister_agent(&self, agent_id: &str) -> Result<()> {
        let mut agent_queues = self.agent_queues.write().await;
        if agent_queues.remove(agent_id).is_some() {
            info!("Agent unregistered from message broker: {}", agent_id);
        }
        Ok(())
    }

    /// Send a message to an agent
    async fn send_message(&self, message: Message) -> Result<()> {
        // Ensure both sender and recipient are registered
        let agent_queues = self.agent_queues.read().await;
        
        if !agent_queues.contains_key(&message.sender_id) {
            return Err(anyhow!("Sender agent not registered: {}", message.sender_id));
        }
        
        if !agent_queues.contains_key(&message.recipient_id) {
            return Err(anyhow!("Recipient agent not registered: {}", message.recipient_id));
        }
        
        // Get the recipient's queue
        let queue = agent_queues.get(&message.recipient_id).unwrap().clone();
        let mut queue = queue.lock().await;
        
        // Add the message to the queue
        debug!("Sending message from {} to {}: {}", 
               message.sender_id, message.recipient_id, message.subject);
        queue.add_message(message);
        
        Ok(())
    }

    /// Get messages for an agent
    async fn get_messages(&self, agent_id: &str, max_count: usize) -> Result<Vec<Message>> {
        let agent_queues = self.agent_queues.read().await;
        
        if let Some(queue) = agent_queues.get(agent_id) {
            let mut queue = queue.lock().await;
            
            // Update the last check time
            queue.update_last_check();
            
            // Get up to max_count messages
            let mut messages = Vec::new();
            for _ in 0..max_count {
                if let Some(message) = queue.get_next_message() {
                    messages.push(message);
                } else {
                    break;
                }
            }
            
            Ok(messages)
        } else {
            Err(anyhow!("Agent not registered: {}", agent_id))
        }
    }

    /// Get all messages for an agent
    async fn get_all_messages(&self, agent_id: &str) -> Result<Vec<Message>> {
        let agent_queues = self.agent_queues.read().await;
        
        if let Some(queue) = agent_queues.get(agent_id) {
            let mut queue = queue.lock().await;
            Ok(queue.get_all_messages())
        } else {
            Err(anyhow!("Agent not registered: {}", agent_id))
        }
    }

    /// Create a new topic
    async fn create_topic(&self, name: String, description: String) -> Result<Topic> {
        let topic = Topic::new(name, description);
        let mut topics = self.topics.write().await;
        
        topics.insert(topic.id.clone(), topic.clone());
        info!("Created topic: {}", topic.name);
        
        Ok(topic)
    }

    /// Get a topic by ID
    async fn get_topic(&self, topic_id: &str) -> Result<Topic> {
        let topics = self.topics.read().await;
        
        if let Some(topic) = topics.get(topic_id) {
            Ok(topic.clone())
        } else {
            Err(anyhow!("Topic not found: {}", topic_id))
        }
    }

    /// Get all topics
    async fn get_all_topics(&self) -> Result<Vec<Topic>> {
        let topics = self.topics.read().await;
        Ok(topics.values().cloned().collect())
    }

    /// Subscribe an agent to a topic
    async fn subscribe_to_topic(&self, agent_id: &str, topic_id: &str) -> Result<()> {
        // Ensure the agent is registered
        let agent_queues = self.agent_queues.read().await;
        if !agent_queues.contains_key(agent_id) {
            return Err(anyhow!("Agent not registered: {}", agent_id));
        }
        
        // Get the topic
        let mut topics = self.topics.write().await;
        if let Some(topic) = topics.get_mut(topic_id) {
            if !topic.subscribers.contains(&agent_id.to_string()) {
                topic.subscribers.push(agent_id.to_string());
                info!("Agent {} subscribed to topic {}", agent_id, topic.name);
            }
            Ok(())
        } else {
            Err(anyhow!("Topic not found: {}", topic_id))
        }
    }

    /// Unsubscribe an agent from a topic
    async fn unsubscribe_from_topic(&self, agent_id: &str, topic_id: &str) -> Result<()> {
        let mut topics = self.topics.write().await;
        
        if let Some(topic) = topics.get_mut(topic_id) {
            topic.subscribers.retain(|id| id != agent_id);
            info!("Agent {} unsubscribed from topic {}", agent_id, topic.name);
            Ok(())
        } else {
            Err(anyhow!("Topic not found: {}", topic_id))
        }
    }

    /// Publish a message to a topic
    async fn publish_to_topic(&self, sender_id: &str, topic_id: &str, subject: String, content: String) -> Result<()> {
        // Get the topic
        let topics = self.topics.read().await;
        let topic = topics.get(topic_id).ok_or_else(|| anyhow!("Topic not found: {}", topic_id))?;
        
        // Send the message to all subscribers
        for subscriber_id in &topic.subscribers {
            if subscriber_id != sender_id {  // Don't send to the sender
                let message = Message::new(
                    sender_id.to_string(),
                    subscriber_id.clone(),
                    subject.clone(),
                    content.clone(),
                ).with_metadata("topic_id", topic_id)
                 .with_metadata("topic_name", &topic.name);
                
                self.send_message(message).await?;
            }
        }
        
        info!("Published message to topic {} with {} subscribers", topic.name, topic.subscribers.len());
        Ok(())
    }
}

/// Register an agent with the message broker
pub async fn register_agent(agent_id: &str) -> Result<()> {
    MESSAGE_BROKER.register_agent(agent_id).await
}

/// Unregister an agent from the message broker
pub async fn unregister_agent(agent_id: &str) -> Result<()> {
    MESSAGE_BROKER.unregister_agent(agent_id).await
}

/// Send a message to an agent
pub async fn send_message(message: Message) -> Result<()> {
    MESSAGE_BROKER.send_message(message).await
}

/// Get messages for an agent
pub async fn get_messages(agent_id: &str, max_count: usize) -> Result<Vec<Message>> {
    MESSAGE_BROKER.get_messages(agent_id, max_count).await
}

/// Get all messages for an agent
pub async fn get_all_messages(agent_id: &str) -> Result<Vec<Message>> {
    MESSAGE_BROKER.get_all_messages(agent_id).await
}

/// Create a new message
pub fn create_message(sender_id: &str, recipient_id: &str, subject: &str, content: &str) -> Message {
    Message::new(
        sender_id.to_string(),
        recipient_id.to_string(),
        subject.to_string(),
        content.to_string(),
    )
}

/// Create a new topic
pub async fn create_topic(name: &str, description: &str) -> Result<Topic> {
    MESSAGE_BROKER.create_topic(name.to_string(), description.to_string()).await
}

/// Get a topic by ID
pub async fn get_topic(topic_id: &str) -> Result<Topic> {
    MESSAGE_BROKER.get_topic(topic_id).await
}

/// Get all topics
pub async fn get_all_topics() -> Result<Vec<Topic>> {
    MESSAGE_BROKER.get_all_topics().await
}

/// Subscribe an agent to a topic
pub async fn subscribe_to_topic(agent_id: &str, topic_id: &str) -> Result<()> {
    MESSAGE_BROKER.subscribe_to_topic(agent_id, topic_id).await
}

/// Unsubscribe an agent from a topic
pub async fn unsubscribe_from_topic(agent_id: &str, topic_id: &str) -> Result<()> {
    MESSAGE_BROKER.unsubscribe_from_topic(agent_id, topic_id).await
}

/// Publish a message to a topic
pub async fn publish_to_topic(sender_id: &str, topic_id: &str, subject: &str, content: &str) -> Result<()> {
    MESSAGE_BROKER.publish_to_topic(sender_id, topic_id, subject.to_string(), content.to_string()).await
}