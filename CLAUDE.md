---
The below project spec is your persistent guide. Do not change it.
Git commands will be handled by the user. Do not use them.
If you need to install a package, ask the user to do it for you.
Do not write outside this directory.
Keep your todo list in todo.txt
---


# Abstract

This project is a terminal user interface (TUI) application for interactively chunking text files. The tool lets users split large text files into smaller chunks by selecting text in a console UI and saving those selections as separate files. It provides a file explorer and text viewer with visual cues to track chunking progress. Key user stories and requirements include:

- **File Navigation**: A user can navigate through files and directories in a file explorer, restricted to a given root directory (chroot mode) to prevent leaving the allowed path.
- **Progress Highlighting**: The explorer indicates each file’s chunking progress with color highlights – partially chunked files in yellow, mostly chunked in orange, and fully chunked in green for quick visual status.
- **Text Viewing**: A user can open a selected text file in a viewer pane and scroll through its content (using keyboard arrows/Vim keys for up/down, or other typical scrolling keys).
- **Text Selection**: A user can select text within the viewer either with the mouse (click and drag to highlight) or the keyboard (holding Shift while using arrow keys or Vim movement keys). The selection is visually highlighted in the text viewer.
- **Save Chunks**: Pressing `S` will save the currently selected text as a chunk file in a designated storage directory. Each saved chunk is written to disk persistently.
- **Highlight Saved Chunks**: Once saved, the chunked text segment in the viewer is highlighted (e.g., in orange) to indicate it has been chunked. This helps avoid re-selecting the same text. The file’s entry in the explorer also updates its color if the new chunk changes its overall chunking status.
- **Undo Last Chunk**: Pressing `R` removes the last saved chunk (an undo feature for one step). The chunk file is deleted, and the file’s highlights/progress are updated. (As a stretch goal, pressing `Shift+R` could redo the removed chunk.)
- **Real-time Updates**: The application watches the filesystem in real time. If chunk files or source files are added, removed, or changed outside the application, it detects these changes and updates the UI highlights and file list immediately.
- **Configurable Settings**: A configuration allows setting the chunk storage directory path and a maximum chunk size. Stretch goals: make keybindings (like the keys for save/undo) and color schemes configurable to suit user preferences.

# Development Phases

The implementation will be broken down into manageable phases, each with specific features, technical considerations, and tests. This phased approach ensures incremental development and verification at each step.

## Phase 1: File Explorer UI

### Description
Implement the file explorer interface. This includes displaying a list of files/folders under the given root directory and allowing the user to navigate this list using the keyboard (with Vim-style or arrow keys). The explorer must enforce chroot mode, meaning navigation is confined to the specified root – the user should not be able to go to parent directories above the root. This phase establishes basic navigation without yet opening files.

### Technical Implementation Details
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

### Test Cases
- Navigate up and down the list and ensure correct selection movement.
- Attempt to navigate above the root directory and verify it remains within bounds.
- Enter a subdirectory and then navigate back to root.
- Verify that files vs directories are indicated properly.
- Select a file and press `Enter` to confirm it registers the file-open action (stubbed for Phase 2).

## Phase 2: Text Viewer & Scrolling

### Description
Implement the text viewer pane that opens when a user selects a file from the explorer. In this phase, the user can view the contents of the text file and scroll through it.

### Technical Implementation Details
- When a file-open action is triggered, a full-screen text viewer.
- Read the file’s content from disk and display it.
- Implement vertical scrolling:
  - `Up/Down` arrow (`K/J` in Vim) for line-by-line scrolling.
  - `PageUp/PageDown` for fast scrolling.
  - `Home/End` to jump to start or end.
- Allow exiting the text viewer with `Q`.

### Test Cases
- Open a known text file and verify displayed content.
- Scroll through the file and confirm expected behavior.
- Test boundary conditions (no over-scrolling).
- Exit and re-enter the viewer to verify correct state persistence.

## Phase 3: Text Selection & Highlighting

### Description
Enable the user to select a region of text using the mouse (click and drag) or keyboard (`Shift+Arrow keys`).

### Technical Implementation Details
- Capture mouse and keyboard events to track selection range.
- Highlight selected text distinctly.
- Handle multi-line selections correctly.
- Ensure selections clear when a new selection starts or an action is performed.

### Test Cases
- Verify accurate selection with the mouse.
- Test keyboard selection accuracy.
- Ensure selection cancels correctly.
- Verify multi-line selection behavior.

## Phase 4: Chunk Saving & Tracking

### Description
Allow users to save selected text chunks to disk.

### Technical Implementation Details
- On `S` key press:
  - Extract selected text.
  - Save it as a chunk file in the configured directory.
  - Highlight saved text in orange.
  - Update file explorer color-coding based on chunk progress.

### Test Cases
- Ensure saved chunks are written to disk correctly.
- Verify that chunked text is highlighted.
- Check that file explorer updates chunking progress colors.

## Phase 5: Filesystem Monitoring

### Description
Detect external changes (e.g., chunk file deletion/addition) and update UI dynamically.

### Technical Implementation Details
- Use Rust’s `Notify` crate to monitor directories.
- Detect and reflect changes in chunk storage.
- Refresh UI dynamically without requiring a restart.

### Test Cases
- Add/remove files externally and verify real-time updates.
- Manually delete a chunk file and confirm UI updates correctly.

## Phase 6: Undo Feature

### Description
Allow users to undo (`R`) the last chunk save and optionally redo (`Shift+R`).

### Technical Implementation Details
- Maintain an undo stack for the last saved chunk.
- On `R`, remove the chunk file and update UI.
- On `Shift+R`, restore the last undone chunk.

### Test Cases
- Undo a saved chunk and verify it disappears.
- Redo an undone chunk and ensure restoration.
- Ensure undo/redo logic prevents unintended redoing of old actions.

## Phase 7: Polish & Stretch Goals

### Description
Refine UX, add customization options, and improve performance.

### Technical Implementation Details
- Support configuration files (`TOML`, `YAML`).
- Allow users to change keybindings and colors.
- Implement error handling and performance optimizations.

### Test Cases
- Validate config file settings (e.g., custom chunk directory).
- Ensure error handling for permission issues.
- Confirm UI layout remains stable on window resizing.

---

# Conclusion

Each phase ensures incremental development and testing to build a stable and user-friendly TUI text chunking tool. The final version will provide efficient file navigation, selection, chunking, real-time updates, and customization options.
