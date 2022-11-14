#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

extern crate specs;
#[macro_use]
extern crate specs_derive;

mod consts;
mod ecs;
mod scenes;
mod shapes;
mod utils;
use std::path::Path;
use winit::dpi::LogicalSize;

use std::env;

use ecs::resources::{GameState, KeyState};
use ggez::audio;
use ggez::graphics;
use ggez::input::keyboard::KeyInput;
use ggez::{conf, event, Context, GameResult};
use log::info;
use scenes::{menu::MenuScene, stack::SceneStack};
use specs::prelude::*;

use crate::ecs::resources::Sound;
use crate::utils::Colour;

struct MainState {
    world: World,
    scenes: SceneStack,
}

impl MainState {
    pub fn new(ctx: &mut Context) -> GameResult<MainState> {
        let wall_sound = audio::Source::new(ctx, "/sounds/knock.flac").unwrap();
        let enemy_sound = audio::Source::new(ctx, "/sounds/enemy.flac").unwrap();
        let sound = Sound {
            wall: Some(wall_sound),
            enemy: Some(enemy_sound),
        };

        ctx.gfx.add_font(
            "Monaco",
            graphics::FontData::from_path(ctx, "/fonts/monaco.ttf")?,
        );

        let mut world = specs::World::new();
        world.register::<ecs::components::Position>();
        world.register::<ecs::components::View>();
        world.register::<ecs::components::Player>();
        world.register::<ecs::components::Enemy>();
        world.register::<ecs::components::ConstantMovement>();

        world.insert(GameState::default());
        world.insert(KeyState::default());
        world.insert(sound);

        let scenes = SceneStack::new(Box::new(MenuScene::new(ctx, &mut world)));

        Ok(MainState { world, scenes })
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while ctx.time.check_update_time(consts::DESIRED_FPS) {
            self.scenes.update(ctx, &mut self.world)?;
            // Reset input
            let mut k = self.world.fetch_mut::<KeyState>();
            k.key = None;
            k.mods = None;
        }
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(
            ctx,
            Colour::Bg.value(&self.world.fetch::<GameState>().theme),
        );
        self.scenes.draw(ctx, &mut self.world, &mut canvas)?;
        canvas.finish(ctx)
    }

    fn key_down_event(&mut self, ctx: &mut Context, input: KeyInput, repeat: bool) -> GameResult {
        {
            let mut k = self.world.fetch_mut::<KeyState>();
            k.key = input.keycode;
            k.mods = Some(input.mods);
            k.repeat = repeat;
        }

        self.scenes
            .key_down_event(ctx, input, repeat, &mut self.world);
        Ok(())
    }
}

fn main() -> GameResult {
    // Create logger
    #[cfg(debug_assertions)]
    {
        use log::LevelFilter;

        env_logger::Builder::new()
            .filter(None, LevelFilter::Info)
            .filter(Some("gfx_device_gl"), LevelFilter::Error)
            .init();
    }

    // Find resource
    let mut current_dir = env::current_dir()?;
    current_dir.push("resources");

    // Make ContextBuilder
    let mut cb = ggez::ContextBuilder::new("tocenter", "silentsokolov")
        .with_conf_file(false)
        .window_setup(conf::WindowSetup::default().title("ToCenter"))
        .window_mode(conf::WindowMode {
            resizable: false,
            borderless: true,
            logical_size: Some(LogicalSize::new(500.0, 500.0)),
            ..Default::default()
        });

    #[cfg(debug_assertions)]
    {
        info!("Use resource in {:?}", current_dir);
        cb = cb.add_resource_path(current_dir);
    }

    #[cfg(not(debug_assertions))]
    {
        info!("Use resource archive");
        cb = cb.add_zipfile_bytes(include_bytes!("../resources.zip").to_vec());
    }

    let (mut ctx, event_loop) = cb.build()?;
    ctx.gfx.set_window_icon(&ctx, Path::new("/128x128.png"))?;

    let state = MainState::new(&mut ctx)?;
    // Run
    event::run(ctx, event_loop, state)
}
