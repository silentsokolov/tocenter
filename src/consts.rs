use std::f32::consts::PI;

pub const DESIRED_FPS: u32 = 60;
pub const DEFAULT_TOLERANCE: f32 = 0.1;
pub const LEVEL_FILL_RADIUS: f32 = 25.0;
pub const LEVEL_SPACE_RADIUS: f32 = 20.0;
pub const LEVEL_RADIUS: f32 = LEVEL_FILL_RADIUS + LEVEL_SPACE_RADIUS;
pub const FINAL_RADIUS: f32 = 80.0;
pub const HORIZONTAL_SPEED: f32 = 0.16;
pub const HORIZONTAL_SPEED_MIN: f32 = 0.04;
pub const HORIZONTAL_SPEED_MAX: f32 = 0.20;
pub const VERTICAL_SPEED: f32 = 3.;
pub const GAME_TIME: u64 = 15;
pub const LIFE_SIZE: u32 = 4;
pub const PLAYER_LIFE: u32 = 3;
pub const PLAYER_START_LEVEL: i32 = 8;
pub const PLAYER_START_ANGLE: f32 = 0.5 * PI;
pub const PI_2: f32 = 2.0 * PI;
