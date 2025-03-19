use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;
use anyhow::{Result, Context};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher, WatcherKind};

/// File system events that we care about
#[derive(Debug, Clone)]
pub enum FileEvent {
    /// A file was created
    Created(PathBuf),
    /// A file was modified
    Modified(PathBuf),
    /// A file was deleted
    Deleted(PathBuf),
    /// A file was renamed (from, to)
    Renamed(PathBuf, PathBuf),
    /// An error occurred
    Error(String),
}

/// File system watcher that monitors directories for changes
pub struct FileSystemWatcher {
    /// The underlying watcher
    _watcher: RecommendedWatcher,
    /// Receiver for file system events
    receiver: Receiver<FileEvent>,
}

impl FileSystemWatcher {
    /// Create a new file system watcher for the given paths
    pub fn new<P: AsRef<Path>>(paths: &[P]) -> Result<Self> {
        // Create a channel to receive events
        let (tx, rx) = channel();
        
        // Create the event handler
        let event_handler = EventHandler::new(tx);
        
        // Create the watcher
        let mut watcher = notify::recommended_watcher(event_handler)
            .context("Failed to create file system watcher")?;
        
        // Watch each path
        for path in paths {
            watcher.watch(path.as_ref(), RecursiveMode::Recursive)
                .with_context(|| format!("Failed to watch path: {}", path.as_ref().display()))?;
        }
        
        Ok(Self {
            _watcher: watcher,
            receiver: rx,
        })
    }
    
    /// Check if there are any pending events
    pub fn has_events(&self) -> bool {
        self.receiver.try_recv().is_ok()
    }
    
    /// Get the next event (non-blocking)
    pub fn try_next_event(&self) -> Option<FileEvent> {
        match self.receiver.try_recv() {
            Ok(event) => Some(event),
            Err(_) => None,
        }
    }
    
    /// Get the next event with timeout
    pub fn next_event_timeout(&self, timeout: Duration) -> Option<FileEvent> {
        match self.receiver.recv_timeout(timeout) {
            Ok(event) => Some(event),
            Err(_) => None,
        }
    }
}

/// Handler for file system events
struct EventHandler {
    sender: Sender<FileEvent>,
}

impl EventHandler {
    /// Create a new event handler
    fn new(sender: Sender<FileEvent>) -> Self {
        Self { sender }
    }
    
    /// Convert notify events to our FileEvent type
    fn handle_event(&self, event: Event) {
        match event.kind {
            EventKind::Create(_) => {
                for path in event.paths {
                    let _ = self.sender.send(FileEvent::Created(path));
                }
            },
            EventKind::Modify(_) => {
                for path in event.paths {
                    let _ = self.sender.send(FileEvent::Modified(path));
                }
            },
            EventKind::Remove(_) => {
                for path in event.paths {
                    let _ = self.sender.send(FileEvent::Deleted(path));
                }
            },
            EventKind::Rename(rename) => {
                if event.paths.len() == 2 {
                    let from = event.paths[0].clone();
                    let to = event.paths[1].clone();
                    let _ = self.sender.send(FileEvent::Renamed(from, to));
                }
            },
            _ => {
                // Ignore other event types
            }
        }
    }
}

impl notify::EventHandler for EventHandler {
    fn handle_event(&mut self, event: notify::Result<Event>) {
        match event {
            Ok(event) => self.handle_event(event),
            Err(e) => {
                let _ = self.sender.send(FileEvent::Error(e.to_string()));
            }
        }
    }
}