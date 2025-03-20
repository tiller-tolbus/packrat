# Packrat

A TUI (Terminal User Interface) application for interactively chunking text files, specifically designed for preparing content to be sent to Large Language Models like Claude.

## Features

- **File Explorer**: Navigate your filesystem to select files for chunking
- **Text Viewer**: View file contents with syntax highlighting
- **Text Selection**: Select text ranges for chunking
- **Chunk Editor**: Edit text chunks before saving
- **Token Counter**: Real-time token counting using Claude's tokenizer
- **Vim Keybindings**: Familiar navigation and editing for Vim users
- **Progress Tracking**: Track chunking progress for each file

## Installation

```bash
# Clone the repository
git clone https://github.com/your-username/packrat.git
cd packrat

# Build the application
cargo build --release

# Run the application
./target/release/packrat
```

## Usage

### Basic Controls

- **?**: Toggle help panel
- **q/Esc**: Quit current mode or application
- **j/k or Up/Down**: Navigate files/text
- **Enter or l**: Open file/directory
- **h or Left**: Go to parent directory
- **Space**: Toggle selection mode in viewer
- **e**: Edit selected text
- **s**: Save selection as a chunk

### Modes

Packrat operates in three modes:

1. **Explorer Mode**: Navigate files and directories
2. **Viewer Mode**: View file contents and select text for chunking
3. **Editor Mode**: Edit selected text before saving as a chunk

### Configuration

Packrat looks for configuration in:
1. `./packrat.toml` (current directory)
2. User config directory (platform-specific)

Generate a default configuration file with:

```bash
packrat --generate-config
```

See `packrat.example.toml` for configuration options.

## Purpose

Packrat helps break down large text files into manageable chunks for LLM processing. It uses Claude's tokenizer to count tokens in real-time, ensuring chunks stay within model context limits.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.