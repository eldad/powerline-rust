use std::marker::PhantomData;
use std::{env, path};

use super::Module;
use crate::{Color, Powerline, Style};

pub struct FlatCwd<S: FlatCwdScheme> {
    resolve_symlinks: bool,
    scheme: PhantomData<S>,
}

pub trait FlatCwdScheme {
    const CWD_FG: Color;
    const PATH_FG: Color;
    const PATH_BG: Color;
    const HOME_FG: Color;
    const HOME_BG: Color;
    const SEPARATOR_FG: Color;
    const CWD_HOME_SYMBOL: &'static str = "~";
}

impl<S: FlatCwdScheme> FlatCwd<S> {
    pub fn new(resolve_symlinks: bool) -> FlatCwd<S> {
        FlatCwd { resolve_symlinks, scheme: PhantomData }
    }
}

impl<S: FlatCwdScheme> Module for FlatCwd<S> {
    fn append_segments(&mut self, powerline: &mut Powerline) {
        let current_dir = if self.resolve_symlinks {
            env::current_dir().unwrap()
        } else {
            path::PathBuf::from(env::var("PWD").unwrap())
        };

        let cwd = current_dir.to_str().unwrap();

        if cwd == "/" {
            return powerline.add_segment('/', Style::simple(S::PATH_FG, S::PATH_BG));
        }

        let mut at_home = false;
        if let Ok(home_str) = env::var("HOME") {
            if cwd.starts_with(&home_str) {
                at_home = true;

                let home_cwd = format!("~{}", &cwd[home_str.len()..]);
                powerline.add_segment(home_cwd, Style::special(S::HOME_FG, S::HOME_BG, '\u{E0B1}', S::SEPARATOR_FG));
            }
        }

        if !at_home {
            powerline.add_segment(cwd, Style::special(S::PATH_FG, S::PATH_BG, '\u{E0B1}', S::SEPARATOR_FG));
        };

        if let Some(style) = powerline.last_style_mut() {
            style.sep = '\u{E0B0}';
            style.sep_fg = style.bg.transpose();
        }
    }
}
