use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, ListState};
use ratatui::Frame;

use crate::app::state::AppState;
use crate::explorer::Explorer;

/// Render the UI
pub fn render(frame: &mut Frame, state: &AppState, explorer: &Explorer) {
    // Create the layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(0),     // Explorer
            Constraint::Length(1),  // Status line
        ])
        .split(frame.size());
    
    // Render title
    render_title(frame, chunks[0]);
    
    // Render file explorer
    render_explorer(frame, chunks[1], explorer);
    
    // Render status line
    render_status(frame, chunks[2]);
}

/// Render the application title
fn render_title(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new("Packrat - Text Chunking Tool")
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL));
    
    frame.render_widget(title, area);
}

/// Render the file explorer
fn render_explorer(frame: &mut Frame, area: Rect, explorer: &Explorer) {
    let block = Block::default()
        .title("File Explorer")
        .borders(Borders::ALL);
    
    let inner_area = block.inner(area);
    frame.render_widget(block, area);
    
    // Create list items from directory entries
    let items: Vec<ListItem> = explorer
        .entries()
        .iter()
        .map(|entry| {
            let prefix = if entry.is_dir { "📁 " } else { "📄 " };
            let content = format!("{}{}", prefix, entry.name);
            
            ListItem::new(Line::from(vec![
                Span::raw(content)
            ]))
        })
        .collect();
    
    // Create the list widget
    let list = List::new(items)
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        )
        .highlight_symbol("> ");
    
    // Render the list with the current selection
    // Create a ListState with the current selection
    let mut state = ListState::default();
    state.select(Some(explorer.selected_index()));
    
    frame.render_stateful_widget(
        list,
        inner_area,
        &mut state
    );
}

/// Render the status line
fn render_status(frame: &mut Frame, area: Rect) {
    let status = Paragraph::new(" q: Quit | ↑/k,↓/j: Navigate | Enter/l: Open | h: Back")
        .style(Style::default().fg(Color::Gray));
    
    frame.render_widget(status, area);
}