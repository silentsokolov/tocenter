use std::f32::consts::PI;
use std::fmt;

use ggez::graphics::{self, Color};
use ggez::{event::KeyCode, filesystem, Context};
use rand::prelude::*;
use rand::seq::SliceRandom;

use crate::consts::{FINAL_RADIUS, LEVEL_RADIUS, PI_2};

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left(f32),
    Right(f32),
}

impl Direction {
    pub fn inverse(dir: Direction) -> Direction {
        match dir {
            Direction::Left(speed) => Direction::Right(speed),
            Direction::Right(speed) => Direction::Left(speed),
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
        }
    }
}

#[derive(Debug)]
pub enum Colour {
    Bg,
    Fg,
    Gray,
    Special,
    Border,
    Life,
    LifeM,
    LifeL,
    Player,
    Enemy,
    EnemyA,
    White,
}

impl Colour {
    pub fn value(&self, t: &Theme) -> Color {
        match t {
            Theme::Dark => match self {
                Colour::White => graphics::WHITE,
                Colour::Bg => Color {
                    r: 0.12,
                    g: 0.14,
                    b: 0.19,
                    a: 1.0,
                },
                Colour::Fg => Color {
                    r: 0.8,
                    g: 0.8,
                    b: 0.78,
                    a: 1.0,
                },
                Colour::Gray => Color {
                    r: 0.36,
                    g: 0.4,
                    b: 0.45,
                    a: 1.0,
                },
                Colour::Special => Color {
                    r: 1.0,
                    g: 0.9,
                    b: 0.7,
                    a: 1.0,
                },
                Colour::Border => Color {
                    r: 0.44,
                    g: 0.48,
                    b: 0.55,
                    a: 1.0,
                },
                Colour::Life => Color {
                    r: 0.73,
                    g: 0.9,
                    b: 0.49,
                    a: 1.0,
                },
                Colour::LifeM => Color {
                    r: 1.0,
                    g: 0.65,
                    b: 0.35,
                    a: 1.0,
                },
                Colour::LifeL => Color {
                    r: 1.0,
                    g: 0.2,
                    b: 0.2,
                    a: 1.0,
                },
                Colour::Player => Color {
                    r: 1.0,
                    g: 0.8,
                    b: 0.4,
                    a: 1.0,
                },
                Colour::Enemy => Color {
                    r: 0.95,
                    g: 0.53,
                    b: 0.47,
                    a: 1.0,
                },
                Colour::EnemyA => Color {
                    r: 1.0,
                    g: 0.84,
                    b: 0.50,
                    a: 1.0,
                },
            },
            Theme::Light => match self {
                Colour::White => graphics::BLACK,
                Colour::Bg => Color {
                    r: 0.98,
                    g: 0.98,
                    b: 0.98,
                    a: 1.0,
                },
                Colour::Fg => Color {
                    r: 0.42,
                    g: 0.46,
                    b: 0.50,
                    a: 1.0,
                },
                Colour::Gray => Color {
                    r: 0.67,
                    g: 0.69,
                    b: 0.71,
                    a: 1.0,
                },
                Colour::Special => Color {
                    r: 0.90,
                    g: 0.73,
                    b: 0.49,
                    a: 1.0,
                },
                Colour::Border => Color {
                    r: 0.58,
                    g: 0.62,
                    b: 0.65,
                    a: 1.0,
                },
                Colour::Life => Color {
                    r: 0.50,
                    g: 0.70,
                    b: 0.0,
                    a: 1.0,
                },
                Colour::LifeM => Color {
                    r: 0.98,
                    g: 0.55,
                    b: 0.24,
                    a: 1.0,
                },
                Colour::LifeL => Color {
                    r: 0.96,
                    g: 0.09,
                    b: 0.09,
                    a: 1.0,
                },
                Colour::Player => Color {
                    r: 1.0,
                    g: 0.60,
                    b: 0.25,
                    a: 1.0,
                },
                Colour::Enemy => Color {
                    r: 0.94,
                    g: 0.44,
                    b: 0.44,
                    a: 1.0,
                },
                Colour::EnemyA => Color {
                    r: 0.95,
                    g: 0.68,
                    b: 0.29,
                    a: 1.0,
                },
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Theme {
    Light,
    Dark,
}

impl fmt::Display for Theme {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Theme::Dark => write!(f, "dark"),
            Theme::Light => write!(f, "light"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Control {
    Normal,
    Advanced,
}

impl fmt::Display for Control {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Control::Normal => write!(f, "normal"),
            Control::Advanced => write!(f, "advanced"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum GameStatus {
    GameOver,
    LevelCompleted,
}

pub fn normalize_angle(angle: f32) -> f32 {
    let max_arc_len = PI_2;
    if (angle).abs() >= max_arc_len {
        return (angle).abs() - max_arc_len;
    }
    if angle < 0.0 {
        return max_arc_len + angle;
    }
    angle
}

pub fn get_level_radius(level: i32) -> f32 {
    FINAL_RADIUS + LEVEL_RADIUS * level as f32
}

pub fn approx_eq(a: f32, b: f32) -> bool {
    // it not best solution
    // read https://floating-point-gui.de/errors/comparison/
    let eps = 1.0e-6;
    (a - b).abs() < eps
}

pub fn create_map_of_element(level: usize) -> Vec<(f32, f32)> {
    let mut rng = rand::thread_rng();
    let magic_number = 2;
    let amount_element = 8 + level;

    let amount_full_element = rng.gen_range(amount_element / 2, amount_element / 2 + magic_number);
    let mut map_of_element = vec![1; amount_full_element];
    let zero_vec = vec![0; amount_element - map_of_element.len()];
    map_of_element.extend_from_slice(&zero_vec);

    let mut dubl = true;
    let mut attempt = 0;
    'outer: while dubl && attempt < 5 {
        attempt += 1;
        map_of_element.shuffle(&mut rng);
        let mut el = 2;
        let mut c = 0;
        for x in &map_of_element {
            if *x != el {
                el = *x;
                c = 0;
            } else if *x == 1 {
                c += 1;
            }

            if c >= 3 {
                continue 'outer;
            }
        }
        dubl = false;
    }

    let size_element = PI_2 / amount_element as f32;
    let mut angle = 0.0;

    let mut result = vec![];
    for i in &map_of_element {
        if *i == 1 {
            result.push((angle, size_element - (size_element / amount_element as f32)));
        }
        angle += size_element;
    }
    result
}

#[allow(dead_code)]
pub fn get_dir(keycode: Option<KeyCode>, angle: f32, horizontal_speed: f32) -> Option<Direction> {
    match keycode {
        Some(KeyCode::H) | Some(KeyCode::Left) | Some(KeyCode::Numpad4) => {
            // if angle >= PI * 0.25 && angle <= PI * 0.75 {
            //     Some(Direction::Left(horizontal_speed))
            // } else if angle >= PI * 0.75 && angle <= PI * 1.25 {
            //     Some(Direction::Down)
            // } else if angle >= PI * 1.25 && angle <= PI * 1.75 {
            //     Some(Direction::Right(horizontal_speed))
            // } else {
            //     Some(Direction::Up)
            // }
            if angle >= PI * 0.0 && angle <= PI * 1.0 {
                Some(Direction::Left(horizontal_speed))
            } else {
                Some(Direction::Right(horizontal_speed))
            }
        }
        Some(KeyCode::L) | Some(KeyCode::Right) | Some(KeyCode::Numpad6) => {
            // if angle >= PI * 0.25 && angle <= PI * 0.75 {
            //     Some(Direction::Right(horizontal_speed))
            // } else if angle >= PI * 0.75 && angle <= PI * 1.25 {
            //     Some(Direction::Up)
            // } else if angle >= PI * 1.25 && angle <= PI * 1.75 {
            //     Some(Direction::Left(horizontal_speed))
            // } else {
            //     Some(Direction::Down)
            // }
            if angle >= PI * 0.0 && angle <= PI * 1.0 {
                Some(Direction::Right(horizontal_speed))
            } else {
                Some(Direction::Left(horizontal_speed))
            }
        }
        Some(KeyCode::K) | Some(KeyCode::Down) | Some(KeyCode::Numpad2) => {
            // if angle >= PI * 0.25 && angle <= PI * 0.75 {
            //     Some(Direction::Down)
            // } else if angle >= PI * 0.75 && angle <= PI * 1.25 {
            //     Some(Direction::Right(horizontal_speed))
            // } else if angle >= PI * 1.25 && angle <= PI * 1.75 {
            //     Some(Direction::Up)
            // } else {
            //     Some(Direction::Left(horizontal_speed))
            // }
            if angle >= PI * 0.0 && angle <= PI * 1.0 {
                Some(Direction::Down)
            } else {
                Some(Direction::Up)
            }
        }
        Some(KeyCode::J) | Some(KeyCode::Up) | Some(KeyCode::Numpad8) => {
            // if angle >= PI * 0.25 && angle <= PI * 0.75 {
            //     Some(Direction::Up)
            // } else if angle >= PI * 0.75 && angle <= PI * 1.25 {
            //     Some(Direction::Left(horizontal_speed))
            // } else if angle >= PI * 1.25 && angle <= PI * 1.75 {
            //     Some(Direction::Down)
            // } else {
            //     Some(Direction::Right(horizontal_speed))
            // }
            if angle >= PI * 0.0 && angle <= PI * 1.0 {
                Some(Direction::Up)
            } else {
                Some(Direction::Down)
            }
        }
        Some(_) => None,
        None => None,
    }
}

pub fn fix_path(ctx: &Context, path: &str) -> String {
    let slash_path = String::from("/") + path;

    if filesystem::is_file(ctx, &slash_path) || filesystem::is_dir(ctx, &slash_path) {
        slash_path
    } else {
        String::from(path)
    }
}
