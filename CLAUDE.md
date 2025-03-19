---
The below project spec is your persistent guide. Do not change it.
Git commands will be handled by the user. Do not use them.
If you need to install a package, ask the user to do it for you.
Do not write outside this directory.
Keep your todo list in todo.txt
---

---
CLAUDE, DO NOT RUN THIS CODE YOURSELF!
You are being run in a terminal and attempting to run this program will cause the user session to bug out, erasing your context window.
Instead, write test cases for the things you want to handle, or let the user try the program themselves and ask them questions.
---

# Abstract

This project is a terminal user interface (TUI) application for interactively chunking text files. The tool lets users split large text files into smaller chunks by selecting text in a console UI and saving those selections as separate files. It provides a file explorer and text viewer with visual cues to track chunking progress. Key user stories and requirements include:

- **File Navigation**: A user can navigate through files and directories in a file explorer, restricted to a given root directory (chroot mode) to prevent leaving the allowed path.
- **Progress Highlighting**: The explorer indicates each file's chunking progress with color highlights – partially chunked files in yellow, mostly chunked in orange, and fully chunked in green for quick visual status.
- **Text Viewing**: A user can open a selected text file in a viewer pane and scroll through its content (using keyboard arrows/Vim keys for up/down, or other typical scrolling keys).
- **Text Selection**: A user can toggle selection mode by pressing **Space**; when selection mode is active, moving the cursor up or down highlights the selected lines to be chunked.  
- **Save Chunks**: Pressing `S` will save the currently highlighted lines as a chunk file in a designated storage directory. Each saved chunk is written to disk persistently.
- **Highlight Saved Chunks**: Once saved, the chunked lines in the viewer are highlighted (e.g., in orange) to indicate they have been chunked. This helps avoid re-selecting the same text. The file's entry in the explorer also updates its color if the new chunk changes its overall chunking status.
- **Undo Last Chunk**: Pressing `R` removes the last saved chunk (an undo feature for one step). The chunk file is deleted, and the file's highlights/progress are updated. (As a stretch goal, pressing `Shift+R` could redo the removed chunk.)
- **Real-time Updates**: The application watches the filesystem in real time. If chunk files or source files are added, removed, or changed outside the application, it detects these changes and updates the UI highlights and file list immediately.
- **Configurable Settings**: A configuration allows setting the chunk storage directory path and a maximum chunk size. Stretch goals: make keybindings (like the keys for save/undo) and color schemes configurable to suit user preferences.

---

# Development Phases

## Phase 1: File Explorer UI (Completed)

**Description**  
Implement the file explorer interface. This includes displaying a list of files/folders under the given root directory and allowing the user to navigate this list using the keyboard (with Vim-style or arrow keys). The explorer must enforce chroot mode, meaning navigation is confined to the specified root – the user should not be able to go to parent directories above the root. This phase establishes basic navigation without yet opening files.

**Technical Implementation Details**  
- Use a Rust TUI library (e.g., `ratatui`) to render a scrollable list of directory entries.
- Implement keyboard navigation:
  - `Up/Down` (or `K/J` in Vim mode) moves the selection.
  - `PageUp/PageDown` for faster scroll.
  - `Home/End` to jump to top or bottom.
  - `H/L` (Vim mode) to navigate directories.
- Enforce the root directory constraint:
  - Do not display any parent of the root.
  - When inside a subdirectory, an `..` entry or `[Parent Directory]` option can be included.
- Opening directories/files:
  - If a directory is selected, pressing `Enter` (or `L` in Vim mode) should open it.
  - If a file is selected, pressing `Enter` should trigger Phase 2 (opening the file in the text viewer).
- Dynamic file list: It will later reflect highlighting based on chunks but can display all items in a neutral color for now.

---

## Phase 2: Text Viewer & Scrolling (Completed)

**Description**  
Implement the text viewer pane that opens when a user selects a file from the explorer. In this phase, the user can view the contents of the text file and scroll through it.

**Technical Implementation Details**  
- When a file-open action is triggered, a full-screen text viewer.
- Read the file's content from disk and display it.
- Implement vertical scrolling:
  - `Up/Down` arrow (`K/J` in Vim) for line-by-line scrolling.
  - `PageUp/PageDown` for fast scrolling.
  - `Home/End` to jump to start or end.
- Allow exiting the text viewer with `Q`.

---

## Phase 3: Line-Based Text Selection & Highlighting (Completed)

**Description**  
Enhance the text viewer to allow selecting contiguous lines of text by toggling a selection mode via **Space**. When selection mode is active, moving the cursor up or down (via arrow keys or Vim-style keys) highlights the lines to be chunked. Pressing **Space** again exits selection mode.

**Technical Implementation Details**  
- Implement a line-based cursor that the user can move up or down.
- Pressing **Space** toggles selection mode on/off.
- While selection mode is active, moving the cursor updates the selection range.
- Once the user exits selection mode or performs another action, the highlighted region remains as the current selection.
- Selections should clear when a new selection starts or when certain actions are performed (e.g., saving a chunk).

---

## Phase 3.5: Vim-like Text Editing

**Description**  
Replace the basic tui-textarea implementation with EdTUI to provide a more powerful Vim-like editing experience. EdTUI offers modal editing (Normal, Insert, Visual) that better aligns with the project's use of Vim-style keybindings throughout the application.

**Technical Implementation Details**  
- Integrate the EdTUI crate to replace tui-textarea
- Implement proper modal editing:
  - Normal mode for navigation and commands
  - Insert mode for text modification
  - Visual mode for selecting text
- Maintain the edit workflow where 'E' opens selected text in the editor
- Ensure edited content is properly saved with chunks
- Update keybinding documentation to reflect Vim commands
- Maintain the separation between viewing and editing modes

**Test Cases**  
- Verify that all Vim modes (Normal, Insert, Visual) work correctly
- Test common Vim commands in each mode (navigation, editing, selection)
- Confirm proper transfer of content between viewer and editor
- Ensure edited content is properly saved in chunk files
- Test boundary conditions for selection and editing
- Verify no original files are modified during editing operations

---

## Phase 4: Chunk Saving & Tracking (In Progress)

**Description**  
Allow users to save selected lines (with or without optional edits from Phase 3.5) as a chunk to disk.

**Technical Implementation Details**  
- On `S` key press:
  - Extract the currently selected lines (post-edit if edited).
  - Save them as a chunk file in the configured directory.
  - Highlight saved lines in orange.
  - Update file explorer color-coding based on chunk progress.

**Implementation Status**
- Basic chunking functionality is implemented
- Chunk filename scheme is working correctly
- Need to implement saving edited content instead of original content
- Need to add chunk metadata tracking and UI updates

---

## Phase 5: Filesystem Monitoring

**Description**  
Detect external changes (e.g., chunk file deletion/addition) and update the UI dynamically.

**Technical Implementation Details**  
- Use Rust's `Notify` crate to monitor directories.
- Detect and reflect changes in chunk storage.
- Refresh UI dynamically without requiring a restart.

---

## Phase 6: Undo Feature

**Description**  
Allow users to undo (`R`) the last chunk save and optionally redo (`Shift+R`).

**Technical Implementation Details**  
- Maintain an undo stack for the last saved chunk.
- On `R`, remove the chunk file and update the UI.
- On `Shift+R`, restore the last undone chunk.

---

## Phase 7: Polish & Further Stretch Goals

**Description**  
Refine the user experience, add customization options, improve performance, and consider additional features.

**Technical Implementation Details**  
- Support configuration files (`TOML`, `YAML`) for custom settings such as chunk directory, keybindings, and color themes.
- Implement robust error handling for permission issues or unexpected file operations.
- Optimize performance for large files.
- Consider expansions to the editor view or improved selection heuristics.

---

# Library Notes

## Text Editing Implementation

After research, EdTUI has been identified as the preferred library for the text editing component:

- **EdTUI**: A Vim-inspired editor widget for Ratatui with modal editing
  - Provides Normal/Insert/Visual modes out of the box
  - Supports common Vim commands and motions (h/j/k/l, w/e/b, dd, y/p, etc.)
  - Includes helpful features like line numbers, syntax highlighting, search
  - MIT licensed and actively maintained
  - Direct integration with Ratatui/Crossterm
  
This will replace the current tui-textarea implementation to provide a more powerful and familiar editing experience for users accustomed to Vim.

## Implementation Strategy

1. Add EdTUI dependency to Cargo.toml
2. Create a new implementation of the Editor component using EdTUI
3. Update the app event handling to work with EdTUI's modes and commands
4. Ensure proper content transfer between viewer and editor
5. Implement saving of edited content when chunking

## Testing Strategy

Create comprehensive tests for:
- Modal editing functionality
- Content transfer between components
- File safety (ensuring no modification of original files)
- Integration with chunking workflow