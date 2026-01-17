use crate::app::{App, AppState};
use crate::command::Command;
use crate::config::AppConfig;
use crate::domain::types::{PrContext, PrListItem, RepoListItem, ReviewAction, Story};

use super::helpers;

pub fn handle_repo_list_loaded(app: &mut App, result: Result<Vec<RepoListItem>, String>) -> Vec<Command> {
    match result {
        Ok(repo_list) => {
            app.repo_list = repo_list;
            app.repo_selected = 0;
            app.state = AppState::RepoSelector;
            app.show_picker = false;
            Vec::new()
        }
        Err(err) => {
            app.state = AppState::Error(format!("Failed to fetch repo list: {}", err));
            Vec::new()
        }
    }
}

pub fn handle_pr_list_loaded(app: &mut App, result: Result<Vec<PrListItem>, String>) -> Vec<Command> {
    match result {
        Ok(pr_list) => {
            app.pr_list = pr_list;
            app.picker_selected = 0;
            app.state = AppState::PrPicker;
            app.show_picker = app.story.is_some();
            Vec::new()
        }
        Err(err) => {
            app.state = AppState::Error(format!("Failed to fetch PR list: {}", err));
            Vec::new()
        }
    }
}

pub fn handle_pr_loaded(app: &mut App, result: Result<PrContext, String>) -> Vec<Command> {
    match result {
        Ok(pr) => {
            app.current_repo = Some((pr.owner.clone(), pr.repo.clone()));
            app.current_pr_number = Some(pr.number);
            app.pr = Some(pr.clone());
            app.state = AppState::GeneratingStory;
            vec![Command::GenerateStory { pr }]
        }
        Err(err) => {
            app.state = AppState::Error(err);
            Vec::new()
        }
    }
}

pub fn handle_story_generated(
    app: &mut App,
    result: Result<Story, String>,
    config: &AppConfig,
) -> Vec<Command> {
    match result {
        Ok(story) => {
            app.populate_from_story(&story);
            app.story = Some(story.clone());
            app.state = AppState::Viewing;
            app.show_picker = false;
            vec![Command::SaveCache {
                path: config.cache_file.clone(),
                story,
            }]
        }
        Err(err) => {
            app.state = AppState::Error(err);
            Vec::new()
        }
    }
}

pub fn handle_cache_loaded(app: &mut App, story: Option<Story>) -> Vec<Command> {
    match story {
        Some(story) => {
            app.populate_from_story(&story);
            app.story = Some(story);
            app.state = AppState::Viewing;
            app.show_picker = false;
            app.status = Some("Loaded from cache".to_string());
            helpers::ensure_cached_pr_context(app);
            Vec::new()
        }
        None => {
            if let Some((owner, repo, number)) = helpers::current_pr_ref(app) {
                app.state = AppState::LoadingPr;
                vec![Command::FetchPr { owner, repo, number }]
            } else {
                app.state = AppState::Error("Missing PR context".to_string());
                Vec::new()
            }
        }
    }
}

pub fn handle_submission_result(
    app: &mut App,
    action: ReviewAction,
    result: Result<(), String>,
) -> Vec<Command> {
    match result {
        Ok(()) => {
            app.status = Some(format!("{} submitted successfully!", action.title()));
        }
        Err(err) => {
            app.status = Some(format!("Error: {}", err));
        }
    }
    app.state = AppState::Viewing;
    app.show_picker = false;
    Vec::new()
}
