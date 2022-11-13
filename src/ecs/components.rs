use std::f32;
use std::mem;

use specs::{prelude::*, Component};

use crate::consts::{PLAYER_START_ANGLE, PLAYER_START_LEVEL};
use crate::utils::{self, Direction};

#[derive(Debug, PartialEq, Eq)]
pub enum Form {
    Triangle,
    Circle,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CollisionType {
    Wall,
    Enemy,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Player {
    pub life: u32,
    pub speed: f32,
    pub speed_press_ms: f32,
    pub start_angle_repeat: f32,
    pub collision: Option<CollisionType>,
}

impl Player {
    pub fn take_life(&mut self) -> Option<u32> {
        let check = self.life.checked_sub(1);
        if let Some(i) = check {
            self.life = i
        }
        check
    }
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Enemy {
    pub color: u32,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct ConstantMovement {
    pub direction: Direction,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct View {
    pub form: Form,
    pub size: f32,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Position {
    pub radius: f32,
    pub angle: f32,
    pub current_level: i32,
    pub next_level: i32,
}

impl Position {
    pub fn new(level: i32, angle: f32) -> Self {
        Position {
            current_level: level,
            next_level: level,
            radius: utils::get_level_radius(level),
            angle,
        }
    }

    pub fn is_between_level(&self) -> bool {
        self.current_level != self.next_level
    }

    pub fn swap_level(&mut self) {
        mem::swap(&mut self.next_level, &mut self.current_level);
    }

    pub fn set_level(&mut self, level: i32) {
        self.current_level = level;
        self.next_level = level;
        self.radius = utils::get_level_radius(level);
    }

    pub fn set_default_player(&mut self) {
        self.angle = PLAYER_START_ANGLE;
        self.set_level(PLAYER_START_LEVEL);
    }
}
