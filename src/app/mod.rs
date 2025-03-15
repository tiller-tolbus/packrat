pub mod state;
mod events;

use anyhow::Result;
use crossterm::event::{self, Event};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{self, Stdout};
use std::time::Duration;

use self::events::EventHandler;
use self::state::AppState;
use crate::explorer::Explorer;
use crate::ui::render;

/// Main application struct
pub struct App {
    /// Terminal backend
    terminal: Terminal<CrosstermBackend<Stdout>>,
    /// Application state
    state: AppState,
    /// Event handler
    events: EventHandler,
    /// File explorer
    explorer: Explorer,
}

impl App {
    /// Create a new application instance
    pub fn new() -> Result<Self> {
        // Setup terminal
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        stdout.execute(EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        // Create app components
        let state = AppState::default();
        let events = EventHandler::new(Duration::from_millis(100));
        let explorer = Explorer::new(".")?; // Start in current directory

        Ok(Self {
            terminal,
            state,
            events,
            explorer,
        })
    }

    /// Run the application main loop
    pub fn run(&mut self) -> Result<()> {
        // Main loop
        while !self.state.should_quit {
            // Draw the UI
            self.terminal.draw(|frame| {
                render(frame, &self.state, &self.explorer);
            })?;

            // Handle events
            if let Ok(event) = self.events.next() {
                if let Event::Key(key_event) = event {
                    self.handle_key_event(key_event);
                }
            }
        }

        // Cleanup terminal
        terminal::disable_raw_mode()?;
        self.terminal.backend_mut().execute(LeaveAlternateScreen)?;

        Ok(())
    }

    /// Handle key events
    fn handle_key_event(&mut self, event: event::KeyEvent) {
        use crossterm::event::KeyCode;

        match event.code {
            // Quit application
            KeyCode::Char('q') => self.state.should_quit = true,
            
            // Handle navigation in explorer
            KeyCode::Up | KeyCode::Char('k') => self.explorer.select_previous(),
            KeyCode::Down | KeyCode::Char('j') => self.explorer.select_next(),
            KeyCode::Enter | KeyCode::Char('l') => {
                // Open directory or file
                if let Err(e) = self.explorer.open_selected() {
                    // In a real app, we'd use a better error handling strategy
                    eprintln!("Error: {}", e);
                }
            },
            KeyCode::Char('h') => {
                // Go back to parent directory
                if let Err(e) = self.explorer.go_to_parent() {
                    eprintln!("Error: {}", e);
                }
            },
            _ => {}
        }
    }
}