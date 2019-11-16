use std::fmt::Debug;

use ggez::input::keyboard::{KeyCode, KeyMods};
use ggez::{Context, GameResult};
use log::warn;
use specs::World;

pub trait Scene: Debug {
    fn update(&mut self, ctx: &mut Context, world: &mut World) -> Result<Transition, String>;
    fn draw(&mut self, ctx: &mut Context, world: &mut World) -> GameResult;
    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        keymods: KeyMods,
        repeat: bool,
        world: &mut World,
    ) -> Result<Transition, String>;
    fn name(&self) -> &str;
    fn draw_previous(&self) -> bool {
        false
    }
}

#[derive(Debug)]
pub enum Transition {
    None,
    Push(Box<dyn Scene>),
    Pop,
    Replace(Box<dyn Scene>),
    MultiReplace(Vec<Box<dyn Scene>>, u32),
}

pub struct SceneStack {
    scenes: Vec<Box<dyn Scene>>,
}

impl SceneStack {
    pub fn new(scene: Box<dyn Scene>) -> Self {
        Self {
            scenes: vec![scene],
        }
    }

    #[allow(dead_code)]
    pub fn push(&mut self, scene: Box<dyn Scene>) {
        self.scenes.push(scene)
    }

    #[allow(dead_code)]
    pub fn pop(&mut self) {
        self.scenes.pop();
    }

    pub fn update(&mut self, ctx: &mut Context, world: &mut World) -> GameResult {
        let scene = self.mut_scene();
        let trans = scene.update(ctx, world).unwrap();
        self.switch(trans);
        Ok(())
    }

    pub fn draw(&mut self, ctx: &mut Context, world: &mut World) -> GameResult {
        SceneStack::draw_scenes(&mut self.scenes, ctx, world)
    }

    fn draw_scenes(
        scenes: &mut [Box<dyn Scene>],
        ctx: &mut Context,
        world: &mut World,
    ) -> GameResult {
        assert!(!scenes.is_empty());
        if let Some((current, rest)) = scenes.split_last_mut() {
            if current.draw_previous() {
                SceneStack::draw_scenes(rest, ctx, world)?
            }
            current.draw(ctx, world)?
        }
        Ok(())
    }

    pub fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        keymods: KeyMods,
        repeat: bool,
        world: &mut World,
    ) {
        let scene = self.mut_scene();
        let trans = scene
            .key_down_event(ctx, keycode, keymods, repeat, world)
            .unwrap();
        self.switch(trans);
    }

    fn mut_scene(&mut self) -> &mut dyn Scene {
        &mut **self.scenes.last_mut().expect("Not scene in stack")
    }

    fn switch(&mut self, trans: Transition) {
        match trans {
            Transition::Push(scene) => {
                self.scenes.push(scene);
            }
            Transition::Pop => {
                if self.scenes.pop().is_none() {
                    warn!("Stack doesn't have scene for pop");
                }
            }
            Transition::Replace(scene) => {
                if self.scenes.pop().is_none() {
                    warn!("Stack doesn't have scene for replace");
                }
                self.scenes.push(scene)
            }
            Transition::MultiReplace(scenes, mut num) => {
                while num > 0 {
                    if self.scenes.pop().is_none() {
                        warn!("Stack doesn't have scene for replace");
                    }
                    num -= 1;
                }

                self.scenes.extend(scenes)
            }
            Transition::None => {}
        };
    }
}
