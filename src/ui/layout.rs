use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::app::{App, AppState};
use crate::ui::components::{
    document, error, header, keybindings, loading, picker, repo_selector, sidebar,
};

/// Main render function
pub fn render(frame: &mut Frame, app: &App) {
    let size = frame.area();

    // Split into main area and bottom keybindings bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),    // Main content
            Constraint::Length(3), // Keybindings bar
        ])
        .split(size);

    let main_area = chunks[0];
    let keys_area = chunks[1];

    // Render main content based on state
    match &app.state {
        AppState::LoadingRepoList => {
            loading::render_loading(frame, main_area, "Fetching repositories...")
        }
        AppState::RepoSelector => repo_selector::render_repo_selector(frame, app, main_area),
        AppState::LoadingPrList => {
            loading::render_loading(frame, main_area, "Fetching PR list...")
        }
        AppState::LoadingPr => loading::render_loading(frame, main_area, "Fetching PR from GitHub..."),
        AppState::GeneratingStory => {
            loading::render_loading(frame, main_area, "Generating story with AI...")
        }
        AppState::Error(msg) => error::render_error(frame, main_area, msg),
        AppState::PrPicker => picker::render_picker(frame, app, main_area),
        AppState::Viewing | AppState::EditingAction(_) | AppState::Submitting(_) => {
            render_main(frame, app, main_area);
            // Show picker as overlay if open
            if app.show_picker {
                picker::render_picker_overlay(frame, app, main_area);
            }
        }
    }

    // Always render keybindings bar at bottom
    keybindings::render_keybindings(frame, app, keys_area);
}

fn render_main(frame: &mut Frame, app: &App, area: Rect) {
    // Split into header and content area
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Header (app name, repo, title)
            Constraint::Min(10),   // Content area
        ])
        .split(area);

    header::render_header(frame, app, vertical_chunks[0]);

    // Split content into sidebar and main document
    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(32), // Sidebar
            Constraint::Min(50),    // Main content
        ])
        .split(vertical_chunks[1]);

    sidebar::render_sidebar(frame, app, horizontal_chunks[0]);
    document::render_document(frame, app, horizontal_chunks[1]);
}
