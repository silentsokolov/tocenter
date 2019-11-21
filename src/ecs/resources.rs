use std::time;

use ggez::audio;
use ggez::event::{KeyCode, KeyMods};
use ggez::graphics;
use ggez::nalgebra as na;

use crate::consts::{GAME_TIME, PLAYER_START_ANGLE, PLAYER_START_LEVEL};
use crate::utils;

#[derive(Debug)]
pub struct KeyState {
    pub key: Option<KeyCode>,
    pub mods: Option<KeyMods>,
    pub repeat: bool,
}

impl Default for KeyState {
    fn default() -> Self {
        Self {
            key: Option::None,
            mods: Option::None,
            repeat: false,
        }
    }
}

pub struct GameState {
    pub game_level: u32,
    pub score: u64,
    pub status: Option<utils::GameStatus>,
    pub theme: utils::Theme,
    pub control: utils::Control,
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            game_level: 1,
            score: 0,
            status: None,
            theme: utils::Theme::Dark,
            control: utils::Control::Normal,
        }
    }
}

impl GameState {
    pub fn reset_result(&mut self) {
        self.game_level = 1;
        self.score = 0;
    }
}

#[derive(Debug)]
pub struct GameTime {
    pub last_instant: Option<time::Instant>,
    pub timer: time::Duration,
}

impl Default for GameTime {
    fn default() -> Self {
        Self {
            last_instant: None,
            timer: time::Duration::new(GAME_TIME, 0),
        }
    }
}

#[derive(Debug)]
pub struct Curtain {
    pub radius: f32,
    pub point: na::Point2<f32>,
    pub constriction: f32,
}

impl Default for Curtain {
    fn default() -> Self {
        Self {
            radius: 0.0,
            point: na::Point2::new(0.0, 0.0),
            constriction: 7.0,
        }
    }
}

impl Curtain {
    pub fn new_center_player() -> Self {
        let player_radius = utils::get_level_radius(PLAYER_START_LEVEL);

        Self {
            radius: 15.0,
            point: na::Point2::new(
                PLAYER_START_ANGLE.cos() * player_radius,
                PLAYER_START_ANGLE.sin() * player_radius,
            ),
            constriction: 7.0,
        }
    }

    pub fn new_center() -> Self {
        Self {
            radius: 500.0,
            point: na::Point2::new(0.0, 0.0),
            constriction: -7.0,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Action {
    EndlessMode,
    StoryMode,
    Continue,
    Quit,
}

#[derive(Debug)]
pub struct MenuItem {
    pub action: Action,
    pub text: String,
    pub height: f32,
    pub available: bool,
}

#[derive(Default, Debug)]
pub struct Menu {
    pub title: String,
    pub subtitle: String,
    pub items: Vec<MenuItem>,
    pub current_item: usize,
}

impl Menu {
    pub fn new(title: String) -> Self {
        let mut m = Self::default();
        m.title = title;
        m
    }

    pub fn add_item(&mut self, action: Action, text: String, height: f32, available: bool) {
        self.items.push(MenuItem {
            action,
            text,
            height,
            available,
        });
    }

    pub fn get_currect_action(&self) -> Action {
        self.items[self.current_item].action.clone()
    }
}

#[derive(Default, Debug)]
pub struct Sound {
    pub wall: Option<audio::Source>,
    pub enemy: Option<audio::Source>,
}

#[derive(Default, Debug)]
pub struct Font {
    pub base: Option<graphics::Font>,
}
