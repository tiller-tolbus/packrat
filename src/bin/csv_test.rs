use anyhow::Result;
use csv::{QuoteStyle, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Chunk {
    id: String,
    file_path: PathBuf,
    start_line: usize,
    end_line: usize,
    content: String,
    timestamp: u64,
    edited: bool,
    labels: String,
}

fn main() -> Result<()> {
    // Create a chunk with the problematic UI module code
    let problem_content = r#"use ratatui::text::{Line, Span};
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
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
        .split(frame.size());"#;

    let chunk = Chunk {
        id: "test-id-123".to_string(),
        file_path: PathBuf::from("src/ui/mod.rs"),
        start_line: 3,
        end_line: 38,
        content: problem_content.to_string(),
        timestamp: 1742527630,
        edited: false,
        labels: "".to_string(),
    };

    // Test with default CSV writer
    let test_file1 = File::create("test_default.csv")?;
    let writer1 = BufWriter::new(test_file1);
    let mut csv_writer1 = csv::Writer::from_writer(writer1);
    csv_writer1.serialize(&chunk)?;
    csv_writer1.flush()?;
    println!("Wrote test_default.csv");

    // Test with improved CSV writer (with proper quoting)
    let test_file2 = File::create("test_improved.csv")?;
    let writer2 = BufWriter::new(test_file2);
    let mut csv_writer2 = WriterBuilder::new()
        .quote_style(QuoteStyle::Always)
        .double_quote(true)
        .from_writer(writer2);
    csv_writer2.serialize(&chunk)?;
    csv_writer2.flush()?;
    println!("Wrote test_improved.csv");

    // Test reading from both files
    println!("\nReading test_default.csv:");
    let test_read1 = File::open("test_default.csv")?;
    let reader1 = BufReader::new(test_read1);
    let mut csv_reader1 = csv::Reader::from_reader(reader1);
    for result in csv_reader1.deserialize() {
        let read_chunk: Chunk = result?;
        println!("Read chunk with id: {}", read_chunk.id);
        println!("Content length: {}", read_chunk.content.len());
        println!("Content contains 'chunks =' substring: {}", 
                 read_chunk.content.contains("chunks ="));
    }

    println!("\nReading test_improved.csv:");
    let test_read2 = File::open("test_improved.csv")?;
    let reader2 = BufReader::new(test_read2);
    let mut csv_reader2 = csv::Reader::from_reader(reader2);
    for result in csv_reader2.deserialize() {
        let read_chunk: Chunk = result?;
        println!("Read chunk with id: {}", read_chunk.id);
        println!("Content length: {}", read_chunk.content.len());
        println!("Content contains 'chunks =' substring: {}", 
                 read_chunk.content.contains("chunks ="));
    }

    println!("Tests completed successfully");
    Ok(())
}