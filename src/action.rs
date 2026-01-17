use crossterm::event::{KeyCode, KeyModifiers};

use crate::domain::types::{PrContext, PrListItem, RepoListItem, ReviewAction, Story};

#[derive(Debug)]
pub enum Action {
    Input { code: KeyCode, modifiers: KeyModifiers },
    RepoListLoaded(Result<Vec<RepoListItem>, String>),
    PrListLoaded(Result<Vec<PrListItem>, String>),
    PrLoaded(Result<PrContext, String>),
    StoryGenerated(Result<Story, String>),
    CacheLoaded(Option<Story>),
    SubmissionResult {
        action: ReviewAction,
        result: Result<(), String>,
    },
}
