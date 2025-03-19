use ratatui::crossterm::event::{self, Event};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

/// Event handler for handling terminal events
pub struct EventHandler {
    /// Event receiver channel
    rx: mpsc::Receiver<Event>,
    /// Event polling interval
    tick_rate: Duration,
    /// Last poll time
    last_tick: Instant,
}

impl EventHandler {
    /// Create a new event handler with the given tick rate
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::channel();
        
        // Spawn a thread to poll for events
        thread::spawn(move || {
            loop {
                // Poll for events and send them through the channel
                if event::poll(tick_rate).unwrap() {
                    if let Ok(event) = event::read() {
                        if let Err(_) = tx.send(event) {
                            break;
                        }
                    }
                }
                
                // Check if the receiver is dropped (if we can't send, it means the receiver is gone)
                if tx.send(Event::FocusGained).is_err() {
                    break;
                }
            }
        });

        Self {
            rx,
            tick_rate,
            last_tick: Instant::now(),
        }
    }

    /// Get the next event
    pub fn next(&mut self) -> Result<Event, mpsc::RecvError> {
        // First check if we have any events in the channel
        if let Ok(event) = self.rx.try_recv() {
            return Ok(event);
        }
        
        // If not, check if we should tick
        if self.last_tick.elapsed() >= self.tick_rate {
            self.last_tick = Instant::now();
            // Return an empty tick event - not needed for our simple app yet
        }
        
        // Wait for the next event
        self.rx.recv()
    }
}