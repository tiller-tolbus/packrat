use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, ListState, Wrap};
use ratatui::Frame;

use crate::app::state::{AppMode, AppState};
use crate::explorer::Explorer;
use crate::viewer::Viewer;

/// Render the UI
pub fn render(frame: &mut Frame, state: &AppState, explorer: &Explorer, viewer: &Viewer) {
    match state.mode {
        AppMode::Explorer => render_explorer_mode(frame, explorer),
        AppMode::Viewer => render_viewer_mode(frame, viewer),
    }
}

/// Render the explorer mode UI
fn render_explorer_mode(frame: &mut Frame, explorer: &Explorer) {
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
    render_title(frame, chunks[0], "Packrat - Text Chunking Tool");
    
    // Render file explorer
    render_explorer_content(frame, chunks[1], explorer);
    
    // Render explorer status line
    render_explorer_status(frame, chunks[2]);
}

/// Render the viewer mode UI
fn render_viewer_mode(frame: &mut Frame, viewer: &Viewer) {
    // Create the layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(0),     // Viewer content
            Constraint::Length(1),  // Status line
        ])
        .split(frame.size());
    
    // Get file name for the title
    let file_name = viewer.file_path()
        .map(|p| p.file_name().unwrap_or_default().to_string_lossy().to_string())
        .unwrap_or_else(|| "Unknown File".to_string());
    
    // Render title
    render_title(frame, chunks[0], &format!("Viewing: {}", file_name));
    
    // Render text viewer content
    render_viewer_content(frame, chunks[1], viewer);
    
    // Render viewer status line
    render_viewer_status(frame, chunks[2]);
}

/// Render the application title
fn render_title(frame: &mut Frame, area: Rect, title: &str) {
    let title_widget = Paragraph::new(title)
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL));
    
    frame.render_widget(title_widget, area);
}

/// Render the file explorer content
fn render_explorer_content(frame: &mut Frame, area: Rect, explorer: &Explorer) {
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
    let mut state = ListState::default();
    state.select(Some(explorer.selected_index()));
    
    frame.render_stateful_widget(
        list,
        inner_area,
        &mut state
    );
}

/// Render the text viewer content
fn render_viewer_content(frame: &mut Frame, area: Rect, viewer: &Viewer) {
    let block = Block::default()
        .title("Text Viewer")
        .borders(Borders::ALL);
    
    let inner_area = block.inner(area);
    frame.render_widget(block, area);
    
    // Get visible content based on scroll position and terminal height
    let content_height = inner_area.height as usize;
    let visible_content = viewer.visible_content(content_height);
    
    // Create text content for the paragraph
    let content = visible_content
        .iter()
        .map(|line| Line::from(line.clone().as_str()))
        .collect::<Vec<Line>>();
    
    // Create and render the paragraph widget
    let content_widget = Paragraph::new(content)
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: false });
    
    frame.render_widget(content_widget, inner_area);
}

/// Render the explorer status line
fn render_explorer_status(frame: &mut Frame, area: Rect) {
    let status = Paragraph::new(" q: Quit | ↑/k,↓/j: Navigate | PgUp/PgDn: Page | Home/End: Jump | Enter/l/→: Open | h/←: Back")
        .style(Style::default().fg(Color::Gray));
    
    frame.render_widget(status, area);
}

/// Render the viewer status line
fn render_viewer_status(frame: &mut Frame, area: Rect) {
    let status = Paragraph::new(" q: Back to Explorer | ↑/k,↓/j: Scroll | PgUp/PgDn: Page | Home/End: Jump")
        .style(Style::default().fg(Color::Gray));
    
    frame.render_widget(status, area);
}