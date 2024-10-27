use std::env;
use std::marker::PhantomData;

use super::Module;
use crate::{Color, Powerline, Style};

pub struct Distrobox<S: DistroboxScheme> {
    scheme: PhantomData<S>,
}

pub trait DistroboxScheme {
    const DISTROBOX_FG: Color;
    const DISTROBOX_BG: Color;
}

impl<S: DistroboxScheme> Distrobox<S> {
    pub fn new() -> Distrobox<S> {
        Distrobox { scheme: PhantomData }
    }
}

impl<S: DistroboxScheme> Module for Distrobox<S> {
    fn append_segments(&mut self, powerline: &mut Powerline) {
        let container_id = env::var("CONTAINER_ID");

        if let Ok(container_id) = container_id.as_ref() {
            powerline.add_short_segment("📦️", Style::simple(S::DISTROBOX_FG, S::DISTROBOX_BG));
            powerline.add_segment(container_id, Style::simple(S::DISTROBOX_FG, S::DISTROBOX_BG))
        }
    }
}
