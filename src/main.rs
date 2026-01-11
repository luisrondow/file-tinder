mod async_preview;
mod domain;
mod preview;
mod tui;

use async_preview::SyncPreviewManager;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use domain::{discover_files, AppState, Decision, DecisionEngine};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, path::Path, time::Duration};
use tui::{
    handle_key_event, render_help_overlay, render_summary, render_with_preview, KeyAction,
    ViewState,
};

fn main() -> io::Result<()> {
    println!("File Tinder - Terminal File Declutterer");
    Ok(())
}

/// Runs the TUI application
pub fn run_app(directory: &Path) -> io::Result<()> {
    // Discover files
    let files = discover_files(directory)?;
    if files.is_empty() {
        println!("No files found in directory");
        return Ok(());
    }

    // Initialize state
    let mut app_state = AppState::new(files.clone());
    let mut decision_engine = DecisionEngine::new(files);
    let mut preview_manager = SyncPreviewManager::new();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Main loop
    let result = run_loop(
        &mut terminal,
        &mut app_state,
        &mut decision_engine,
        &mut preview_manager,
    );

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

/// Main application loop
fn run_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app_state: &mut AppState,
    decision_engine: &mut DecisionEngine,
    preview_manager: &mut SyncPreviewManager,
) -> io::Result<()> {
    let mut view_state = ViewState::Browsing;

    loop {
        // Render based on current view state
        terminal.draw(|frame| {
            render_with_preview(frame, app_state, preview_manager);

            // Render overlays
            match view_state {
                ViewState::Help => render_help_overlay(frame),
                ViewState::Summary => {
                    let stats = decision_engine.get_statistics();
                    render_summary(frame, &stats);
                }
                ViewState::Browsing => {}
            }
        })?;

        // Handle input
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Handle overlay-specific input
                match view_state {
                    ViewState::Help => {
                        // Any key closes help (or toggle with ?)
                        let action = handle_key_event(key);
                        if matches!(action, KeyAction::Help | KeyAction::Quit | KeyAction::None) {
                            view_state = ViewState::Browsing;
                        }
                        continue;
                    }
                    ViewState::Summary => {
                        // Any key exits from summary
                        break;
                    }
                    ViewState::Browsing => {}
                }

                let action = handle_key_event(key);

                match action {
                    KeyAction::Quit => {
                        // Show summary before quitting if any decisions were made
                        let stats = decision_engine.get_statistics();
                        if stats.kept > 0 || stats.trashed > 0 {
                            view_state = ViewState::Summary;
                        } else {
                            break;
                        }
                    }
                    KeyAction::Keep => {
                        if decision_engine
                            .record_decision(app_state.current_index, Decision::Keep)
                            .is_ok()
                        {
                            app_state.record_decision(Decision::Keep);
                            app_state.next();
                            preview_manager.reset();

                            // Check if we've processed all files
                            if is_all_files_processed(app_state, decision_engine) {
                                view_state = ViewState::Summary;
                            }
                        }
                    }
                    KeyAction::Trash => {
                        if decision_engine
                            .record_decision(app_state.current_index, Decision::Trash)
                            .is_ok()
                        {
                            app_state.record_decision(Decision::Trash);
                            app_state.next();
                            preview_manager.reset();

                            // Check if we've processed all files
                            if is_all_files_processed(app_state, decision_engine) {
                                view_state = ViewState::Summary;
                            }
                        }
                    }
                    KeyAction::Next => {
                        app_state.next();
                        preview_manager.reset();
                    }
                    KeyAction::Previous => {
                        app_state.previous();
                        preview_manager.reset();
                    }
                    KeyAction::Undo => {
                        if decision_engine.undo().is_ok() {
                            app_state.undo();
                            preview_manager.reset();
                            // Return to browsing if we were in summary
                            if view_state == ViewState::Summary {
                                view_state = ViewState::Browsing;
                            }
                        }
                    }
                    KeyAction::Help => {
                        view_state = ViewState::Help;
                    }
                    KeyAction::None => {}
                }
            }
        }
    }

    Ok(())
}

/// Checks if all files have been processed
fn is_all_files_processed(app_state: &AppState, decision_engine: &DecisionEngine) -> bool {
    let stats = decision_engine.get_statistics();
    stats.kept + stats.trashed >= app_state.files.len()
}
