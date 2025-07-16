use crate::types::{Agent, Task, SystemStatus};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum AppEvent {
    Key(KeyEvent),
    Tick,
    AgentUpdated(Agent),
    TaskUpdated(Task),
    SystemStatusUpdated(SystemStatus),
    Quit,
}

pub struct EventHandler {
    sender: mpsc::UnboundedSender<AppEvent>,
    receiver: mpsc::UnboundedReceiver<AppEvent>,
    tick_rate: Duration,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        Self {
            sender,
            receiver,
            tick_rate,
        }
    }

    pub fn sender(&self) -> mpsc::UnboundedSender<AppEvent> {
        self.sender.clone()
    }

    pub async fn next(&mut self) -> Option<AppEvent> {
        self.receiver.recv().await
    }

    pub async fn start_event_loop(&self) -> Result<()> {
        let sender = self.sender.clone();
        let tick_rate = self.tick_rate;

        tokio::spawn(async move {
            let mut last_tick = Instant::now();

            loop {
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));

                if event::poll(timeout).unwrap_or(false) {
                    match event::read() {
                        Ok(Event::Key(key)) => {
                            if key.kind == crossterm::event::KeyEventKind::Press {
                                if let Err(_) = sender.send(AppEvent::Key(key)) {
                                    break;
                                }
                            }
                        }
                        _ => {}
                    }
                }

                if last_tick.elapsed() >= tick_rate {
                    if let Err(_) = sender.send(AppEvent::Tick) {
                        break;
                    }
                    last_tick = Instant::now();
                }
            }
        });

        Ok(())
    }
}

impl AppEvent {
    pub fn is_quit_key(key: &KeyEvent) -> bool {
        matches!(
            key,
            KeyEvent {
                code: KeyCode::Char('q') | KeyCode::Char('Q'),
                modifiers: KeyModifiers::NONE,
                ..
            } | KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }
        )
    }
}