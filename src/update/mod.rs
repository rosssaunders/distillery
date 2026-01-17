mod actions;
mod editing;
mod error;
mod helpers;
mod loading;
mod picker;
mod repo;
mod viewing;

use crate::action::Action;
use crate::app::{App, AppState};
use crate::command::Command;
use crate::config::AppConfig;

pub fn update(app: &mut App, action: Action, config: &AppConfig) -> Vec<Command> {
    match action {
        Action::Input { code, modifiers } => match &app.state {
            AppState::RepoSelector => repo::handle_input(app, code),
            AppState::PrPicker => picker::handle_input(app, code),
            AppState::Viewing => viewing::handle_input(app, code, modifiers),
            AppState::EditingAction(_) => editing::handle_input(app, code, modifiers),
            AppState::Error(_) => error::handle_input(app, code),
            AppState::LoadingRepoList
            | AppState::LoadingPrList
            | AppState::LoadingPr
            | AppState::GeneratingStory
            | AppState::Submitting(_) => loading::handle_input(app, code),
        },
        Action::RepoListLoaded(result) => actions::handle_repo_list_loaded(app, result),
        Action::PrListLoaded(result) => actions::handle_pr_list_loaded(app, result),
        Action::PrLoaded(result) => actions::handle_pr_loaded(app, result),
        Action::StoryGenerated(result) => actions::handle_story_generated(app, result, config),
        Action::CacheLoaded(story) => actions::handle_cache_loaded(app, story),
        Action::SubmissionResult { action, result } => {
            actions::handle_submission_result(app, action, result)
        }
    }
}
