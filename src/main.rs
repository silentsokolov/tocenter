#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

extern crate specs;
#[macro_use]
extern crate specs_derive;

mod consts;
mod ecs;
mod scenes;
mod shapes;
mod utils;

use std::env;

use ecs::resources::{GameState, KeyState};
use ggez::audio;
use ggez::event::{self, EventHandler};
use ggez::graphics::{self};
use ggez::input::keyboard::{KeyCode, KeyMods};
use ggez::{conf, timer, Context, ContextBuilder, GameResult};
use log::{error, info};
use scenes::{menu::MenuScene, stack::SceneStack};
use specs::prelude::*;

use crate::ecs::resources::{Font, Sound};
use crate::utils::{fix_path, Colour};

struct MainState {
    world: World,
    scenes: SceneStack,
}

impl MainState {
    pub fn new(ctx: &mut Context) -> MainState {
        let wall_sound = audio::Source::new(ctx, fix_path(&ctx, "sounds/knock.flac")).unwrap();
        let enemy_sound = audio::Source::new(ctx, fix_path(&ctx, "sounds/enemy.flac")).unwrap();
        let sound = Sound {
            wall: Some(wall_sound),
            enemy: Some(enemy_sound),
        };

        let base_font = graphics::Font::new(ctx, fix_path(&ctx, "fonts/monaco.ttf")).unwrap();
        let font = Font {
            base: Some(base_font),
        };

        let mut world = specs::World::new();
        world.register::<ecs::components::Position>();
        world.register::<ecs::components::View>();
        world.register::<ecs::components::Player>();
        world.register::<ecs::components::Enemy>();
        world.register::<ecs::components::ConstantMovement>();

        world.insert(GameState::default());
        world.insert(KeyState::default());
        world.insert(sound);
        world.insert(font);

        let scenes = SceneStack::new(Box::new(MenuScene::new(ctx, &mut world)));

        MainState { world, scenes }
    }
}

impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        while timer::check_update_time(ctx, consts::DESIRED_FPS) {
            self.scenes.update(ctx, &mut self.world)?;
            // Reset input
            let mut k = self.world.fetch_mut::<KeyState>();
            k.key = None;
            k.mods = None;
        }
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(
            ctx,
            Colour::Bg.value(&self.world.fetch::<GameState>().theme),
        );

        #[cfg(debug_assertions)]
        {
            use ggez::nalgebra as na;

            let fps = timer::fps(ctx);
            let text = graphics::Text::new(format!("FPS: {:.0}", fps));

            graphics::draw(ctx, &text, (na::Point2::new(0.0, 0.0),))?;
        }

        self.scenes.draw(ctx, &mut self.world)?;

        graphics::present(ctx)?;
        timer::yield_now();
        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        keymods: KeyMods,
        repeat: bool,
    ) {
        {
            let mut k = self.world.fetch_mut::<KeyState>();
            k.key = Some(keycode);
            k.mods = Some(keymods);
            k.repeat = repeat;
        }

        if keycode == KeyCode::Q && keymods == KeyMods::LOGO {
            event::quit(ctx);
        }

        self.scenes
            .key_down_event(ctx, keycode, keymods, repeat, &mut self.world);
    }
}

fn main() {
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
    let mut current_dir = env::current_dir().unwrap();
    current_dir.push("resources");

    // Make ContextBuilder
    let mut cb = ContextBuilder::new("tocenter", "silentsokolov")
        .with_conf_file(false)
        .window_setup(
            conf::WindowSetup::default()
                .title("ToCenter")
                .samples(conf::NumSamples::Four),
        )
        .window_mode(
            conf::WindowMode::default()
                .dimensions(500.0, 500.0)
                .min_dimensions(500.0, 500.0)
                .resizable(false),
        );

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

    let (ctx, event_loop) = &mut cb.build().unwrap();

    graphics::set_window_icon(ctx, Some(fix_path(ctx, "128x128.ico"))).unwrap();

    // Create game state
    let mut state = MainState::new(ctx);

    // Run
    match event::run(ctx, event_loop, &mut state) {
        Ok(_) => info!("Exited cleanly."),
        Err(e) => error!("Exit with error: {}", e),
    }
}
