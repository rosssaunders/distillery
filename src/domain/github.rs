use anyhow::{Context, Result};
use serde::Deserialize;
use std::process::Command;

use super::types::{CiStatus, PrContext, PrListItem, RepoListItem};

/// Response from `gh pr view --json`
#[derive(Debug, Deserialize)]
struct GhPrView {
    number: u32,
    title: String,
    body: Option<String>,
    author: GhAuthor,
    #[serde(rename = "baseRefName")]
    base_ref_name: String,
    #[serde(rename = "headRefName")]
    head_ref_name: String,
}

#[derive(Debug, Deserialize)]
struct GhAuthor {
    login: String,
}

/// Response from `gh pr list --json`
#[derive(Debug, Deserialize)]
struct GhPrListItem {
    number: u32,
    title: String,
    author: GhAuthor,
    #[serde(rename = "headRefName")]
    head_ref_name: String,
    #[serde(rename = "isDraft")]
    is_draft: bool,
    additions: u32,
    deletions: u32,
    #[serde(rename = "reviewRequests")]
    review_requests: Vec<GhReviewRequest>,
    #[serde(rename = "statusCheckRollup")]
    status_check_rollup: Option<Vec<GhStatusCheck>>,
}

#[derive(Debug, Deserialize)]
struct GhReviewRequest {
    login: Option<String>,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GhStatusCheck {
    state: Option<String>,
    status: Option<String>,
    conclusion: Option<String>,
}

impl GhPrListItem {
    fn into_list_item(self, current_user: &str) -> PrListItem {
        let review_requested = self.review_requests.iter().any(|r| {
            r.login.as_deref() == Some(current_user) || r.name.as_deref() == Some(current_user)
        });

        let ci_status = self.compute_ci_status();

        PrListItem {
            number: self.number,
            title: self.title,
            author: self.author.login,
            head_branch: self.head_ref_name,
            is_draft: self.is_draft,
            review_requested,
            ci_status,
            additions: self.additions,
            deletions: self.deletions,
        }
    }

    fn compute_ci_status(&self) -> CiStatus {
        let Some(checks) = &self.status_check_rollup else {
            return CiStatus::Unknown;
        };

        if checks.is_empty() {
            return CiStatus::Unknown;
        }

        let mut has_pending = false;
        let mut has_failure = false;

        for check in checks {
            // Check conclusion first (for completed checks)
            if let Some(conclusion) = &check.conclusion {
                match conclusion.as_str() {
                    "SUCCESS" | "NEUTRAL" | "SKIPPED" => {}
                    "FAILURE" | "TIMED_OUT" | "CANCELLED" | "ACTION_REQUIRED" => {
                        has_failure = true;
                    }
                    _ => {}
                }
            }

            // Check state/status for in-progress
            if let Some(state) = &check.state {
                match state.as_str() {
                    "PENDING" | "QUEUED" | "IN_PROGRESS" | "WAITING" => {
                        has_pending = true;
                    }
                    "FAILURE" | "ERROR" => {
                        has_failure = true;
                    }
                    _ => {}
                }
            }

            if let Some(status) = &check.status
                && (status == "IN_PROGRESS" || status == "QUEUED" || status == "PENDING")
            {
                has_pending = true;
            }
        }

        if has_failure {
            CiStatus::Failure
        } else if has_pending {
            CiStatus::Pending
        } else {
            CiStatus::Success
        }
    }
}

/// Fetch PR metadata and diff using gh CLI
pub async fn fetch_pr(owner: &str, repo: &str, number: u32) -> Result<PrContext> {
    let repo_spec = format!("{}/{}", owner, repo);

    // Fetch PR metadata
    let output = Command::new("gh")
        .args([
            "pr",
            "view",
            &number.to_string(),
            "--repo",
            &repo_spec,
            "--json",
            "number,title,body,author,baseRefName,headRefName",
        ])
        .output()
        .context("Failed to execute gh pr view")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gh pr view failed: {}", stderr);
    }

    let pr_view: GhPrView =
        serde_json::from_slice(&output.stdout).context("Failed to parse gh pr view output")?;

    // Fetch diff
    let diff_output = Command::new("gh")
        .args(["pr", "diff", &number.to_string(), "--repo", &repo_spec])
        .output()
        .context("Failed to execute gh pr diff")?;

    if !diff_output.status.success() {
        let stderr = String::from_utf8_lossy(&diff_output.stderr);
        anyhow::bail!("gh pr diff failed: {}", stderr);
    }

    let diff = String::from_utf8_lossy(&diff_output.stdout).to_string();

    Ok(PrContext {
        owner: owner.to_string(),
        repo: repo.to_string(),
        number: pr_view.number,
        title: pr_view.title,
        body: pr_view.body.unwrap_or_default(),
        diff,
        author: pr_view.author.login,
        base_branch: pr_view.base_ref_name,
        head_branch: pr_view.head_ref_name,
    })
}

/// Post a review requesting changes
pub fn post_review(owner: &str, repo: &str, number: u32, body: &str) -> Result<()> {
    let repo_spec = format!("{}/{}", owner, repo);

    let output = Command::new("gh")
        .args([
            "pr",
            "review",
            &number.to_string(),
            "--repo",
            &repo_spec,
            "--request-changes",
            "--body",
            body,
        ])
        .output()
        .context("Failed to execute gh pr review")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gh pr review failed: {}", stderr);
    }

    Ok(())
}

/// Post a comment on the PR
pub fn post_comment(owner: &str, repo: &str, number: u32, body: &str) -> Result<()> {
    let repo_spec = format!("{}/{}", owner, repo);

    let output = Command::new("gh")
        .args([
            "pr",
            "comment",
            &number.to_string(),
            "--repo",
            &repo_spec,
            "--body",
            body,
        ])
        .output()
        .context("Failed to execute gh pr comment")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gh pr comment failed: {}", stderr);
    }

    Ok(())
}

/// Create an issue and return the issue number
pub fn create_issue(owner: &str, repo: &str, title: &str, body: &str) -> Result<u32> {
    let repo_spec = format!("{}/{}", owner, repo);

    let output = Command::new("gh")
        .args([
            "issue",
            "create",
            "--repo",
            &repo_spec,
            "--title",
            title,
            "--body",
            body,
        ])
        .output()
        .context("Failed to execute gh issue create")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gh issue create failed: {}", stderr);
    }

    // Parse the issue URL to get the number
    let stdout = String::from_utf8_lossy(&output.stdout);
    let issue_url = stdout.trim();

    // URL format: https://github.com/owner/repo/issues/123
    let issue_number = issue_url
        .rsplit('/')
        .next()
        .and_then(|s| s.parse().ok())
        .context("Failed to parse issue number from URL")?;

    Ok(issue_number)
}

/// Create issue and post comment linking to it
pub fn create_next_pr_issue(
    owner: &str,
    repo: &str,
    pr_number: u32,
    issue_title: &str,
    issue_body: &str,
) -> Result<u32> {
    // Create the issue
    let issue_number = create_issue(owner, repo, issue_title, issue_body)?;

    // Post a comment on the PR linking to the issue
    let comment = format!(
        "Follow-up work tracked in #{}\n\n_Created via [Distillery](https://github.com/rosssaunders/distillery)_",
        issue_number
    );
    post_comment(owner, repo, pr_number, &comment)?;

    Ok(issue_number)
}

/// Fetch the current GitHub user
pub fn get_current_user() -> Result<String> {
    let output = Command::new("gh")
        .args(["api", "user", "--jq", ".login"])
        .output()
        .context("Failed to execute gh api user")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gh api user failed: {}", stderr);
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Fetch all open PRs for a repo, sorted by priority:
/// 1. Review requested from current user (non-draft)
/// 2. Other open PRs (non-draft)
/// 3. Draft PRs
pub fn fetch_pr_list(owner: &str, repo: &str) -> Result<Vec<PrListItem>> {
    let repo_spec = format!("{}/{}", owner, repo);
    let current_user = get_current_user().unwrap_or_default();

    let output = Command::new("gh")
        .args([
            "pr",
            "list",
            "--repo",
            &repo_spec,
            "--limit",
            "50",
            "--json",
            "number,title,author,headRefName,isDraft,additions,deletions,reviewRequests,statusCheckRollup",
        ])
        .output()
        .context("Failed to execute gh pr list")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gh pr list failed: {}", stderr);
    }

    let pr_list: Vec<GhPrListItem> =
        serde_json::from_slice(&output.stdout).context("Failed to parse gh pr list output")?;

    let mut items: Vec<PrListItem> = pr_list
        .into_iter()
        .map(|p| p.into_list_item(&current_user))
        .collect();

    // Sort: review_requested + non-draft first, then non-draft, then drafts
    items.sort_by(|a, b| {
        // Priority order: review_requested non-draft > non-draft > draft
        let priority_a = if a.is_draft {
            2
        } else if a.review_requested {
            0
        } else {
            1
        };
        let priority_b = if b.is_draft {
            2
        } else if b.review_requested {
            0
        } else {
            1
        };

        priority_a.cmp(&priority_b).then_with(|| a.number.cmp(&b.number))
    });

    Ok(items)
}

/// Response from `gh repo list --json`
#[derive(Debug, Deserialize)]
struct GhRepoListItem {
    #[serde(rename = "nameWithOwner")]
    name_with_owner: String,
    description: Option<String>,
    #[serde(rename = "isFork")]
    is_fork: bool,
    #[serde(rename = "isPrivate")]
    is_private: bool,
}

/// Fetch repositories the user has access to, sorted by most recently pushed
pub fn fetch_repo_list() -> Result<Vec<RepoListItem>> {
    // Fetch repos the user owns (gh repo list returns them sorted by push date by default)
    let output = Command::new("gh")
        .args([
            "repo",
            "list",
            "--limit",
            "50",
            "--json",
            "nameWithOwner,description,isFork,isPrivate",
        ])
        .output()
        .context("Failed to execute gh repo list")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gh repo list failed: {}", stderr);
    }

    let repo_list: Vec<GhRepoListItem> =
        serde_json::from_slice(&output.stdout).context("Failed to parse gh repo list output")?;

    let items: Vec<RepoListItem> = repo_list
        .into_iter()
        .map(|r| {
            let (owner, name) = r.name_with_owner.split_once('/').unwrap_or(("", &r.name_with_owner));
            RepoListItem {
                owner: owner.to_string(),
                name: name.to_string(),
                description: r.description.unwrap_or_default(),
                is_fork: r.is_fork,
                is_private: r.is_private,
            }
        })
        .collect();

    Ok(items)
}

/// Parse a PR URL or owner/repo#number format
pub fn parse_pr_reference(input: &str) -> Result<(String, String, u32)> {
    // Try URL format: https://github.com/owner/repo/pull/123
    if input.contains("github.com") {
        let parts: Vec<&str> = input.trim_end_matches('/').split('/').collect();
        if parts.len() >= 2 {
            let number_str = parts.last().context("Missing PR number")?;
            let number: u32 = number_str.parse().context("Invalid PR number")?;

            // Find owner and repo
            if let Some(pos) = parts.iter().position(|&p| p == "github.com")
                && parts.len() > pos + 2
            {
                let owner = parts[pos + 1].to_string();
                let repo = parts[pos + 2].to_string();
                return Ok((owner, repo, number));
            }
        }
        anyhow::bail!("Invalid GitHub PR URL format");
    }

    // Try owner/repo#number format
    if let Some((repo_part, number_str)) = input.split_once('#')
        && let Some((owner, repo)) = repo_part.split_once('/')
    {
        let number: u32 = number_str.parse().context("Invalid PR number")?;
        return Ok((owner.to_string(), repo.to_string(), number));
    }

    // Try owner/repo number format (two args)
    anyhow::bail!(
        "Invalid PR reference. Use: owner/repo#123 or https://github.com/owner/repo/pull/123"
    );
}
