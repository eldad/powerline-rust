use std::cmp::Ordering;
use std::env;
use std::marker::PhantomData;
use std::path::PathBuf;
use std::time::Duration;

use super::Module;
use crate::{Color, Powerline, Style};

#[cfg(not(feature = "libgit"))]
mod process;

#[cfg(not(feature = "libgit"))]
use process as internal;

#[cfg(feature = "libgit")]
mod libgit;

#[cfg(feature = "libgit")]
use libgit as internal;

pub struct Git<S> {
    scheme: PhantomData<S>,
}

pub trait GitScheme {
    const GIT_AHEAD_BG: Color;
    const GIT_AHEAD_FG: Color;
    const GIT_BEHIND_BG: Color;
    const GIT_BEHIND_FG: Color;
    const GIT_STAGED_BG: Color;
    const GIT_STAGED_FG: Color;
    const GIT_NOTSTAGED_BG: Color;
    const GIT_NOTSTAGED_FG: Color;
    const GIT_UNTRACKED_BG: Color;
    const GIT_UNTRACKED_FG: Color;
    const GIT_CONFLICTED_BG: Color;
    const GIT_CONFLICTED_FG: Color;
    const GIT_REPO_CLEAN_BG: Color;
    const GIT_REPO_CLEAN_FG: Color;
    const GIT_REPO_DIRTY_BG: Color;
    const GIT_REPO_DIRTY_FG: Color;
    const GIT_FETCH_AGE_BG: Color;
    const GIT_FETCH_AGE_FG: Color;
    const GIT_FETCH_AGE_STALE_FG: Color;
}

impl<S: GitScheme> Git<S> {
    pub fn new() -> Git<S> {
        Git { scheme: PhantomData }
    }
}

#[derive(Default)]
pub struct GitStats {
    pub untracked: u32,
    pub conflicted: u32,
    pub non_staged: u32,
    pub ahead: u32,
    pub behind: u32,
    pub staged: u32,
    pub branch_name: String,
    pub fetch_head_age: Option<Duration>,
}

impl GitStats {
    pub fn is_dirty(&self) -> bool {
        (self.untracked + self.conflicted + self.staged + self.non_staged) > 0
    }
}

fn find_git_dir() -> Option<PathBuf> {
    let mut git_dir = match env::current_dir() {
        Err(_) => return None,
        Ok(v) => v,
    };

    loop {
        git_dir.push(".git/");

        if git_dir.exists() {
            git_dir.pop();
            return Some(git_dir);
        }
        git_dir.pop();

        if !git_dir.pop() {
            return None;
        }
    }
}

fn duration_string(duration: &Duration) -> String {
    let mut minutes = duration.as_secs() / 60;
    let mut hours = minutes / 60;
    let mut days = hours / 24;
    let weeks = days / 7;
    days %= 7;
    hours %= 24;
    minutes %= 60;

    let mut timestr = String::new();

    if weeks > 0 {
        timestr += &format!("{}w", weeks);
    }
    if weeks > 0 || days > 0 {
        timestr += &format!("{}d", days);
    }
    if weeks > 0 || days > 0 || hours > 0 {
        timestr += &format!("{}h", hours);
    }
    if weeks > 0 || days > 0 || hours > 0 || minutes > 0{
        timestr += &format!("{}m", minutes);
    }

    timestr
}

impl<S: GitScheme> Module for Git<S> {
    fn append_segments(&mut self, powerline: &mut Powerline) {
        let git_dir = match find_git_dir() {
            Some(dir) => dir,
            _ => return,
        };

        let stats = match internal::run_git(&git_dir) {
            Ok(stats) => stats,
            _ => return,
        };

        let (branch_fg, branch_bg) = if stats.is_dirty() {
            (S::GIT_REPO_DIRTY_FG, S::GIT_REPO_DIRTY_BG)
        } else {
            (S::GIT_REPO_CLEAN_FG, S::GIT_REPO_CLEAN_BG)
        };

        powerline.add_segment(stats.branch_name, Style::simple(branch_fg, branch_bg));

        let mut add_elem = |count: u32, symbol, fg, bg| match count.cmp(&1) {
            Ordering::Equal => powerline.add_segment(symbol, Style::simple(fg, bg)),
            Ordering::Greater => powerline.add_segment(format!("{count}{symbol}"), Style::simple(fg, bg)),
            Ordering::Less => (),
        };

        add_elem(stats.ahead, '\u{2B06}', S::GIT_AHEAD_FG, S::GIT_AHEAD_BG);
        add_elem(stats.behind, '\u{2B07}', S::GIT_BEHIND_FG, S::GIT_BEHIND_BG);
        add_elem(stats.staged, '\u{2714}', S::GIT_STAGED_FG, S::GIT_STAGED_BG);
        add_elem(stats.non_staged, '\u{270E}', S::GIT_NOTSTAGED_FG, S::GIT_NOTSTAGED_BG);
        add_elem(stats.untracked, '\u{2753}', S::GIT_UNTRACKED_FG, S::GIT_UNTRACKED_BG);
        add_elem(stats.conflicted, '\u{273C}', S::GIT_CONFLICTED_FG, S::GIT_CONFLICTED_BG);

        if let Some(fetch_head_age) = stats.fetch_head_age {

            if fetch_head_age.as_secs() > 3600 {
                let symbol = '\u{23F0}';

                let fg = if fetch_head_age.as_secs() > 12 * 3600 {
                    S::GIT_FETCH_AGE_FG
                } else {
                    S::GIT_FETCH_AGE_STALE_FG
                };

                powerline.add_segment(format!("{}{}", duration_string(&fetch_head_age), symbol), Style::simple(fg, S::GIT_FETCH_AGE_BG));
            }
        }
    }
}
