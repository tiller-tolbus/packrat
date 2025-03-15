use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, ListState, Wrap, Clear};
use ratatui::Frame;
use std::fmt::Write;

use crate::app::state::{AppMode, AppState};
use crate::explorer::Explorer;
use crate::viewer::Viewer;

/// Render the UI
pub fn render(frame: &mut Frame, state: &AppState, explorer: &Explorer, viewer: &Viewer) {
    // Render the main UI based on the current mode
    match state.mode {
        AppMode::Explorer => render_explorer_mode(frame, state, explorer),
        AppMode::Viewer => render_viewer_mode(frame, state, viewer),
    }
    
    // Render debug message overlay if one exists
    if let Some(message) = &state.debug_message {
        render_debug_overlay(frame, message);
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
    // Create a centered title
    let title_text = "◆ Packrat ◆";
    let title = create_centered_title(&title_text, area.width);
    
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL);
    
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
        
    // Create a centered title
    let title_text = format!("⊡ {}", file_name);
    let title = create_centered_title(&title_text, area.width);
    
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL);
    
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

/// Create a centered title string based on the available width
fn create_centered_title(title: &str, width: u16) -> String {
    if width <= 4 {  // Need at least 2 chars for borders + 1 for title + 1 for space
        return format!(" {} ", title); // Basic padding with minimal space
    }
    
    // Calculate usable width (accounting for borders and spaces)
    let usable_width = width as usize - 4;  // 2 for borders, 2 for minimum spaces
    let title_len = title.chars().count();
    
    if title_len >= usable_width {
        return format!(" {} ", title); // Not enough space for centering, just add minimal padding
    }
    
    // Calculate padding
    let padding = usable_width - title_len;
    let left_padding = padding / 2;
    let right_padding = padding - left_padding;
    
    // Create centered title with proper spaces on both sides to preserve borders
    format!(" {}{}{} ", " ".repeat(left_padding), title, " ".repeat(right_padding))
}

/// Render a debug message overlay at the bottom of the screen
/// 
/// This function creates a temporary overlay that appears at the bottom of the screen
/// and shows debug information without disrupting the main UI.
pub fn render_debug_overlay(frame: &mut Frame, message: &str) {
    let terminal_size = frame.size();
    
    // Create an area for the debug overlay at the bottom of the screen
    // Maximum height of 8 rows or 30% of the terminal height, whichever is smaller
    let max_height = (terminal_size.height as f32 * 0.3).min(8.0) as u16;
    let message_lines = message.lines().count() as u16;
    let height = message_lines.min(max_height);
    
    let overlay_area = Rect {
        x: 2,
        y: terminal_size.height.saturating_sub(height + 2),
        width: terminal_size.width.saturating_sub(4),
        height: height + 2, // +2 for borders
    };
    
    // Create a semi-transparent background that covers the overlay area
    frame.render_widget(Clear, overlay_area);
    
    // Render the debug message in a bordered block
    let debug_block = Block::default()
        .title(" Debug Info ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    
    // Wrap our message in a paragraph
    let debug_text = message.lines()
        .map(|line| Line::from(line))
        .collect::<Vec<Line>>();
    
    let debug_paragraph = Paragraph::new(debug_text)
        .block(debug_block)
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .wrap(Wrap { trim: false });
    
    frame.render_widget(debug_paragraph, overlay_area);
}

/// UI state serialization for debugging
pub struct UiSerializer;

impl UiSerializer {
    /// Capture the explorer mode UI state as a formatted string
    pub fn capture_explorer(state: &AppState, explorer: &Explorer) -> String {
        let mut output = String::new();
        
        // Add header
        writeln!(&mut output, "=== PACKRAT UI STATE DUMP ===").unwrap();
        writeln!(&mut output, "Mode: Explorer").unwrap();
        writeln!(&mut output, "Time: {:?}", std::time::SystemTime::now()).unwrap();
        writeln!(&mut output, "Show Help: {}", state.show_help).unwrap();
        writeln!(&mut output, "").unwrap();
        
        // Current directory
        writeln!(&mut output, "Current Directory: {:?}", explorer.current_path()).unwrap();
        writeln!(&mut output, "").unwrap();
        
        // Directory entries
        writeln!(&mut output, "Directory Entries:").unwrap();
        writeln!(&mut output, "==================").unwrap();
        for (i, entry) in explorer.entries().iter().enumerate() {
            let selected = if i == explorer.selected_index() { " -> " } else { "    " };
            let entry_type = if entry.is_dir { "[DIR] " } else { "[FILE]" };
            writeln!(&mut output, "{}{} {}", selected, entry_type, entry.name).unwrap();
        }
        writeln!(&mut output, "").unwrap();
        
        // Status
        writeln!(&mut output, "Status Line:").unwrap();
        writeln!(&mut output, "------------").unwrap();
        writeln!(&mut output, "q:Quit | ↑↓/kj:Nav | PgUp/Dn:Page | Enter/→:Open | ←:Back | ?:Help").unwrap();
        writeln!(&mut output, "").unwrap();
        
        // Debug info
        writeln!(&mut output, "Terminal Info:").unwrap();
        writeln!(&mut output, "-------------").unwrap();
        writeln!(&mut output, "Debug Mode: Active").unwrap();
        writeln!(&mut output, "Shortcut to dump UI state: Ctrl+D").unwrap();
        
        output
    }
    
    /// Capture the viewer mode UI state as a formatted string
    pub fn capture_viewer(state: &AppState, viewer: &Viewer) -> String {
        let mut output = String::new();
        
        // Add header
        writeln!(&mut output, "=== PACKRAT UI STATE DUMP ===").unwrap();
        writeln!(&mut output, "Mode: Viewer").unwrap();
        writeln!(&mut output, "Time: {:?}", std::time::SystemTime::now()).unwrap();
        writeln!(&mut output, "Show Help: {}", state.show_help).unwrap();
        writeln!(&mut output, "").unwrap();
        
        // File info
        writeln!(&mut output, "Viewing File: {:?}", viewer.file_path()).unwrap();
        writeln!(&mut output, "").unwrap();
        
        // Content
        writeln!(&mut output, "File Content Preview:").unwrap();
        writeln!(&mut output, "====================").unwrap();
        
        // Show current scroll position and nearby content (10 lines)
        let pos = viewer.scroll_position();
        let content = viewer.content();
        
        let start = if pos > 5 { pos - 5 } else { 0 };
        let end = (start + 15).min(content.len());
        
        for i in start..end {
            let marker = if i == pos { " -> " } else { "    " };
            let line_num = format!("{:4}", i + 1);
            let line_content = content.get(i).map_or("", |s| s.as_str());
            writeln!(&mut output, "{}{}: {}", marker, line_num, line_content).unwrap();
        }
        writeln!(&mut output, "").unwrap();
        
        // Status
        writeln!(&mut output, "Status Line:").unwrap();
        writeln!(&mut output, "------------").unwrap();
        writeln!(&mut output, "q:Back | ↑↓/kj:Scroll | PgUp/Dn:Page | Home/End:Jump | ?:Help").unwrap();
        writeln!(&mut output, "").unwrap();
        
        // Debug info
        writeln!(&mut output, "Terminal Info:").unwrap();
        writeln!(&mut output, "-------------").unwrap();
        writeln!(&mut output, "Debug Mode: Active").unwrap();
        writeln!(&mut output, "Shortcut to dump UI state: Ctrl+D").unwrap();
        
        output
    }
}