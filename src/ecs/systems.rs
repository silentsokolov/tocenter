use std::cmp::Ordering;
use std::convert::TryInto;
use std::time;

use ggez::audio::SoundSource;
use ggez::graphics::DrawParam;
use ggez::graphics::{self, DrawMode, MeshBuilder};
use ggez::input::keyboard::KeyCode;
use ggez::mint as mt;
use ggez::Context;
use specs::prelude::*;

use crate::consts;
use crate::ecs::components::{
    CollisionType, ConstantMovement, Enemy, Form, Player, Position, View,
};
use crate::ecs::resources::{Curtain, GameState, GameTime, KeyState, Menu, Sound};
use crate::shapes;
use crate::utils::{self, Colour, Control, Direction, GameStatus, Theme};

pub struct UpdatePosition;

impl<'a> System<'a> for UpdatePosition {
    type SystemData = (
        Read<'a, GameState>,
        Read<'a, KeyState>,
        Read<'a, GameTime>,
        WriteStorage<'a, Player>,
        ReadStorage<'a, ConstantMovement>,
        WriteStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (gs, ks, gt, mut player, conmove, mut pos) = data;

        for (pl, pos) in (&mut player, &mut pos).join() {
            let mut dir = match ks.key {
                Some(KeyCode::H) | Some(KeyCode::Left) | Some(KeyCode::Numpad4) => {
                    UpdatePosition::set_speed_boost(pl, ks.repeat, gt.delta);
                    Some(Direction::Left(pl.speed))
                }
                Some(KeyCode::L) | Some(KeyCode::Right) | Some(KeyCode::Numpad6) => {
                    UpdatePosition::set_speed_boost(pl, ks.repeat, gt.delta);
                    Some(Direction::Right(pl.speed))
                }
                Some(KeyCode::K) | Some(KeyCode::Down) | Some(KeyCode::Numpad2) => None,
                Some(KeyCode::J) | Some(KeyCode::Up) | Some(KeyCode::Numpad8) => {
                    Some(Direction::Up)
                }
                Some(_) => None,
                None => None,
            };

            if !ks.repeat {
                pl.start_angle_repeat = pos.angle;
            }
            if gs.control == Control::Advanced {
                dir = utils::get_dir(ks.key, pl.start_angle_repeat, consts::HORIZONTAL_SPEED);
            }
            self.to_move(pos, dir);
        }

        for (cm, pos) in (&conmove, &mut pos).join() {
            self.to_move(pos, Some(cm.direction));
        }
    }
}

impl UpdatePosition {
    pub fn set_speed_boost(pl: &mut Player, repeat: bool, delta: time::Duration) {
        if repeat {
            pl.speed_press_ms += delta.subsec_millis() as f32;
            let u = (pl.speed_press_ms / 100.0).ceil() / 100.0;
            pl.speed = (consts::HORIZONTAL_SPEED_MIN + u).min(consts::HORIZONTAL_SPEED_MAX);
        } else {
            pl.speed_press_ms = 0.0;
            pl.speed = consts::HORIZONTAL_SPEED_MIN;
        }
    }

    pub fn to_move(&self, pos: &mut Position, dir: Option<Direction>) {
        if pos.is_between_level() {
            self.update_radius(pos);
            return;
        }

        match dir {
            Some(Direction::Up) => {
                pos.next_level -= 1;
                self.update_radius(pos);
            }
            Some(Direction::Down) => {
                pos.next_level += 1;
                self.update_radius(pos);
            }
            Some(Direction::Left(speed)) => {
                pos.angle = utils::normalize_angle(pos.angle + speed);
            }
            Some(Direction::Right(speed)) => {
                pos.angle = utils::normalize_angle(pos.angle - speed);
            }
            None => {}
        }
    }

    pub fn update_radius(&self, pos: &mut Position) {
        let next_radius =
            match pos.current_level.partial_cmp(&pos.next_level) {
                Some(Ordering::Greater) => (pos.radius - consts::VERTICAL_SPEED)
                    .max(utils::get_level_radius(pos.next_level)),
                Some(Ordering::Less) => (pos.radius + consts::VERTICAL_SPEED)
                    .min(utils::get_level_radius(pos.next_level)),
                Some(_) => pos.radius,
                None => pos.radius,
            };

        if utils::approx_eq(pos.radius, next_radius) {
            pos.current_level = pos.next_level
        } else {
            pos.radius = next_radius
        }
    }
}

pub struct Collision;

impl<'a> System<'a> for Collision {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Player>,
        ReadStorage<'a, Enemy>,
        ReadStorage<'a, View>,
        WriteStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut player, enemy, view, mut position) = data;

        let find_levels = (&player, &mut position)
            .join()
            .map(|(_, pos)| (pos.current_level, pos.next_level))
            .collect::<Vec<(i32, i32)>>();

        let mut arcs = vec![];
        for (ent, _, v, p) in (&*entities, !&player, &view, &mut position).join() {
            let mut is_enemy = false;
            if enemy.get(ent).is_some() {
                is_enemy = true;
            };

            for (current, next) in &find_levels {
                if (*current == p.current_level || *next == p.current_level)
                    && v.form == Form::Circle
                {
                    arcs.push((
                        is_enemy,
                        p.current_level,
                        self.make_point(p.angle, p.radius),
                        self.make_point(p.angle + v.size, p.radius),
                    ))
                }
            }
        }

        for (pl, v, p) in (&mut player, &view, &mut position).join() {
            let space_radius =
                utils::get_level_radius(p.current_level) - consts::LEVEL_SPACE_RADIUS;
            let player_points =
                shapes::player(p.radius, p.angle, v.size, consts::LEVEL_FILL_RADIUS);

            for (is_enemy, current_level, start, end) in &arcs {
                if p.current_level == *current_level {
                    if self.is_body_collision(*start, &player_points) {
                        let sa = utils::normalize_angle(start.y.atan2(start.x));
                        let pa =
                            utils::normalize_angle(player_points[1].y.atan2(player_points[1].x));
                        p.angle = (pa - p.angle) + sa;
                        if *is_enemy {
                            pl.collision = Some(CollisionType::Enemy);
                            pl.take_life();
                            p.set_default_player();
                        }
                        break;
                    } else if self.is_body_collision(*end, &player_points) {
                        let sa = utils::normalize_angle(end.y.atan2(end.x));
                        let pa =
                            utils::normalize_angle(player_points[3].y.atan2(player_points[3].x));
                        p.angle = sa - (p.angle - pa);
                        if *is_enemy {
                            pl.collision = Some(CollisionType::Enemy);
                            pl.take_life();
                            p.set_default_player();
                        }
                        break;
                    }
                } else {
                    if utils::approx_eq(p.radius.max(space_radius), space_radius)
                        && self.is_radius_collision(p.angle, *start, *end)
                    {
                        if *is_enemy {
                            pl.collision = Some(CollisionType::Enemy);
                            pl.take_life();
                            p.set_default_player();
                        } else {
                            pl.collision = Some(CollisionType::Wall);
                            p.swap_level();
                        }
                        break;
                    }
                    if p.current_level < p.next_level
                        && self.is_radius_collision(p.angle, *start, *end)
                    {
                        pl.collision = Some(CollisionType::Enemy);
                        pl.take_life();
                        p.set_default_player();
                        break;
                    }
                    if p.radius < space_radius
                        && (self.is_body_collision(*start, &player_points)
                            || self.is_body_collision(*end, &player_points))
                    {
                        if *is_enemy {
                            pl.collision = Some(CollisionType::Enemy);
                            pl.take_life();
                            p.set_default_player();
                        } else {
                            pl.collision = Some(CollisionType::Wall);
                            p.swap_level();
                        }
                        break;
                    }
                }
            }
        }
    }
}

impl Collision {
    pub fn is_between_angle(&self, mid: f32, start: f32, end: f32) -> bool {
        let r = consts::PI_2;
        let e = if (end - start) < 0.0 {
            end - start + r
        } else {
            end - start
        };
        let m = if (mid - start) < 0.0 {
            mid - start + r
        } else {
            mid - start
        };
        m < e
    }

    pub fn make_point(&self, angle: f32, radius: f32) -> mt::Point2<f32> {
        mt::Point2 {
            x: angle.cos() * radius,
            y: angle.sin() * radius,
        }
    }

    pub fn is_radius_collision(
        &self,
        mid: f32,
        start: mt::Point2<f32>,
        end: mt::Point2<f32>,
    ) -> bool {
        let start = utils::normalize_angle(start.y.atan2(start.x));
        let end = utils::normalize_angle(end.y.atan2(end.x));
        let mid = utils::normalize_angle(mid);
        if self.is_between_angle(mid, start, end) {
            return true;
        }
        false
    }

    pub fn is_body_collision(&self, point: mt::Point2<f32>, triangle: &[mt::Point2<f32>]) -> bool {
        if self.collision_point_in_triangle(point, triangle[0], triangle[1], triangle[3]) {
            return true;
        }
        false
    }

    pub fn collision_point_in_triangle(
        &self,
        p: mt::Point2<f32>,
        p0: mt::Point2<f32>,
        p1: mt::Point2<f32>,
        p2: mt::Point2<f32>,
    ) -> bool {
        let a =
            1.0 / 2.0 * (-p1.y * p2.x + p0.y * (-p1.x + p2.x) + p0.x * (p1.y - p2.y) + p1.x * p2.y);
        let sign = if a < 0.0 { -1.0 } else { 1.0 };

        let s = (p0.y * p2.x - p0.x * p2.y + (p2.y - p0.y) * p.x + (p0.x - p2.x) * p.y) * sign;
        let t = (p0.x * p1.y - p0.y * p1.x + (p0.y - p1.y) * p.x + (p1.x - p0.x) * p.y) * sign;

        s > 0.0 && t > 0.0 && (s + t) < 2.0 * a * sign
    }
}

pub struct UpdateTimer;

impl<'a> System<'a> for UpdateTimer {
    type SystemData = Write<'a, GameTime>;

    fn run(&mut self, mut gt: Self::SystemData) {
        if gt.last_instant.is_none() {
            gt.last_instant = Some(time::Instant::now())
        }
        let now = time::Instant::now();
        gt.delta = now - gt.last_instant.expect("last instant is none");
        gt.last_instant = Some(now);

        if let Some(delta) = gt.timer.checked_sub(gt.delta) {
            gt.timer = delta;
            return;
        };

        gt.timer = time::Duration::default();
    }
}

pub struct UpdateGameState;

impl<'a> System<'a> for UpdateGameState {
    type SystemData = (
        Write<'a, GameState>,
        Read<'a, GameTime>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut gs, gt, player, position) = data;

        if gt.timer == time::Duration::default() {
            gs.status = Some(GameStatus::GameOver);
        }

        for (p, pos) in (&player, &position).join() {
            if p.life == 0 {
                gs.status = Some(GameStatus::GameOver);
            }
            if pos.current_level == 0 {
                gs.status = Some(GameStatus::LevelCompleted);
            }
        }
    }
}

pub struct UpdateGlobalState;

impl<'a> System<'a> for UpdateGlobalState {
    type SystemData = (Write<'a, GameState>, Read<'a, KeyState>);

    fn run(&mut self, (mut gs, ks): Self::SystemData) {
        match ks.key {
            Some(KeyCode::F2) => {
                gs.control = if gs.control == Control::Normal {
                    Control::Advanced
                } else {
                    Control::Normal
                }
            }
            Some(KeyCode::F3) => {
                gs.theme = if gs.theme == Theme::Dark {
                    Theme::Light
                } else {
                    Theme::Dark
                }
            }
            Some(_) => (),
            None => (),
        };
    }
}

pub struct UpdateCurtain;

impl<'a> System<'a> for UpdateCurtain {
    type SystemData = Write<'a, Curtain>;

    fn run(&mut self, mut curtain: Self::SystemData) {
        curtain.radius += curtain.constriction;
    }
}

pub struct UpdateMenu;

impl<'a> System<'a> for UpdateMenu {
    type SystemData = (Read<'a, KeyState>, Write<'a, Menu>);

    fn run(&mut self, (ks, mut menu): Self::SystemData) {
        match ks.key {
            Some(KeyCode::K) | Some(KeyCode::Down) | Some(KeyCode::Numpad2) => {
                menu.current_item = (menu.current_item + 1).min(menu.items.len() - 1);
            }
            Some(KeyCode::J) | Some(KeyCode::Up) | Some(KeyCode::Numpad8) => {
                if let Some(x) = menu.current_item.checked_sub(1) {
                    menu.current_item = x;
                };
            }
            Some(_) => (),
            None => (),
        };
    }
}

pub struct GameRender<'c> {
    ctx: &'c mut Context,
    canvas: &'c mut graphics::Canvas,
}

impl<'c> GameRender<'c> {
    pub fn new(ctx: &'c mut Context, canvas: &'c mut graphics::Canvas) -> GameRender<'c> {
        GameRender { ctx, canvas }
    }
}

impl<'a, 'c> System<'a> for GameRender<'c> {
    type SystemData = (
        Entities<'a>,
        Read<'a, GameState>,
        Read<'a, GameTime>,
        ReadStorage<'a, Enemy>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, View>,
        ReadStorage<'a, Player>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, gs, gt, enemy, pos, view, player) = data;

        let size = self.ctx.gfx.size();

        let mesh = &mut MeshBuilder::new();

        mesh.circle(
            DrawMode::fill(),
            [0., 0.],
            utils::get_level_radius(0) - 12.0,
            consts::DEFAULT_TOLERANCE,
            Colour::White.value(&gs.theme),
        )
        .unwrap();

        // GameRender life
        for p in (&player).join() {
            let life = p.life;
            let color = match life {
                life if life < 2 => Colour::LifeL.value(&gs.theme),
                2 => Colour::LifeM.value(&gs.theme),
                _ => Colour::Life.value(&gs.theme),
            };
            for x in (0..p.life * consts::LIFE_SIZE).step_by(consts::LIFE_SIZE.try_into().unwrap())
            {
                mesh.circle(
                    DrawMode::stroke(2.0),
                    [0., 0.],
                    utils::get_level_radius(0) - x as f32,
                    consts::DEFAULT_TOLERANCE,
                    color,
                )
                .unwrap();
            }
        }

        for (ent, pos, view) in (&*entities, &pos, &view).join() {
            match view.form {
                Form::Triangle => {
                    let points =
                        shapes::player(pos.radius, pos.angle, view.size, consts::LEVEL_FILL_RADIUS);

                    mesh.polyline(DrawMode::fill(), &points, Colour::Player.value(&gs.theme))
                        .unwrap();
                }
                Form::Circle => {
                    let mut is_enemy = false;
                    let mut alt = 0;
                    if let Some(e) = enemy.get(ent) {
                        alt = e.color;
                        is_enemy = true;
                    };
                    let color = if is_enemy {
                        if alt == 1 {
                            Colour::EnemyA.value(&gs.theme)
                        } else {
                            Colour::Enemy.value(&gs.theme)
                        }
                    } else {
                        Colour::Fg.value(&gs.theme)
                    };

                    let points = shapes::arc(
                        pos.radius,
                        pos.angle,
                        view.size,
                        consts::LEVEL_FILL_RADIUS,
                        true,
                        consts::DEFAULT_TOLERANCE,
                    );

                    mesh.polyline(DrawMode::fill(), &points, color).unwrap();
                }
            }
        }

        let ms = mesh.build();

        self.canvas.draw(
            &graphics::Mesh::from_data(self.ctx, ms),
            DrawParam::default()
                .dest([size.0 / 2., size.1 / 2.])
                .color(Colour::Fg.value(&gs.theme)),
        );

        // Render time
        self.canvas.draw(
            graphics::Text::new(format!("{:0>2}", gt.timer.as_secs()))
                .set_font("Monaco")
                .set_scale(100.),
            DrawParam::default()
                .dest([size.0 / 2. - 48., size.1 / 2. - 60.])
                .color(Colour::Bg.value(&gs.theme)),
        );

        self.canvas.draw(
            graphics::Text::new(format!("{:0>3}", gt.timer.subsec_millis()))
                .set_font("Monaco")
                .set_scale(40.),
            DrawParam::default()
                .dest([size.0 / 2. - 30., size.1 / 2. + 20.])
                .color(Colour::Bg.value(&gs.theme)),
        );

        // Render game result
        self.canvas.draw(
            graphics::Text::new(format!("Level: {}", gs.game_level))
                .set_font("Monaco")
                .set_scale(30.),
            DrawParam::default()
                .dest([size.0 - size.0 / 8., 5.])
                .color(Colour::Fg.value(&gs.theme)),
        );

        self.canvas.draw(
            graphics::Text::new(format!("Score: {}", gs.score))
                .set_font("Monaco")
                .set_scale(30.),
            DrawParam::default()
                .dest([size.0 - size.0 / 8., 30.])
                .color(Colour::Fg.value(&gs.theme)),
        );
    }
}

pub struct CurtainRender<'c> {
    ctx: &'c mut Context,
    canvas: &'c mut graphics::Canvas,
}

impl<'c> CurtainRender<'c> {
    pub fn new(ctx: &'c mut Context, canvas: &'c mut graphics::Canvas) -> CurtainRender<'c> {
        CurtainRender { ctx, canvas }
    }
}

impl<'a, 'c> System<'a> for CurtainRender<'c> {
    type SystemData = (Read<'a, GameState>, Read<'a, Curtain>);

    fn run(&mut self, (gs, curtain): Self::SystemData) {
        let mesh = &mut MeshBuilder::new();

        let points = shapes::arc(
            curtain.radius,
            0.0,
            consts::PI_2,
            1000.0,
            false,
            consts::DEFAULT_TOLERANCE,
        );

        mesh.polyline(
            graphics::DrawMode::fill(),
            &points,
            Colour::Bg.value(&gs.theme),
        )
        .unwrap();

        mesh.circle(
            DrawMode::stroke(4.0),
            [0., 0.],
            curtain.radius,
            consts::DEFAULT_TOLERANCE,
            Colour::Border.value(&gs.theme),
        )
        .unwrap();

        let ms = mesh.build();

        self.canvas.draw(
            &graphics::Mesh::from_data(self.ctx, ms),
            DrawParam::default()
                .dest([curtain.point.x + 500., curtain.point.y + 90.])
                .color(Colour::Fg.value(&gs.theme)),
        );
    }
}

pub struct MenuRender<'c> {
    ctx: &'c mut Context,
    canvas: &'c mut graphics::Canvas,
}

impl<'c> MenuRender<'c> {
    pub fn new(ctx: &'c mut Context, canvas: &'c mut graphics::Canvas) -> MenuRender<'c> {
        MenuRender { ctx, canvas }
    }
}

impl<'a, 'c> System<'a> for MenuRender<'c> {
    type SystemData = (Read<'a, GameState>, Read<'a, Menu>);

    fn run(&mut self, (gs, menu): Self::SystemData) {
        let mut y = 300.;

        self.canvas.draw(
            graphics::Text::new(menu.title.to_uppercase())
                .set_font("Monaco")
                .set_scale(60.),
            DrawParam::default()
                .dest([360., y])
                .color(Colour::Fg.value(&gs.theme)),
        );

        if !menu.subtitle.is_empty() {
            y += 80.0;
            self.canvas.draw(
                graphics::Text::new(menu.subtitle.to_lowercase())
                    .set_font("Monaco")
                    .set_scale(60.),
                DrawParam::default()
                    .dest([360., y])
                    .color(Colour::Fg.value(&gs.theme)),
            );
        }

        y += 60.0;
        for (i, item) in menu.items.iter().enumerate() {
            y += 30.0 + item.height;
            self.canvas.draw(
                graphics::Text::new(if menu.current_item == i {
                    format!("> [{}]", item.text)
                } else {
                    format!("   {}", item.text)
                })
                .set_font("Monaco")
                .set_scale(40.),
                DrawParam::default()
                    .dest([360., y])
                    .color(if item.available {
                        if menu.current_item == i {
                            Colour::Special.value(&gs.theme)
                        } else {
                            Colour::Fg.value(&gs.theme)
                        }
                    } else {
                        Colour::Gray.value(&gs.theme)
                    }),
            );
        }

        self.canvas.draw(
            graphics::Text::new(format!("[F2] control: {}", gs.control))
                .set_font("Monaco")
                .set_scale(25.),
            DrawParam::default()
                .dest([450., 950.])
                .color(Colour::Fg.value(&gs.theme)),
        );

        self.canvas.draw(
            graphics::Text::new(format!("[F3] theme: {}", gs.theme))
                .set_font("Monaco")
                .set_scale(25.),
            DrawParam::default()
                .dest([750., 950.])
                .color(Colour::Fg.value(&gs.theme)),
        );
    }
}

pub struct Music<'c> {
    ctx: &'c mut Context,
}

impl<'c> Music<'c> {
    pub fn new(ctx: &'c mut Context) -> Music<'c> {
        Music { ctx }
    }
}

impl<'a, 'c> System<'a> for Music<'c> {
    type SystemData = (Write<'a, Sound>, WriteStorage<'a, Player>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut sound, mut player) = data;

        for p in (&mut player).join() {
            match p.collision {
                Some(CollisionType::Enemy) => {
                    let s = sound.enemy.as_mut().unwrap();
                    s.play(self.ctx).unwrap();
                }
                Some(CollisionType::Wall) => {
                    let s = sound.wall.as_mut().unwrap();
                    s.play(self.ctx).unwrap();
                }
                None => {}
            }
            p.collision = None;
        }
    }
}
