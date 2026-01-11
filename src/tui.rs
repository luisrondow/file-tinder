// TUI module for rendering the terminal interface
#![allow(dead_code)]

use crate::async_preview::{PreviewState, SyncPreviewManager};
use crate::domain::{AppState, DecisionStatistics};
use crate::preview;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Gauge, Paragraph, Wrap},
    Frame,
};

// Color scheme - Modern dark theme with vibrant accents
const ACCENT_PRIMARY: Color = Color::Rgb(255, 107, 107); // Coral red for trash
const ACCENT_SECONDARY: Color = Color::Rgb(107, 255, 158); // Mint green for keep
const ACCENT_HIGHLIGHT: Color = Color::Rgb(255, 217, 102); // Golden yellow for highlights
const TEXT_PRIMARY: Color = Color::Rgb(240, 240, 240); // Off-white
const TEXT_SECONDARY: Color = Color::Rgb(160, 160, 170); // Muted gray
const BG_DARK: Color = Color::Rgb(30, 30, 40); // Deep purple-black
const BORDER_COLOR: Color = Color::Rgb(80, 80, 100); // Subtle purple-gray

/// Represents the result of handling a key event
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyAction {
    /// Quit the application
    Quit,
    /// Mark current file to keep
    Keep,
    /// Mark current file to trash
    Trash,
    /// Move to next file
    Next,
    /// Move to previous file
    Previous,
    /// Undo last decision
    Undo,
    /// Toggle help overlay
    Help,
    /// No action
    None,
}

/// UI view state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewState {
    /// Main file browsing view
    Browsing,
    /// Help overlay visible
    Help,
    /// Summary screen at end
    Summary,
}

/// Maps keyboard events to actions
pub fn handle_key_event(key: KeyEvent) -> KeyAction {
    match (key.code, key.modifiers) {
        // Quit: q or Ctrl+C
        (KeyCode::Char('q'), KeyModifiers::NONE) => KeyAction::Quit,
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => KeyAction::Quit,
        (KeyCode::Esc, KeyModifiers::NONE) => KeyAction::Quit,

        // Keep: Right arrow or k
        (KeyCode::Right, KeyModifiers::NONE) => KeyAction::Keep,
        (KeyCode::Char('k'), KeyModifiers::NONE) => KeyAction::Keep,

        // Trash: Left arrow or t
        (KeyCode::Left, KeyModifiers::NONE) => KeyAction::Trash,
        (KeyCode::Char('t'), KeyModifiers::NONE) => KeyAction::Trash,

        // Navigation
        (KeyCode::Down, KeyModifiers::NONE) => KeyAction::Next,
        (KeyCode::Up, KeyModifiers::NONE) => KeyAction::Previous,
        (KeyCode::Char('j'), KeyModifiers::NONE) => KeyAction::Next,
        (KeyCode::Char('i'), KeyModifiers::NONE) => KeyAction::Previous,

        // Undo: u or Ctrl+Z
        (KeyCode::Char('u'), KeyModifiers::NONE) => KeyAction::Undo,
        (KeyCode::Char('z'), KeyModifiers::CONTROL) => KeyAction::Undo,

        // Help: ?
        (KeyCode::Char('?'), KeyModifiers::NONE) => KeyAction::Help,

        _ => KeyAction::None,
    }
}

/// Calculates progress percentage
pub fn calculate_progress(current: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        (current as f64 / total as f64) * 100.0
    }
}

/// Formats file size in human-readable format
pub fn format_file_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size >= GB {
        format!("{:.1} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.1} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.1} KB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}

/// Renders the TUI (legacy, without async preview)
pub fn render(frame: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Header with progress
            Constraint::Min(0),    // Content
            Constraint::Length(3), // Footer
        ])
        .split(frame.area());

    render_header_polished(frame, chunks[0], state);
    render_content(frame, chunks[1], state);
    render_footer_polished(frame, chunks[2]);
}

/// Renders the TUI with async preview support
pub fn render_with_preview(
    frame: &mut Frame,
    state: &AppState,
    preview_manager: &mut SyncPreviewManager,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Header with progress
            Constraint::Min(0),    // Content
            Constraint::Length(3), // Footer
        ])
        .split(frame.area());

    render_header_polished(frame, chunks[0], state);
    render_content_async(frame, chunks[1], state, preview_manager);
    render_footer_polished(frame, chunks[2]);
}

/// Renders the summary screen at the end
pub fn render_summary(frame: &mut Frame, stats: &DecisionStatistics) {
    let area = frame.area();

    // Center the summary box
    let summary_area = centered_rect(60, 50, area);

    // Clear the background
    frame.render_widget(Clear, summary_area);

    let block = Block::default()
        .title(" ✨ Session Complete ✨ ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(ACCENT_HIGHLIGHT))
        .style(Style::default().bg(BG_DARK));

    let inner = block.inner(summary_area);
    frame.render_widget(block, summary_area);

    // Build summary content
    let total = stats.total_files;
    let kept = stats.kept;
    let trashed = stats.trashed;
    let remaining = total.saturating_sub(kept + trashed);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "📊 Summary",
            Style::default()
                .fg(TEXT_PRIMARY)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::raw("   Total files:  "),
            Span::styled(
                format!("{}", total),
                Style::default()
                    .fg(ACCENT_HIGHLIGHT)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("   ✓ ", Style::default().fg(ACCENT_SECONDARY)),
            Span::raw("Kept:     "),
            Span::styled(
                format!("{}", kept),
                Style::default()
                    .fg(ACCENT_SECONDARY)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("   ✗ ", Style::default().fg(ACCENT_PRIMARY)),
            Span::raw("Trashed:  "),
            Span::styled(
                format!("{}", trashed),
                Style::default()
                    .fg(ACCENT_PRIMARY)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("   ○ ", Style::default().fg(TEXT_SECONDARY)),
            Span::raw("Skipped:  "),
            Span::styled(
                format!("{}", remaining),
                Style::default().fg(TEXT_SECONDARY),
            ),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "Press any key to exit",
            Style::default().fg(TEXT_SECONDARY),
        )),
    ];

    let paragraph = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .style(Style::default().fg(TEXT_PRIMARY));

    frame.render_widget(paragraph, inner);
}

/// Renders the help overlay
pub fn render_help_overlay(frame: &mut Frame) {
    let area = frame.area();
    let help_area = centered_rect(50, 70, area);

    // Clear background
    frame.render_widget(Clear, help_area);

    let block = Block::default()
        .title(" ❓ Help ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(ACCENT_HIGHLIGHT))
        .style(Style::default().bg(BG_DARK));

    let inner = block.inner(help_area);
    frame.render_widget(block, help_area);

    let help_lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Keyboard Shortcuts",
            Style::default()
                .fg(ACCENT_HIGHLIGHT)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  → ", Style::default().fg(ACCENT_SECONDARY)),
            Span::raw("or "),
            Span::styled("k", Style::default().fg(ACCENT_SECONDARY)),
            Span::raw("     Keep file"),
        ]),
        Line::from(vec![
            Span::styled("  ← ", Style::default().fg(ACCENT_PRIMARY)),
            Span::raw("or "),
            Span::styled("t", Style::default().fg(ACCENT_PRIMARY)),
            Span::raw("     Trash file"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ↑↓ ", Style::default().fg(TEXT_SECONDARY)),
            Span::raw("or "),
            Span::styled("i/j", Style::default().fg(TEXT_SECONDARY)),
            Span::raw("   Navigate"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  u ", Style::default().fg(ACCENT_HIGHLIGHT)),
            Span::raw("or "),
            Span::styled("Ctrl+Z", Style::default().fg(ACCENT_HIGHLIGHT)),
            Span::raw("  Undo"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  q ", Style::default().fg(TEXT_SECONDARY)),
            Span::raw("or "),
            Span::styled("Esc", Style::default().fg(TEXT_SECONDARY)),
            Span::raw("     Quit"),
        ]),
        Line::from(vec![
            Span::styled("  ?", Style::default().fg(TEXT_SECONDARY)),
            Span::raw("           Toggle help"),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "Press ? or Esc to close",
            Style::default().fg(TEXT_SECONDARY),
        )),
    ];

    let paragraph = Paragraph::new(help_lines)
        .alignment(Alignment::Center)
        .style(Style::default().fg(TEXT_PRIMARY));

    frame.render_widget(paragraph, inner);
}

/// Helper to create a centered rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Renders the header with file info (legacy)
fn render_header(frame: &mut Frame, area: Rect, state: &AppState) {
    render_header_polished(frame, area, state);
}

/// Renders the polished header with progress bar
fn render_header_polished(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(2)])
        .split(area);

    // Title and file info
    let (title_text, file_info) = if let Some(file) = state.current_file() {
        let size_str = format_file_size(file.size);
        let file_type = format!("{:?}", file.file_type);
        (
            format!(
                " 🗂  File {}/{} ",
                state.current_index + 1,
                state.files.len()
            ),
            vec![
                Span::styled(
                    &file.name,
                    Style::default()
                        .fg(TEXT_PRIMARY)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(
                    format!("({} • {})", size_str, file_type),
                    Style::default().fg(TEXT_SECONDARY),
                ),
            ],
        )
    } else {
        (
            " 🗂  File Tinder ".to_string(),
            vec![Span::styled(
                "No files to review",
                Style::default().fg(TEXT_SECONDARY),
            )],
        )
    };

    let title_line = Line::from(vec![Span::styled(
        title_text,
        Style::default()
            .fg(ACCENT_HIGHLIGHT)
            .add_modifier(Modifier::BOLD),
    )]);

    let info_line = Line::from(file_info);

    let header = Paragraph::new(vec![title_line, info_line])
        .block(
            Block::default()
                .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(BORDER_COLOR)),
        )
        .alignment(Alignment::Left);

    frame.render_widget(header, chunks[0]);

    // Progress bar
    let total = state.files.len();
    let processed = state.decisions_stack.len();
    let progress = if total > 0 {
        processed as f64 / total as f64
    } else {
        0.0
    };

    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(BORDER_COLOR)),
        )
        .gauge_style(Style::default().fg(ACCENT_SECONDARY).bg(BG_DARK))
        .ratio(progress)
        .label(format!(
            "{}% ({}/{})",
            (progress * 100.0) as u16,
            processed,
            total
        ));

    frame.render_widget(gauge, chunks[1]);
}

/// Renders the main content area (synchronous version)
fn render_content(frame: &mut Frame, area: Rect, state: &AppState) {
    let content = if let Some(file) = state.current_file() {
        // Generate file preview
        let preview_lines = match preview::generate_preview(file) {
            Ok(lines) => lines,
            Err(e) => vec![
                format!("Error generating preview: {}", e),
                String::new(),
                format!("File: {}", file.name),
                format!("Path: {}", file.path.display()),
                format!("Size: {} bytes", file.size),
                format!("Type: {:?}", file.file_type),
            ],
        };

        // Convert strings to Lines
        let lines: Vec<Line> = preview_lines.into_iter().map(Line::from).collect();

        Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(BORDER_COLOR))
                    .title(format!(" 📄 {} ", file.name)),
            )
            .style(Style::default().fg(TEXT_PRIMARY))
            .wrap(Wrap { trim: false })
    } else {
        render_empty_state_widget()
    };

    frame.render_widget(content, area);
}

/// Creates an empty state widget for when no files are present
fn render_empty_state_widget() -> Paragraph<'static> {
    let lines = vec![
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "📂 No Files Found",
            Style::default()
                .fg(ACCENT_HIGHLIGHT)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "The directory is empty or contains only hidden files.",
            Style::default().fg(TEXT_SECONDARY),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Try a different directory with visible files.",
            Style::default().fg(TEXT_SECONDARY),
        )),
    ];

    Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(BORDER_COLOR))
                .title(" Content "),
        )
        .alignment(Alignment::Center)
}

/// Renders the main content area with async preview loading
fn render_content_async(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    preview_manager: &mut SyncPreviewManager,
) {
    let content = if let Some(file) = state.current_file() {
        // Get preview state from manager
        let preview_state = preview_manager.request_preview(file);

        let (preview_lines, title_suffix, border_color) = match preview_state {
            PreviewState::Loading => {
                let loading_lines = generate_loading_indicator(file);
                (loading_lines, " ⏳", ACCENT_HIGHLIGHT)
            }
            PreviewState::Ready(lines) => (lines.clone(), "", BORDER_COLOR),
            PreviewState::Error(e) => {
                let error_lines = vec![
                    String::new(),
                    format!("  ⚠️  Error generating preview"),
                    String::new(),
                    format!("  {}", e),
                    String::new(),
                    format!("  File: {}", file.name),
                    format!("  Path: {}", file.path.display()),
                    format!("  Size: {}", format_file_size(file.size)),
                    format!("  Type: {:?}", file.file_type),
                ];
                (error_lines, " ⚠️", ACCENT_PRIMARY)
            }
        };

        // Convert strings to Lines
        let lines: Vec<Line> = preview_lines.into_iter().map(Line::from).collect();

        Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(border_color))
                    .title(format!(" 📄 {}{} ", file.name, title_suffix)),
            )
            .style(Style::default().fg(TEXT_PRIMARY))
            .wrap(Wrap { trim: false })
    } else {
        render_empty_state_widget()
    };

    frame.render_widget(content, area);
}

/// Generates a loading indicator for file preview
fn generate_loading_indicator(file: &crate::domain::FileEntry) -> Vec<String> {
    vec![
        String::new(),
        String::new(),
        format!("  📂 Loading preview..."),
        String::new(),
        format!("  ⏳ Processing: {}", file.name),
        String::new(),
        format!("  Type: {:?}", file.file_type),
        format!("  Size: {}", format_file_size(file.size)),
        format!("  Path: {}", file.path.display()),
    ]
}

/// Renders the footer with controls (legacy)
fn render_footer(frame: &mut Frame, area: Rect) {
    render_footer_polished(frame, area);
}

/// Renders the polished footer with styled controls
fn render_footer_polished(frame: &mut Frame, area: Rect) {
    let controls = Line::from(vec![
        Span::styled(
            " ← ",
            Style::default()
                .fg(ACCENT_PRIMARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Trash", Style::default().fg(TEXT_SECONDARY)),
        Span::raw("  │  "),
        Span::styled(
            "→ ",
            Style::default()
                .fg(ACCENT_SECONDARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Keep", Style::default().fg(TEXT_SECONDARY)),
        Span::raw("  │  "),
        Span::styled("↑↓ ", Style::default().fg(TEXT_SECONDARY)),
        Span::styled("Navigate", Style::default().fg(TEXT_SECONDARY)),
        Span::raw("  │  "),
        Span::styled("u ", Style::default().fg(ACCENT_HIGHLIGHT)),
        Span::styled("Undo", Style::default().fg(TEXT_SECONDARY)),
        Span::raw("  │  "),
        Span::styled("? ", Style::default().fg(TEXT_SECONDARY)),
        Span::styled("Help", Style::default().fg(TEXT_SECONDARY)),
        Span::raw("  │  "),
        Span::styled("q ", Style::default().fg(TEXT_SECONDARY)),
        Span::styled("Quit", Style::default().fg(TEXT_SECONDARY)),
    ]);

    let footer = Paragraph::new(controls)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(BORDER_COLOR)),
        )
        .alignment(Alignment::Center);

    frame.render_widget(footer, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{FileEntry, FileType};
    use chrono::Utc;
    use crossterm::event::KeyModifiers;
    use ratatui::{backend::TestBackend, Terminal};
    use std::path::PathBuf;

    fn create_test_entry(name: &str) -> FileEntry {
        FileEntry {
            path: PathBuf::from(name),
            name: name.to_string(),
            size: 1024,
            modified_date: Utc::now(),
            file_type: FileType::Text,
        }
    }

    mod key_handling_tests {
        use super::*;

        #[test]
        fn test_key_quit() {
            let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
            assert_eq!(handle_key_event(key), KeyAction::Quit);

            let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
            assert_eq!(handle_key_event(key), KeyAction::Quit);

            let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
            assert_eq!(handle_key_event(key), KeyAction::Quit);
        }

        #[test]
        fn test_key_keep() {
            let key = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
            assert_eq!(handle_key_event(key), KeyAction::Keep);

            let key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
            assert_eq!(handle_key_event(key), KeyAction::Keep);
        }

        #[test]
        fn test_key_trash() {
            let key = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
            assert_eq!(handle_key_event(key), KeyAction::Trash);

            let key = KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE);
            assert_eq!(handle_key_event(key), KeyAction::Trash);
        }

        #[test]
        fn test_key_navigation() {
            let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
            assert_eq!(handle_key_event(key), KeyAction::Next);

            let key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
            assert_eq!(handle_key_event(key), KeyAction::Previous);

            let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
            assert_eq!(handle_key_event(key), KeyAction::Next);

            let key = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE);
            assert_eq!(handle_key_event(key), KeyAction::Previous);
        }

        #[test]
        fn test_key_undo() {
            let key = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::NONE);
            assert_eq!(handle_key_event(key), KeyAction::Undo);

            let key = KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL);
            assert_eq!(handle_key_event(key), KeyAction::Undo);
        }

        #[test]
        fn test_key_none() {
            let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
            assert_eq!(handle_key_event(key), KeyAction::None);

            let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
            assert_eq!(handle_key_event(key), KeyAction::None);
        }

        #[test]
        fn test_key_help() {
            let key = KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE);
            assert_eq!(handle_key_event(key), KeyAction::Help);
        }
    }

    mod layout_tests {
        use super::*;

        #[test]
        fn test_render_empty_state() {
            let state = AppState::new(vec![]);
            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).unwrap();

            terminal
                .draw(|frame| {
                    render(frame, &state);
                })
                .unwrap();

            // Verify no panics and rendering succeeds
            let buffer = terminal.backend().buffer().clone();
            let content = buffer.content();

            // Check for empty state message in content
            let buffer_str: String = content.iter().map(|c| c.symbol()).collect();
            // The new empty state shows "No Files Found" or contains "empty"
            assert!(
                buffer_str.contains("No Files") || buffer_str.contains("empty"),
                "Expected empty state message, got: {}",
                buffer_str
            );
        }

        #[test]
        fn test_render_with_files() {
            let files = vec![
                create_test_entry("file1.txt"),
                create_test_entry("file2.rs"),
            ];
            let state = AppState::new(files);
            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).unwrap();

            terminal
                .draw(|frame| {
                    render(frame, &state);
                })
                .unwrap();

            let buffer = terminal.backend().buffer().clone();
            let content = buffer.content();
            let buffer_str: String = content.iter().map(|c| c.symbol()).collect();

            // Check for file info in buffer
            assert!(buffer_str.contains("file1.txt"));
            assert!(buffer_str.contains("File 1/2"));
        }

        #[test]
        fn test_render_footer() {
            let state = AppState::new(vec![create_test_entry("test.txt")]);
            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).unwrap();

            terminal
                .draw(|frame| {
                    render(frame, &state);
                })
                .unwrap();

            let buffer = terminal.backend().buffer().clone();
            let content = buffer.content();
            let buffer_str: String = content.iter().map(|c| c.symbol()).collect();

            // Check for controls in footer
            assert!(buffer_str.contains("Trash"));
            assert!(buffer_str.contains("Keep"));
            assert!(buffer_str.contains("Quit"));
        }

        #[test]
        fn test_render_header_progress() {
            let files = vec![
                create_test_entry("file1.txt"),
                create_test_entry("file2.txt"),
                create_test_entry("file3.txt"),
            ];
            let mut state = AppState::new(files);
            state.next();

            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).unwrap();

            terminal
                .draw(|frame| {
                    render(frame, &state);
                })
                .unwrap();

            let buffer = terminal.backend().buffer().clone();
            let content = buffer.content();
            let buffer_str: String = content.iter().map(|c| c.symbol()).collect();

            // Check that we're on file 2 of 3
            assert!(buffer_str.contains("File 2/3"));
            assert!(buffer_str.contains("file2.txt"));
        }

        #[test]
        fn test_render_help_overlay() {
            let backend = TestBackend::new(80, 30);
            let mut terminal = Terminal::new(backend).unwrap();

            terminal
                .draw(|frame| {
                    render_help_overlay(frame);
                })
                .unwrap();

            let buffer = terminal.backend().buffer().clone();
            let content = buffer.content();
            let buffer_str: String = content.iter().map(|c| c.symbol()).collect();

            // Check for help content
            assert!(buffer_str.contains("Help"));
            assert!(buffer_str.contains("Keep"));
            assert!(buffer_str.contains("Trash"));
        }

        #[test]
        fn test_render_summary() {
            let stats = DecisionStatistics {
                total_files: 10,
                kept: 6,
                trashed: 3,
            };

            let backend = TestBackend::new(80, 30);
            let mut terminal = Terminal::new(backend).unwrap();

            terminal
                .draw(|frame| {
                    render_summary(frame, &stats);
                })
                .unwrap();

            let buffer = terminal.backend().buffer().clone();
            let content = buffer.content();
            let buffer_str: String = content.iter().map(|c| c.symbol()).collect();

            // Check for summary content
            assert!(buffer_str.contains("Summary") || buffer_str.contains("Complete"));
        }
    }

    mod utility_tests {
        use super::*;

        #[test]
        fn test_calculate_progress() {
            assert_eq!(calculate_progress(0, 10), 0.0);
            assert_eq!(calculate_progress(5, 10), 50.0);
            assert_eq!(calculate_progress(10, 10), 100.0);
            assert_eq!(calculate_progress(0, 0), 0.0);
        }

        #[test]
        fn test_format_file_size() {
            assert_eq!(format_file_size(0), "0 B");
            assert_eq!(format_file_size(512), "512 B");
            assert_eq!(format_file_size(1024), "1.0 KB");
            assert_eq!(format_file_size(1536), "1.5 KB");
            assert_eq!(format_file_size(1048576), "1.0 MB");
            assert_eq!(format_file_size(1073741824), "1.0 GB");
        }

        #[test]
        fn test_view_state_enum() {
            assert_eq!(ViewState::Browsing, ViewState::Browsing);
            assert_ne!(ViewState::Browsing, ViewState::Help);
            assert_ne!(ViewState::Help, ViewState::Summary);
        }
    }
}
