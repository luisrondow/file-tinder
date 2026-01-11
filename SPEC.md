Here is the comprehensive specification for your terminal-based file decluttering app, summarizing our brainstorming session, technical decisions, and architectural choices.

### `file-tinder-specs.md`

# Project Specification: Terminal File Declutterer ("File Tinder")

## 1. Project Vision
A fast, keyboard-centric terminal application built in Rust that allows users to clean up directories using a "Tinder-like" interface. Users view files one by one and make rapid decisions to "Keep" or "Trash" them, with rich file previews directly in the terminal.

## 2. Core Tech Stack
*   **Language:** Rust (chosen for performance and safety).
*   **TUI Framework:** `ratatui` (formerly `tui-rs`) for the interface.[1][2]
*   **Terminal Events:** `crossterm` for handling raw input and rendering backend.
*   **File Deletion:** `trash` crate to safely move files to the system Recycle Bin/Trash instead of permanent deletion.[3][4]
*   **Async Runtime:** `tokio` to handle heavy operations (like PDF rendering) without freezing the UI.[5][6]

## 3. User Interface (UX)
### The "Tinder View" Layout
*   **Single Card Focus:** The interface focuses on one file at a time, maximizing screen real estate for the preview.
*   **Visual Hierarchy:**
    *   **Header:** File Name, File Size, Modification Date.
    *   **Center:** Large content preview (Image, Text, or PDF page).
    *   **Footer:** Controls/Status bar (e.g., `< Trash | Keep >`).

### Controls & Input
*   **Navigation:**
    *   `Left Arrow` / `h`: **Trash** (Move to system trash).
    *   `Right Arrow` / `l`: **Keep** (Skip to next file).
    *   `Backspace` / `u`: **Undo** (Reverse the last action - *Recommended Feature*).
    *   `q` / `Esc`: **Quit** application.

## 4. Preview Strategy
To ensure a "beautiful" experience, the app handles different file types distinctly:

| File Type | Rendering Strategy | Libraries |
| :--- | :--- | :--- |
| **Plain Text / Code** | Syntax highlighted text. Shows the first ~50 lines. | `syntect` (for highlighting) [7][8]. |
| **Images** | Renders directly in terminal using high-res protocols (Sixel, Kitty) with ASCII/block fallback. | `ratatui-image` [9][10]. |
| **PDF Documents** | Renders the first page of the PDF into an in-memory image, then displays it using the Image strategy. | `pdfium-render` (**Option A** - Static Binding) [11][12]. |
| **Others** | Displays metadata generic icon (e.g., "Binary File"). | N/A |

## 5. Logic & Behavior
*   **"Keep" Logic:** *Survivor Mode*. Files marked as "Keep" are left untouched in the directory. The app simply advances to the next file.
*   **"Trash" Logic:** Files are moved to the OS-specific Trash immediately upon action (or queued, depending on safety preference).
*   **PDF Implementation:** Uses `pdfium-render` with `static` linking where possible to bundle the PDF engine, avoiding external user dependencies like `poppler`.[12][13]
*   **Concurrency:** Image generation happens on a separate Tokio thread to keep the interface responsive during swiping.

## 6. Implementation Roadmap
1.  **Project Setup:** Initialize `cargo` with `ratatui`, `tokio`, and `crossterm`.
2.  **State Management:** Build the `App` struct to hold the list of files and current index.
3.  **The Viewer:** Implement `ratatui-image` and `pdfium-render` integration.
4.  **Interaction Loop:** Connect keyboard events to the `trash` and `next_file` functions.
5.  **Polish:** Add "Undo" stack and nice borders/colors.
