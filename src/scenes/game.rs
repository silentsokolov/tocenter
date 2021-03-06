use std::fmt;

use ggez::input::keyboard::{KeyCode, KeyMods};
use ggez::{Context, GameResult};
use rand::Rng;
use specs::prelude::*;

use crate::consts::{HORIZONTAL_SPEED_MIN, PLAYER_LIFE, PLAYER_START_ANGLE, PLAYER_START_LEVEL};
use crate::ecs::components::{ConstantMovement, Enemy, Form, Player, Position, View};
use crate::ecs::resources::{GameState, GameTime};
use crate::ecs::systems::{Collision, GameRender, UpdateGameState, UpdatePosition, UpdateTimer};
use crate::scenes::curtain::CurtainScene;
use crate::scenes::gameover::GameOverScene;
use crate::scenes::pause::PauseScene;
use crate::scenes::stack::{Scene, Transition};
use crate::utils::{self, Direction, GameStatus};

pub struct GameScene<'a, 'b> {
    dispatcher: Dispatcher<'a, 'b>,
}

impl<'a, 'b> GameScene<'a, 'b> {
    pub fn new(_ctx: &mut Context, world: &mut World) -> Self {
        world.insert(GameTime::default());

        world
            .create_entity()
            .with(Position::new(PLAYER_START_LEVEL, PLAYER_START_ANGLE))
            .with(View {
                form: Form::Triangle,
                size: 8.0,
            })
            .with(Player {
                life: PLAYER_LIFE,
                speed: HORIZONTAL_SPEED_MIN,
                speed_press_ms: 0.0,
                start_angle_repeat: PLAYER_START_ANGLE,
            })
            .build();

        let mut dir = Direction::Left(0.01);
        for level in 1..=7 {
            let element_map = utils::create_map_of_element(level);
            for (angle, size) in element_map {
                let color = rand::thread_rng().gen_range(0, 2);
                let mut builder = world
                    .create_entity()
                    .with(Position::new(level as i32, angle))
                    .with(View {
                        form: Form::Circle,
                        size,
                    });

                if level % 2 != 0 {
                    builder = builder.with(Enemy { color });
                    builder = builder.with(ConstantMovement { direction: dir });
                }
                builder.build();
            }
            if level % 2 != 0 {
                dir = Direction::inverse(dir);
            }
        }

        let mut dispatcher = DispatcherBuilder::new()
            .with(UpdateTimer, "time_system", &[])
            .with(UpdatePosition, "pos_system", &["time_system"])
            .with(Collision, "collision_system", &["pos_system"])
            .with(UpdateGameState, "game_system", &["collision_system"])
            .build();
        dispatcher.setup(world);
        Self { dispatcher }
    }
}

impl<'a, 'b> Scene for GameScene<'a, 'b> {
    fn update(&mut self, ctx: &mut Context, world: &mut World) -> Result<Transition, String> {
        self.dispatcher.dispatch(world);

        let status = world.fetch::<GameState>().status.clone();
        let score = world.fetch::<GameTime>().timer.as_secs();

        match status {
            Some(GameStatus::GameOver) => {
                world.delete_all();
                world.maintain();
                world.fetch_mut::<GameState>().status = None;
                Ok(Transition::Replace(Box::new(GameOverScene::new(
                    ctx, world,
                ))))
            }
            Some(GameStatus::LevelCompleted) => {
                world.delete_all();
                world.maintain();
                world.fetch_mut::<GameState>().status = None;
                world.fetch_mut::<GameState>().game_level += 1;
                world.fetch_mut::<GameState>().score += score;
                Ok(Transition::Push(Box::new(CurtainScene::new(world, false))))
            }
            None => Ok(Transition::None),
        }
    }

    fn draw(&mut self, ctx: &mut Context, world: &mut World) -> GameResult {
        let mut rs = GameRender::new(ctx);
        rs.run_now(world);
        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
        world: &mut World,
    ) -> Result<Transition, String> {
        if keycode == KeyCode::Escape {
            Ok(Transition::Push(Box::new(PauseScene::new(ctx, world))))
        } else {
            Ok(Transition::None)
        }
    }

    fn name(&self) -> &str {
        "Game"
    }
}

impl<'a, 'b> fmt::Debug for GameScene<'a, 'b> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
