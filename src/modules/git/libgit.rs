use std::path::Path;
use std::time::Duration;

use git2::{Branch, BranchType, ObjectType, Repository, Status, StatusOptions, StatusShow};

use super::GitStats;

pub fn try_get_fetch_head_age(repository: &Repository) -> anyhow::Result<Duration> {
    let fetch_head_path = repository.path().join("FETCH_HEAD");
    let head_path = repository.path().join("HEAD");
    let fetch_head_age =
        std::fs::metadata(fetch_head_path).or_else(|_| std::fs::metadata(head_path))?.modified()?.elapsed()?;
    Ok(fetch_head_age)
}
pub fn run_git(path: &Path) -> anyhow::Result<GitStats> {
    let repository = Repository::open(path)?;

    let mut status_options = StatusOptions::new();
    status_options
        .show(StatusShow::IndexAndWorkdir)
        .include_untracked(true)
        .renames_from_rewrites(true)
        .renames_head_to_index(true);

    let (mut untracked, mut non_staged, mut conflicted, mut staged, mut ahead, mut behind) = (0, 0, 0, 0, 0, 0);

    for status in repository.statuses(Some(&mut status_options))?.iter().map(|ref x| x.status()) {
        if status.intersects(
            Status::INDEX_NEW
                | Status::INDEX_MODIFIED
                | Status::INDEX_TYPECHANGE
                | Status::INDEX_RENAMED
                | Status::INDEX_DELETED,
        ) {
            staged += 1;
        }
        if status.is_wt_new() {
            untracked += 1;
        }
        if status.is_conflicted() {
            conflicted += 1;
        }
        if status.intersects(Status::WT_MODIFIED | Status::WT_TYPECHANGE | Status::WT_DELETED) {
            non_staged += 1;
        }
    }

    let active_branch: Option<Branch> =
        repository.branches(Some(BranchType::Local))?.filter_map(Result::ok).map(|x| x.0).find(|b| b.is_head());

    if let Some(ref active_branch) = active_branch {
        let local = active_branch.get().target();
        let upstream = active_branch.upstream().ok().and_then(|obj| obj.get().target());

        if let (Some(local), Some(upstream)) = (local, upstream) {
            let (a, b) = repository.graph_ahead_behind(local, upstream)?;
            ahead = a as u32;
            behind = b as u32;
        };
    }

    let branch_name =
        active_branch.as_ref().and_then(|x| x.name().unwrap()).map(ToOwned::to_owned).unwrap_or_else(|| {
            if let Ok(head) = repository.head() {
                let target = head.target().unwrap();

                repository
                    .find_object(target, Some(ObjectType::Any))
                    .unwrap()
                    .short_id()
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_owned()
            } else {
                String::from("*")
            }
        });

    let fetch_head_age = try_get_fetch_head_age(&repository).ok();

    Ok(GitStats { untracked, staged, non_staged, ahead, behind, conflicted, branch_name, fetch_head_age })
}
