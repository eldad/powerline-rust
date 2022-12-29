use std::env;
use std::marker::PhantomData;

use super::Module;
use crate::{Color, Powerline, Style};

pub struct KeyEnv<S: KeyEnvScheme> {
    scheme: PhantomData<S>,
}

pub trait KeyEnvScheme {
    const KEY_ENV_BG: Color;
    const KEY_ENV_FG: Color;
}

impl<S: KeyEnvScheme> KeyEnv<S> {
    pub fn new() -> KeyEnv<S> {
        KeyEnv { scheme: PhantomData }
    }
}

const KEY_ENV_NAME: &str = "PROMPT_KEYS";

impl<S: KeyEnvScheme> Module for KeyEnv<S> {
    fn append_segments(&mut self, powerline: &mut Powerline) {
        if let Some(keys) = env::var_os(KEY_ENV_NAME) {
            let keys: String = keys
                .to_string_lossy()
                .split_whitespace()
                .map(|s| format!("\u{1F511}{}", s))
                .collect::<Vec<String>>()
                .join(" ");
            powerline.add_segment(keys, Style::simple(S::KEY_ENV_FG, S::KEY_ENV_BG))
        }
    }
}
