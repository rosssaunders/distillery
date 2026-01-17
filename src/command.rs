use crate::action::Action;
use crate::config::AppConfig;
use crate::domain::types::{PrContext, ReviewAction, Story};
use crate::domain::{github, llm};

pub enum Command {
    FetchRepoList,
    FetchPrList { owner: String, repo: String },
    FetchPr { owner: String, repo: String, number: u32 },
    GenerateStory { pr: PrContext },
    LoadCache { path: String },
    SaveCache { path: String, story: Story },
    PostReview { owner: String, repo: String, number: u32, body: String },
    PostComment { owner: String, repo: String, number: u32, body: String },
    CreateNextPrIssue {
        owner: String,
        repo: String,
        number: u32,
        title: String,
        body: String,
    },
}

pub async fn execute_command(command: Command, config: &AppConfig) -> Option<Action> {
    match command {
        Command::FetchRepoList => {
            let result = github::fetch_repo_list().map_err(|e| e.to_string());
            Some(Action::RepoListLoaded(result))
        }
        Command::FetchPrList { owner, repo } => {
            let result = github::fetch_pr_list(&owner, &repo).map_err(|e| e.to_string());
            Some(Action::PrListLoaded(result))
        }
        Command::FetchPr { owner, repo, number } => {
            let result = github::fetch_pr(&owner, &repo, number)
                .await
                .map_err(|e| e.to_string());
            Some(Action::PrLoaded(result))
        }
        Command::GenerateStory { pr } => {
            let result = llm::generate_story(&pr, &config.api_key, &config.model)
                .await
                .map_err(|e| e.to_string());
            Some(Action::StoryGenerated(result))
        }
        Command::LoadCache { path } => {
            let story = std::fs::read_to_string(path)
                .ok()
                .and_then(|contents| serde_json::from_str(&contents).ok());
            Some(Action::CacheLoaded(story))
        }
        Command::SaveCache { path, story } => {
            if let Ok(json) = serde_json::to_string_pretty(&story) {
                let _ = std::fs::write(path, json);
            }
            None
        }
        Command::PostReview {
            owner,
            repo,
            number,
            body,
        } => {
            let result = github::post_review(&owner, &repo, number, &body)
                .map(|_| ())
                .map_err(|e| e.to_string());
            Some(Action::SubmissionResult {
                action: ReviewAction::RequestChanges,
                result,
            })
        }
        Command::PostComment {
            owner,
            repo,
            number,
            body,
        } => {
            let result = github::post_comment(&owner, &repo, number, &body)
                .map(|_| ())
                .map_err(|e| e.to_string());
            Some(Action::SubmissionResult {
                action: ReviewAction::ClarificationQuestions,
                result,
            })
        }
        Command::CreateNextPrIssue {
            owner,
            repo,
            number,
            title,
            body,
        } => {
            let result = github::create_next_pr_issue(&owner, &repo, number, &title, &body)
                .map(|_| ())
                .map_err(|e| e.to_string());
            Some(Action::SubmissionResult {
                action: ReviewAction::NextPr,
                result,
            })
        }
    }
}
