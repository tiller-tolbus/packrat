use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, ListState, Wrap, Clear};
use ratatui::Frame;
use std::fmt::Write;

use crate::app::state::{AppMode, AppState};
use packrat::editor::Editor;
use crate::explorer::Explorer;
use crate::viewer::Viewer;

/// Render the UI
pub fn render(frame: &mut Frame, state: &AppState, explorer: &Explorer, viewer: &Viewer, editor: &mut Editor) {
    // Render the main UI based on the current mode
    match state.mode {
        AppMode::Explorer => render_explorer_mode(frame, state, explorer),
        AppMode::Viewer => render_viewer_mode(frame, state, viewer),
        AppMode::Editor => render_editor_mode(frame, state, editor),
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
        .split(frame.area());
    
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
        .split(frame.area());
    
    // Render text viewer content (with file name in its block)
    render_viewer_content(frame, chunks[0], viewer);
    
    // Render viewer status line
    render_viewer_status(frame, chunks[1], viewer);
}


/// Render the file explorer content
fn render_explorer_content(frame: &mut Frame, area: Rect, explorer: &Explorer) {
    // Create a title with a square character on both sides
    let title_text = "□ Packrat □";
    
    // Center align the title
    let centered_title = Line::from(title_text).centered();
    
    let block = Block::default()
        .title(centered_title)
        .borders(Borders::ALL);
    
    let inner_area = block.inner(area);
    frame.render_widget(block, area);
    
    // Create list items from directory entries
    let items: Vec<ListItem> = explorer
        .entries()
        .iter()
        .map(|entry| {
            // Use different colors based on directory or file status
            let (symbol, name_style) = if entry.is_dir {
                ("▶ ", Style::default().fg(Color::Cyan))
            } else {
                // For files, color based on chunking progress
                let progress = entry.chunking_progress;
                let name_style = if progress >= 99.0 {
                    // Fully chunked - green background
                    Style::default().bg(Color::Green).fg(Color::Black)
                } else if progress >= 66.0 {
                    // Mostly chunked - orange background
                    Style::default().bg(Color::LightRed).fg(Color::Black)
                } else if progress >= 33.0 {
                    // Partially chunked - yellow background
                    Style::default().bg(Color::Yellow).fg(Color::Black)
                } else if progress > 0.0 {
                    // Minimally chunked - dim yellow background
                    Style::default().bg(Color::LightYellow).fg(Color::Black)
                } else {
                    // Not chunked - default terminal colors
                    Style::default()
                };
                
                ("■ ", name_style)
            };
            
            // Add progress indicator for files with non-zero progress
            let content = if !entry.is_dir && entry.chunking_progress > 0.0 {
                vec![
                    Span::styled(symbol, Style::default()),
                    Span::styled(&entry.name, name_style),
                    Span::styled(
                        format!(" [{:.0}%]", entry.chunking_progress), 
                        Style::default().fg(Color::DarkGray)
                    )
                ]
            } else {
                vec![
                    Span::styled(symbol, Style::default()),
                    Span::styled(&entry.name, name_style)
                ]
            };
            
            ListItem::new(Line::from(content))
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
    
    // Add chunking status to title with consistent square character
    let chunking_percent = viewer.chunking_percentage();
    let title_text = if chunking_percent > 0.0 {
        format!("□ {} [{:.1}% Chunked] □", file_name, chunking_percent)
    } else {
        format!("□ {} □", file_name)
    };
    
    // Add token count for the current selection with squares on both sides
    let token_info = if let Some(token_count) = viewer.selection_token_count() {
        let percentage = (token_count as f64 / viewer.max_tokens_per_chunk() as f64) * 100.0;
        
        // Token percentage info - debug message for over limit is set in app code
        
        // Format token info consistently
        format!("□ TOKENS: {} / {} ({}%) □", token_count, viewer.max_tokens_per_chunk(), percentage as usize)
    } else {
        let total = viewer.total_token_count();
        format!("□ TOTAL TOKENS: {} □", total)
    };
    
    // Use default style for consistent appearance
    let token_style = Style::default();
    
    // Left and right titles using ratatui's built-in title support
    let left_title = title_text;
    
    // No special border styling needed for consistency
    
    // Use left-aligned and right-aligned titles with Ratatui's alignment methods
    let left_aligned_title = Line::from(left_title).left_aligned();
    let right_aligned_title = Line::from(Span::styled(token_info, token_style)).right_aligned();
    
    // Create the block with both titles
    let block = Block::default()
        .title(left_aligned_title)
        .title(right_aligned_title)
        .borders(Borders::ALL);
    
    let inner_area = block.inner(area);
    frame.render_widget(block, area);
    
    // Get visible content based on scroll position and terminal height
    let content_height = inner_area.height as usize;
    let visible_content = viewer.visible_content(content_height);
    
    // Get selection range if any
    let selection_range = viewer.selection_range();
    
    // Determine cursor position relative to the visible area
    let cursor_position = viewer.cursor_position();
    let scroll_position = viewer.scroll_position();
    
    // We'll calculate the number of lines as needed in our loop
    
    // Create text content for the paragraph with selection highlighting
    let content: Vec<Line> = visible_content
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let line_position = scroll_position + i;
            let is_cursor_line = line_position == cursor_position;
            
            // Check if this line is in the selection range
            let is_selected = selection_range
                .map(|(start, end)| line_position >= start && line_position <= end)
                .unwrap_or(false);
            
            // Define style based on selection and chunk status
            let style = if is_selected {
                Style::default().bg(Color::Yellow).fg(Color::Black)
            } else if viewer.is_line_chunked(line_position) {
                // Use yellow highlight for chunked lines
                Style::default().bg(Color::Yellow).fg(Color::Black)
            } else {
                Style::default().fg(Color::Reset)
            };
            
            // Calculate line number width based on total content lines
            // Use at least 3 chars width for line numbers
            let total_lines = viewer.content().len();
            let line_number_width = total_lines.to_string().len().max(3);
            
            // Create the line number for this line (1-indexed for display)
            let absolute_line_number = scroll_position + i + 1;
            
            // Create the line's content span with appropriate style
            let content_span = Span::styled(line.as_str(), style);
            
            // Cursor and line number handling
            if is_cursor_line {
                // Choose appropriate cursor style based on selection mode
                let cursor_style = if viewer.is_selection_mode() {
                    Style::default().bg(Color::Yellow).fg(Color::Black)
                } else {
                    Style::default().bg(Color::DarkGray).fg(Color::White)
                };
                
                // Format with line number, cursor arrow, and content
                let line_number_with_cursor = format!("{:>width$} > ", absolute_line_number, width = line_number_width);
                
                if line.is_empty() {
                    // For empty lines with cursor
                    Line::from(vec![
                        Span::styled(line_number_with_cursor, cursor_style),
                        Span::raw("") // Empty content but maintain structure
                    ])
                } else {
                    // For lines with content
                    Line::from(vec![
                        Span::styled(line_number_with_cursor, cursor_style),
                        content_span
                    ])
                }
            } else {
                // Non-cursor lines get a subtle line number style
                let line_number_style = Style::default().fg(Color::DarkGray);
                
                // Format with line number and proper spacing to align with cursor lines
                // Add two spaces where the cursor arrow would be (> )
                let line_number = format!("{:>width$}   ", absolute_line_number, width = line_number_width);
                
                if line.is_empty() {
                    // Empty lines still need the line number
                    Line::from(vec![
                        Span::styled(line_number, line_number_style),
                        Span::raw("")
                    ])
                } else {
                    // Standard lines with content
                    Line::from(vec![
                        Span::styled(line_number, line_number_style),
                        content_span
                    ])
                }
            }
        })
        .collect();
    
    // Create and render the paragraph widget with wrap
    // We can't use indent directly in this version of Ratatui, but our content structure 
    // with line numbers already creates the desired indentation effect
    let content_widget = Paragraph::new(content)
        .style(Style::default().fg(Color::Reset))
        .wrap(Wrap { trim: true }); // Use trim=true to handle whitespace consistently
    
    frame.render_widget(content_widget, inner_area);
}

/// Render the explorer status line - more compact to fit in small terminals
fn render_explorer_status(frame: &mut Frame, area: Rect) {
    let status = Paragraph::new(" ?:Help | q/Esc:Quit | ↑↓/kj:Nav | PgUp/Dn:Page | Enter/→:Open | ←:Back")
        .style(Style::default().fg(Color::Reset));
    
    frame.render_widget(status, area);
}

/// Render the viewer status line - more compact to fit in small terminals
fn render_viewer_status(frame: &mut Frame, area: Rect, viewer: &Viewer) {
    let selection_info = if viewer.is_selection_mode() {
        "SELECTION MODE | "
    } else {
        if viewer.selection_range().is_some() {
            "TEXT SELECTED | "
        } else {
            ""
        }
    };
    
    // Add chunking percentage info if any chunks exist
    let chunking_percent = viewer.chunking_percentage();
    let chunk_info = if chunking_percent > 0.0 {
        format!("{:.1}% CHUNKED | ", chunking_percent)
    } else {
        "".to_string()
    };
    
    // Create status line with default styling for consistency
    let status_line = if chunk_info.is_empty() {
        Line::from(format!(" ?:Help | Space:Toggle Selection | s:Save Chunk | {} q/Esc:Back | ↑↓/kj:Move", selection_info))
    } else {
        Line::from(format!(" ?:Help | Space:Toggle Selection | s:Save Chunk | {} | {} q/Esc:Back | ↑↓/kj:Move", 
            chunk_info, selection_info))
    };
    
    let status = Paragraph::new(status_line);
    
    frame.render_widget(status, area);
}

/// Render a help panel with detailed keyboard shortcuts
fn render_help_panel(frame: &mut Frame, mode: AppMode) {
    let area = frame.area();
    
    // Create a centered box for the help panel
    let width = 60.min(area.width.saturating_sub(4));
    let height = match mode {
        AppMode::Explorer => 15.min(area.height.saturating_sub(4)),
        AppMode::Viewer => 15.min(area.height.saturating_sub(4)),
        AppMode::Editor => 13.min(area.height.saturating_sub(4)),
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
    let title = "□ Keyboard Shortcuts □";
    let content = match mode {
        AppMode::Explorer => {
            vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("  Navigation", Style::default().add_modifier(Modifier::BOLD))
                ]),
                Line::from("    ↑/k, ↓/j            Move selection up/down"),
                Line::from("    PgUp, PgDn          Page up/down"),
                Line::from("    Home, End           Jump to top/bottom"),
                Line::from(""),
                Line::from(vec![
                    Span::styled("  Actions", Style::default().add_modifier(Modifier::BOLD))
                ]),
                Line::from("    Enter, l, →         Open selected file/directory"),
                Line::from("    h, ←                Go to parent directory"),
                Line::from("    q, Esc              Quit application"),
                Line::from(""),
                Line::from(vec![
                    Span::styled("  Help", Style::default().add_modifier(Modifier::BOLD))
                ]),
                Line::from("    ?                   Toggle this help panel"),
                Line::from(""),
                Line::from("    Press any key to close help")
            ]
        },
        AppMode::Viewer => {
            vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("  Navigation", Style::default().add_modifier(Modifier::BOLD))
                ]),
                Line::from("    ↑/k, ↓/j            Move cursor up/down"),
                Line::from("    Shift+↑/↓, j/k      Fast scroll (5 lines)"),
                Line::from("    PgUp, PgDn          Page up/down"),
                Line::from("    Home, End           Jump to top/bottom"),
                Line::from(""),
                Line::from(vec![
                    Span::styled("  Selection & Chunking", Style::default().add_modifier(Modifier::BOLD))
                ]),
                Line::from("    Space               Toggle selection mode"),
                Line::from("    s                   Save selected text as chunk"),
                Line::from("    e                   Open selected text in editor"),
                Line::from(""),
                Line::from(vec![
                    Span::styled("  Other Actions", Style::default().add_modifier(Modifier::BOLD))
                ]),
                Line::from("    q, Esc              Return to file explorer"),
                Line::from("    ?                   Toggle this help panel"),
                Line::from(""),
                Line::from("    Press any key to close help")
            ]
        },
        AppMode::Editor => {
            vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("  Vim Commands", Style::default().add_modifier(Modifier::BOLD))
                ]),
                Line::from("    h, j, k, l          Movement (normal mode)"),
                Line::from("    i, a, o             Enter insert mode"),
                Line::from("    v                   Enter visual mode"),
                Line::from("    :                   Enter command mode"),
                Line::from("    :w                  Save changes"),
                Line::from("    :wq, :x             Save and exit"),
                Line::from("    :q                  Quit (requires no changes)"),
                Line::from("    :q!                 Force quit without saving"),
                Line::from(""),
                Line::from(vec![
                    Span::styled("  Direct Actions", Style::default().add_modifier(Modifier::BOLD))
                ]),
                Line::from("    Ctrl+S              Save changes and exit"),
                Line::from("    Esc                 Exit to viewer (normal mode only)"),
                Line::from(""),
                Line::from("    Press any key to close help")
            ]
        }
    };
    
    // Create a clear overlay for the help panel background
    frame.render_widget(Clear, help_area);
    
    // Create the block for the help panel with a centered title
    let block = Block::default()
        .title(title)
        .title_alignment(ratatui::layout::Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Reset).fg(Color::Reset));
    
    let inner_area = block.inner(help_area);
    
    // Render the block first
    frame.render_widget(block, help_area);
    
    // Then render the content
    let help_content = Paragraph::new(content)
        .alignment(ratatui::layout::Alignment::Left);
    
    frame.render_widget(help_content, inner_area);
}

/// Render the editor mode UI
fn render_editor_mode(frame: &mut Frame, state: &AppState, editor: &mut Editor) {
    if state.show_help {
        render_help_panel(frame, AppMode::Editor);
        return;
    }
    
    // Create the layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),     // Editor content
            Constraint::Length(1),  // Status line
        ])
        .split(frame.area());
    
    // Get the filename or use a default
    let file_name = editor.file_name().unwrap_or_else(|| "Untitled".to_string());
    let left_title = format!("□ Editing: {} □", file_name);
    
    // Add token count for the text being edited
    let token_count = editor.token_count();
    let max_tokens = editor.max_tokens();
    let percentage = (token_count as f64 / max_tokens as f64) * 100.0;
    
    // Format token info consistently
    let token_info = format!("□ TOKENS: {} / {} ({}%) □", token_count, max_tokens, percentage as usize);
    
    // Token percentage info - debug message for over limit is set in app code
    
    // Use default style for consistent appearance
    let token_style = Style::default();
    
    // Use left-aligned and right-aligned titles with Ratatui's alignment methods
    let left_aligned_title = Line::from(left_title).left_aligned();
    let right_aligned_title = Line::from(Span::styled(token_info, token_style)).right_aligned();
    
    // Create the block with both titles
    let block = Block::default()
        .title(left_aligned_title)
        .title(right_aligned_title)
        .borders(Borders::ALL);
    
    let inner_area = block.inner(chunks[0]);
    frame.render_widget(block, chunks[0]);
    
    // Render the editor widget
    let view = editor.view();
    frame.render_widget(view, inner_area);
    
    // Render editor status line
    render_editor_status(frame, chunks[1], editor);
}

/// Render the editor status line
fn render_editor_status(frame: &mut Frame, area: Rect, editor: &Editor) {
    // Get editor mode
    let mode = editor.mode();
    
    // Show modified indicator
    let modified = if editor.is_modified() {
        "[MODIFIED] "
    } else {
        ""
    };
    
    // Create status line with appropriate guidance based on mode
    let status_line = if mode == "NORMAL" {
        format!(" {} | {}?:Help | i:Insert Mode | Ctrl+S:Save | Esc:Cancel", mode, modified)
    } else if mode == "INSERT" {
        format!(" {} | {}?:Help | Esc:Normal Mode | Ctrl+S:Save", mode, modified)
    } else {
        format!(" {} | {}?:Help | Ctrl+S:Save | Esc:Cancel", mode, modified)
    };
    
    let status = Paragraph::new(status_line);
    
    frame.render_widget(status, area);
}

/// Render a debug message overlay at the bottom of the screen
fn render_debug_overlay(frame: &mut Frame, message: &str) {
    let area = frame.area();
    
    // Create a small overlay at the bottom of the screen
    let debug_area = Rect {
        x: area.x,
        y: area.height.saturating_sub(2),
        width: area.width,
        height: 1,
    };
    
    // Create a clear widget for the overlay background
    frame.render_widget(Clear, debug_area);
    
    // Create the debug message
    let debug_message = Paragraph::new(message)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White))
        .alignment(ratatui::layout::Alignment::Center);
    
    frame.render_widget(debug_message, debug_area);
}



/// UI serializer for debug output
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
        
        // Explorer state
        writeln!(&mut output, "Explorer State:").unwrap();
        writeln!(&mut output, "---------------").unwrap();
        writeln!(&mut output, "Current Directory: {}", explorer.current_path().display()).unwrap();
        writeln!(&mut output, "Root Directory: {}", explorer.root_dir().display()).unwrap();
        writeln!(&mut output, "Selected Index: {}", explorer.selected_index()).unwrap();
        writeln!(&mut output, "").unwrap();
        
        // Entries
        writeln!(&mut output, "Directory Entries:").unwrap();
        writeln!(&mut output, "-----------------").unwrap();
        for (i, entry) in explorer.entries().iter().enumerate() {
            let selected = if i == explorer.selected_index() { " [SELECTED]" } else { "" };
            let chunking = if entry.chunking_progress > 0.0 { 
                format!(" [CHUNKED: {:.1}%]", entry.chunking_progress) 
            } else { 
                "".to_string() 
            };
            
            writeln!(&mut output, "{}{}  {}{}", 
                if entry.is_dir { "📁" } else { "📄" },
                selected,
                entry.name,
                chunking
            ).unwrap();
        }
        writeln!(&mut output, "").unwrap();
        
        // Status line
        writeln!(&mut output, "Status Line:").unwrap();
        writeln!(&mut output, "------------").unwrap();
        writeln!(&mut output, "?:Help | q/Esc:Quit | ↑↓/kj:Nav | PgUp/Dn:Page | Enter/→:Open | ←:Back").unwrap();
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
        
        // Viewer state
        writeln!(&mut output, "Viewer State:").unwrap();
        writeln!(&mut output, "-------------").unwrap();
        writeln!(&mut output, "File: {:?}", viewer.file_path()).unwrap();
        writeln!(&mut output, "Scroll Position: Line {}", viewer.scroll_position() + 1).unwrap();
        writeln!(&mut output, "Selection Mode: {}", if viewer.is_selection_mode() { "ACTIVE" } else { "INACTIVE" }).unwrap();
        
        if let Some((start, end)) = viewer.selection_range() {
            writeln!(&mut output, "Selection Range: Lines {} to {}", start + 1, end + 1).unwrap();
            writeln!(&mut output, "Selected Line Count: {}", end - start + 1).unwrap();
        } else {
            writeln!(&mut output, "Selection Range: None").unwrap();
        }
        writeln!(&mut output, "Cursor Position: Line {}", viewer.cursor_position() + 1).unwrap();
        writeln!(&mut output, "").unwrap();
        
        // Content
        writeln!(&mut output, "File Content Preview:").unwrap();
        writeln!(&mut output, "====================").unwrap();
        
        // Show current scroll position and nearby content (10 lines)
        let _pos = viewer.scroll_position();
        let cursor_pos = viewer.cursor_position();
        let content = viewer.content();
        let selection_range = viewer.selection_range();
        
        let start = if cursor_pos > 5 { cursor_pos - 5 } else { 0 };
        let end = (start + 15).min(content.len());
        
        for i in start..end {
            let is_selected = selection_range
                .map(|(start, end)| i >= start && i <= end)
                .unwrap_or(false);
                
            let marker = if i == cursor_pos { 
                if viewer.is_selection_mode() { " => " } else { " -> " } 
            } else if is_selected {
                " ** "
            } else { 
                "    " 
            };
            
            let line_num = format!("{:4}", i + 1);
            let line_content = content.get(i).map_or("", |s| s.as_str());
            writeln!(&mut output, "{}{}: {}", marker, line_num, line_content).unwrap();
        }
        writeln!(&mut output, "").unwrap();
        
        // Status
        writeln!(&mut output, "Status Line:").unwrap();
        writeln!(&mut output, "------------").unwrap();
        
        let selection_info = if viewer.is_selection_mode() {
            "SELECTION MODE | "
        } else {
            if viewer.selection_range().is_some() {
                "TEXT SELECTED | "
            } else {
                ""
            }
        };
        
        writeln!(&mut output, "?:Help | Space:Toggle Selection | {} q/Esc:Back | ↑↓/kj:Move | PgUp/Dn:Page | Home/End:Jump", 
            selection_info).unwrap();
        writeln!(&mut output, "").unwrap();
        
        // Token information
        writeln!(&mut output, "Token Information:").unwrap();
        writeln!(&mut output, "------------------").unwrap();
        writeln!(&mut output, "Total tokens in file: {}", viewer.total_token_count()).unwrap();
        if let Some(count) = viewer.selection_token_count() {
            let percentage = (count as f64 / viewer.max_tokens_per_chunk() as f64) * 100.0;
            writeln!(&mut output, "Selection tokens: {} ({:.1}% of limit {})", 
                count, percentage, viewer.max_tokens_per_chunk()).unwrap();
            if percentage > 100.0 {
                writeln!(&mut output, "WARNING: Selection exceeds token limit!").unwrap();
            }
        }
        writeln!(&mut output, "").unwrap();
        
        // Debug info
        writeln!(&mut output, "Terminal Info:").unwrap();
        writeln!(&mut output, "-------------").unwrap();
        writeln!(&mut output, "Debug Mode: Active").unwrap();
        writeln!(&mut output, "Shortcut to dump UI state: Ctrl+D").unwrap();
        
        output
    }
    
    /// Capture the editor mode UI state as a formatted string
    pub fn capture_editor(state: &AppState) -> String {
        let mut output = String::new();
        
        // Add header
        writeln!(&mut output, "=== PACKRAT UI STATE DUMP ===").unwrap();
        writeln!(&mut output, "Mode: Editor").unwrap();
        writeln!(&mut output, "Time: {:?}", std::time::SystemTime::now()).unwrap();
        writeln!(&mut output, "Show Help: {}", state.show_help).unwrap();
        writeln!(&mut output, "").unwrap();
        
        // Status info
        writeln!(&mut output, "Editing selected text - content not shown in debug view").unwrap();
        writeln!(&mut output, "").unwrap();
        
        // Status line
        writeln!(&mut output, "Status Line:").unwrap();
        writeln!(&mut output, "------------").unwrap();
        writeln!(&mut output, "?:Help | Ctrl+S:Save | Esc:Cancel | Arrow keys:Navigate | Type to edit").unwrap();
        writeln!(&mut output, "").unwrap();
        
        // Debug info
        writeln!(&mut output, "Terminal Info:").unwrap();
        writeln!(&mut output, "-------------").unwrap();
        writeln!(&mut output, "Debug Mode: Active").unwrap();
        writeln!(&mut output, "Shortcut to dump UI state: Ctrl+D").unwrap();
        
        output
    }
}