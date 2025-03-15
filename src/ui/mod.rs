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
        AppMode::Explorer => render_explorer_mode(frame, state, explorer),
        AppMode::Viewer => render_viewer_mode(frame, state, viewer),
    }
}

/// Render the explorer mode UI
fn render_explorer_mode(frame: &mut Frame, state: &AppState, explorer: &Explorer) {
    if state.show_help {
        render_help_panel(frame, AppMode::Explorer);
        return;
    }

    // Create the layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),     // Explorer
            Constraint::Length(1),  // Status line
        ])
        .split(frame.size());
    
    // Render file explorer (with the application title in its block)
    render_explorer_content(frame, chunks[0], explorer);
    
    // Render explorer status line
    render_explorer_status(frame, chunks[1]);
}

/// Render the viewer mode UI
fn render_viewer_mode(frame: &mut Frame, state: &AppState, viewer: &Viewer) {
    if state.show_help {
        render_help_panel(frame, AppMode::Viewer);
        return;
    }

    // Create the layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),     // Viewer content
            Constraint::Length(1),  // Status line
        ])
        .split(frame.size());
    
    // Render text viewer content (with file name in its block)
    render_viewer_content(frame, chunks[0], viewer);
    
    // Render viewer status line
    render_viewer_status(frame, chunks[1]);
}


/// Render the file explorer content
fn render_explorer_content(frame: &mut Frame, area: Rect, explorer: &Explorer) {
    let block = Block::default()
        .title("Packrat - Text Chunking Tool")
        .borders(Borders::ALL)
        .title_style(Style::default().add_modifier(Modifier::BOLD));
    
    let inner_area = block.inner(area);
    frame.render_widget(block, area);
    
    // Create list items from directory entries
    let items: Vec<ListItem> = explorer
        .entries()
        .iter()
        .map(|entry| {
            // Use cleaner Unicode symbols for folders and files
            let (symbol, color) = if entry.is_dir {
                ("▶ ", Color::Cyan)
            } else {
                ("■ ", Color::White)
            };
            
            ListItem::new(Line::from(vec![
                Span::styled(symbol, Style::default().fg(color)),
                Span::raw(&entry.name)
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
    // Get file name for the title
    let file_name = viewer.file_path()
        .map(|p| p.file_name().unwrap_or_default().to_string_lossy().to_string())
        .unwrap_or_else(|| "Unknown File".to_string());
        
    let block = Block::default()
        .title(format!("Viewing: {}", file_name))
        .borders(Borders::ALL)
        .title_style(Style::default().add_modifier(Modifier::BOLD));
    
    let inner_area = block.inner(area);
    frame.render_widget(block, area);
    
    // Get visible content based on scroll position and terminal height
    let content_height = inner_area.height as usize;
    let visible_content = viewer.visible_content(content_height);
    
    // Create text content for the paragraph
    let content = visible_content
        .iter()
        .map(|line| Line::from(line.as_str()))
        .collect::<Vec<Line>>();
    
    // Create and render the paragraph widget
    let content_widget = Paragraph::new(content)
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: false });
    
    frame.render_widget(content_widget, inner_area);
}

/// Render the explorer status line - more compact to fit in small terminals
fn render_explorer_status(frame: &mut Frame, area: Rect) {
    let status = Paragraph::new(" q:Quit | ↑↓/kj:Nav | PgUp/Dn:Page | Enter/→:Open | ←:Back | ?:Help")
        .style(Style::default().fg(Color::Gray));
    
    frame.render_widget(status, area);
}

/// Render the viewer status line - more compact to fit in small terminals
fn render_viewer_status(frame: &mut Frame, area: Rect) {
    let status = Paragraph::new(" q:Back | ↑↓/kj:Scroll | PgUp/Dn:Page | Home/End:Jump | ?:Help")
        .style(Style::default().fg(Color::Gray));
    
    frame.render_widget(status, area);
}

/// Render a help panel with detailed keyboard shortcuts
fn render_help_panel(frame: &mut Frame, mode: AppMode) {
    let area = frame.size();
    
    // Create a centered box for the help panel
    let width = 60.min(area.width.saturating_sub(4));
    let height = match mode {
        AppMode::Explorer => 15.min(area.height.saturating_sub(4)),
        AppMode::Viewer => 13.min(area.height.saturating_sub(4)),
    };
    
    let horizontal_padding = (area.width - width) / 2;
    let vertical_padding = (area.height - height) / 2;
    
    let help_area = Rect {
        x: area.x + horizontal_padding,
        y: area.y + vertical_padding,
        width,
        height,
    };
    
    // Create the help content based on current mode
    let title = "Keyboard Shortcuts";
    let content = match mode {
        AppMode::Explorer => {
            vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("Navigation", Style::default().add_modifier(Modifier::BOLD))
                ]),
                Line::from("  ↑/k, ↓/j        Move selection up/down"),
                Line::from("  PgUp, PgDn      Page up/down"),
                Line::from("  Home, End       Jump to top/bottom"),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Actions", Style::default().add_modifier(Modifier::BOLD))
                ]),
                Line::from("  Enter, l, →     Open selected file/directory"),
                Line::from("  h, ←            Go to parent directory"),
                Line::from("  q               Quit application"),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Help", Style::default().add_modifier(Modifier::BOLD))
                ]),
                Line::from("  ?               Toggle this help panel"),
                Line::from("  Press any key to close help")
            ]
        },
        AppMode::Viewer => {
            vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("Navigation", Style::default().add_modifier(Modifier::BOLD))
                ]),
                Line::from("  ↑/k, ↓/j        Scroll up/down"),
                Line::from("  PgUp, PgDn      Page up/down"),
                Line::from("  Home, End       Jump to top/bottom"),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Actions", Style::default().add_modifier(Modifier::BOLD))
                ]),
                Line::from("  q, Esc          Return to file explorer"),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Help", Style::default().add_modifier(Modifier::BOLD))
                ]),
                Line::from("  ?               Toggle this help panel"),
                Line::from("  Press any key to close help")
            ]
        }
    };
    
    // Render help panel with a block and title
    let help_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    
    let help_paragraph = Paragraph::new(content)
        .block(help_block)
        .alignment(ratatui::layout::Alignment::Left);
    
    frame.render_widget(
        help_paragraph, 
        help_area
    );
}