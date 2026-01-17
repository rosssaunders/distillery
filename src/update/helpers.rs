use crate::app::App;
use crate::domain::types::PrContext;

pub fn current_repo(app: &App) -> Option<(String, String)> {
    if let Some((owner, repo)) = &app.current_repo {
        return Some((owner.clone(), repo.clone()));
    }

    app.pr
        .as_ref()
        .map(|pr| (pr.owner.clone(), pr.repo.clone()))
}

pub fn current_pr_ref(app: &App) -> Option<(String, String, u32)> {
    if let Some(pr) = &app.pr {
        return Some((pr.owner.clone(), pr.repo.clone(), pr.number));
    }

    match (&app.current_repo, app.current_pr_number) {
        (Some((owner, repo)), Some(number)) => Some((owner.clone(), repo.clone(), number)),
        _ => None,
    }
}

pub fn ensure_cached_pr_context(app: &mut App) {
    if app.pr.is_some() {
        return;
    }

    let Some((owner, repo, number)) = current_pr_ref(app) else {
        return;
    };

    app.pr = Some(PrContext {
        owner,
        repo,
        number,
        title: "Cached PR".to_string(),
        body: String::new(),
        diff: String::new(),
        author: String::new(),
        base_branch: String::new(),
        head_branch: String::new(),
    });
}
