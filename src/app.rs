use std::collections::HashSet;

use crate::types::{PrContext, PrListItem, RepoListItem, ReviewAction, Story};

/// Application state
#[derive(Debug, Clone)]
pub enum AppState {
    /// Repo selector screen
    RepoSelector,
    /// Loading repo list
    LoadingRepoList,
    /// PR picker popup
    PrPicker,
    /// Loading PR list
    LoadingPrList,
    /// Loading PR data from GitHub
    LoadingPr,
    /// Generating story from LLM
    GeneratingStory,
    /// Main story view
    Viewing,
    /// Editing an action text
    EditingAction(ReviewAction),
    /// Submitting an action
    Submitting(ReviewAction),
    /// Error state
    Error(String),
}

/// The main application
pub struct App {
    /// Current state
    pub state: AppState,
    /// PR context (after loading)
    pub pr: Option<PrContext>,
    /// Generated story (after LLM call)
    pub story: Option<Story>,
    /// Currently selected feature index
    pub selected_feature: usize,
    /// Currently selected diff index within feature
    pub selected_diff: usize,
    /// Currently selected action
    pub selected_action: ReviewAction,
    /// Scroll offset for the feature view
    pub scroll_offset: u16,
    /// Text content for each action
    pub action_texts: ActionTexts,
    /// Cursor position in text editor
    pub cursor_pos: usize,
    /// Status message
    pub status: Option<String>,
    /// Should quit
    pub should_quit: bool,
    /// Set of viewed diffs: (feature_idx, diff_idx)
    pub viewed_diffs: HashSet<(usize, usize)>,
    /// PR list for picker
    pub pr_list: Vec<PrListItem>,
    /// Selected index in PR picker
    pub picker_selected: usize,
    /// Whether picker is showing
    pub show_picker: bool,
    /// Repo list for selector
    pub repo_list: Vec<RepoListItem>,
    /// Selected index in repo selector
    pub repo_selected: usize,
    /// Currently selected repo (owner, name)
    pub current_repo: Option<(String, String)>,
}

/// Text content for the three review actions
#[derive(Debug, Clone, Default)]
pub struct ActionTexts {
    pub request_changes: String,
    pub clarification: String,
    pub next_pr: String,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::LoadingPr,
            pr: None,
            story: None,
            selected_feature: 0,
            selected_diff: 0,
            selected_action: ReviewAction::RequestChanges,
            scroll_offset: 0,
            action_texts: ActionTexts::default(),
            cursor_pos: 0,
            status: None,
            should_quit: false,
            viewed_diffs: HashSet::new(),
            pr_list: Vec::new(),
            picker_selected: 0,
            show_picker: false,
            repo_list: Vec::new(),
            repo_selected: 0,
            current_repo: None,
        }
    }

    /// Create app starting with repo selector
    pub fn new_with_repo_selector() -> Self {
        let mut app = Self::new();
        app.state = AppState::LoadingRepoList;
        app
    }

    /// Create app starting with PR picker for a specific repo
    pub fn new_with_picker(owner: &str, repo: &str) -> Self {
        let mut app = Self::new();
        app.state = AppState::LoadingPrList;
        app.show_picker = true;
        app.current_repo = Some((owner.to_string(), repo.to_string()));
        app
    }

    /// Get the current action text
    pub fn current_action_text(&self) -> &str {
        match self.selected_action {
            ReviewAction::RequestChanges => &self.action_texts.request_changes,
            ReviewAction::ClarificationQuestions => &self.action_texts.clarification,
            ReviewAction::NextPr => &self.action_texts.next_pr,
        }
    }

    /// Get mutable reference to current action text
    pub fn current_action_text_mut(&mut self) -> &mut String {
        match self.selected_action {
            ReviewAction::RequestChanges => &mut self.action_texts.request_changes,
            ReviewAction::ClarificationQuestions => &mut self.action_texts.clarification,
            ReviewAction::NextPr => &mut self.action_texts.next_pr,
        }
    }

    /// Populate action texts from story
    pub fn populate_from_story(&mut self, story: &Story) {
        self.action_texts.request_changes = story.suggested_changes.clone();
        self.action_texts.clarification = story.clarification_questions.clone();
        self.action_texts.next_pr = story.next_pr.clone();
    }

    /// Move to next feature
    pub fn next_feature(&mut self) {
        if let Some(story) = &self.story {
            if self.selected_feature < story.narrative.len().saturating_sub(1) {
                self.selected_feature += 1;
                self.selected_diff = 0;
                self.scroll_offset = 0;
            }
        }
    }

    /// Move to previous feature
    pub fn prev_feature(&mut self) {
        if self.selected_feature > 0 {
            self.selected_feature -= 1;
            self.selected_diff = 0;
            self.scroll_offset = 0;
        }
    }

    /// Move to next diff within current feature
    pub fn next_diff(&mut self) {
        if let Some(story) = &self.story {
            if let Some(feature) = story.narrative.get(self.selected_feature) {
                if self.selected_diff < feature.diff_blocks.len().saturating_sub(1) {
                    self.selected_diff += 1;
                }
            }
        }
    }

    /// Move to previous diff within current feature
    pub fn prev_diff(&mut self) {
        if self.selected_diff > 0 {
            self.selected_diff -= 1;
        }
    }

    /// Toggle viewed status for current diff
    pub fn toggle_viewed(&mut self) {
        let key = (self.selected_feature, self.selected_diff);
        if self.viewed_diffs.contains(&key) {
            self.viewed_diffs.remove(&key);
        } else {
            self.viewed_diffs.insert(key);
        }
    }

    /// Check if a diff is viewed
    pub fn is_diff_viewed(&self, feature_idx: usize, diff_idx: usize) -> bool {
        self.viewed_diffs.contains(&(feature_idx, diff_idx))
    }

    /// Get viewed/total diff counts for a feature
    pub fn feature_progress(&self, feature_idx: usize) -> (usize, usize) {
        if let Some(story) = &self.story {
            if let Some(feature) = story.narrative.get(feature_idx) {
                let total = feature.diff_blocks.len();
                let viewed = (0..total)
                    .filter(|&diff_idx| self.viewed_diffs.contains(&(feature_idx, diff_idx)))
                    .count();
                return (viewed, total);
            }
        }
        (0, 0)
    }

    /// Get total viewed/total diffs across all features
    pub fn total_progress(&self) -> (usize, usize) {
        if let Some(story) = &self.story {
            let total: usize = story.narrative.iter().map(|f| f.diff_blocks.len()).sum();
            let viewed = self.viewed_diffs.len();
            return (viewed, total);
        }
        (0, 0)
    }

    /// Cycle to next action
    pub fn next_action(&mut self) {
        self.selected_action = match self.selected_action {
            ReviewAction::RequestChanges => ReviewAction::ClarificationQuestions,
            ReviewAction::ClarificationQuestions => ReviewAction::NextPr,
            ReviewAction::NextPr => ReviewAction::RequestChanges,
        };
    }

    /// Cycle to previous action
    pub fn prev_action(&mut self) {
        self.selected_action = match self.selected_action {
            ReviewAction::RequestChanges => ReviewAction::NextPr,
            ReviewAction::ClarificationQuestions => ReviewAction::RequestChanges,
            ReviewAction::NextPr => ReviewAction::ClarificationQuestions,
        };
    }

    /// Scroll down in feature view
    pub fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(3);
    }

    /// Scroll up in feature view
    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(3);
    }

    /// Enter edit mode for current action
    pub fn start_editing(&mut self) {
        self.cursor_pos = self.current_action_text().len();
        self.state = AppState::EditingAction(self.selected_action);
    }

    /// Exit edit mode
    pub fn stop_editing(&mut self) {
        self.state = AppState::Viewing;
    }

    /// Insert character at cursor
    pub fn insert_char(&mut self, c: char) {
        let cursor = self.cursor_pos;
        let text = self.current_action_text_mut();
        if cursor <= text.len() {
            text.insert(cursor, c);
            self.cursor_pos += 1;
        }
    }

    /// Delete character before cursor
    pub fn delete_char(&mut self) {
        if self.cursor_pos > 0 {
            let cursor = self.cursor_pos;
            let text = self.current_action_text_mut();
            text.remove(cursor - 1);
            self.cursor_pos -= 1;
        }
    }

    /// Move cursor left
    pub fn cursor_left(&mut self) {
        self.cursor_pos = self.cursor_pos.saturating_sub(1);
    }

    /// Move cursor right
    pub fn cursor_right(&mut self) {
        let len = self.current_action_text().len();
        if self.cursor_pos < len {
            self.cursor_pos += 1;
        }
    }

    /// Get the currently selected feature
    pub fn current_feature(&self) -> Option<&crate::types::Feature> {
        self.story
            .as_ref()
            .and_then(|s| s.narrative.get(self.selected_feature))
    }

    /// Move picker selection down
    pub fn picker_down(&mut self) {
        if self.picker_selected < self.pr_list.len().saturating_sub(1) {
            self.picker_selected += 1;
        }
    }

    /// Move picker selection up
    pub fn picker_up(&mut self) {
        self.picker_selected = self.picker_selected.saturating_sub(1);
    }

    /// Get currently selected PR in picker
    pub fn selected_pr(&self) -> Option<&PrListItem> {
        self.pr_list.get(self.picker_selected)
    }

    /// Open the PR picker
    pub fn open_picker(&mut self) {
        self.show_picker = true;
        self.state = AppState::PrPicker;
    }

    /// Close the PR picker
    pub fn close_picker(&mut self) {
        self.show_picker = false;
        if self.story.is_some() {
            self.state = AppState::Viewing;
        }
    }

    /// Move repo selector selection down
    pub fn repo_selector_down(&mut self) {
        if self.repo_selected < self.repo_list.len().saturating_sub(1) {
            self.repo_selected += 1;
        }
    }

    /// Move repo selector selection up
    pub fn repo_selector_up(&mut self) {
        self.repo_selected = self.repo_selected.saturating_sub(1);
    }

    /// Get currently selected repo in selector
    pub fn selected_repo(&self) -> Option<&RepoListItem> {
        self.repo_list.get(self.repo_selected)
    }

    /// Select the current repo and move to PR picker
    pub fn select_repo(&mut self) {
        if let Some(repo) = self.selected_repo() {
            self.current_repo = Some((repo.owner.clone(), repo.name.clone()));
            self.state = AppState::LoadingPrList;
            self.show_picker = true;
        }
    }

    /// Go back to repo selector from PR picker
    pub fn back_to_repo_selector(&mut self) {
        self.show_picker = false;
        self.pr_list.clear();
        self.picker_selected = 0;
        self.state = AppState::RepoSelector;
    }

    /// Reset for loading a new PR
    pub fn reset_for_new_pr(&mut self) {
        self.story = None;
        self.selected_feature = 0;
        self.selected_diff = 0;
        self.scroll_offset = 0;
        self.viewed_diffs.clear();
        self.action_texts = ActionTexts::default();
        self.show_picker = false;
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
