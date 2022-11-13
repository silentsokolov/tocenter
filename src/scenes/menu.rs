use std::fmt;

use ggez::input::keyboard::KeyCode;
use ggez::{Context, GameResult};
use specs::prelude::*;

use crate::ecs::resources::{Action, Menu};
use crate::ecs::systems::{MenuRender, UpdateGlobalState, UpdateMenu};
use crate::scenes::curtain::CurtainScene;
use crate::scenes::game::GameScene;
use crate::scenes::stack::{Scene, Transition};
use ggez::input::keyboard::KeyInput;

pub struct MenuScene<'a, 'b> {
    dispatcher: Dispatcher<'a, 'b>,
}

impl<'a, 'b> MenuScene<'a, 'b> {
    pub fn new(_ctx: &mut Context, world: &mut World) -> Self {
        let mut menu = Menu::new("To_Center".to_string());
        menu.add_item(Action::StoryMode, "story mode".to_string(), 10.0, false);
        menu.add_item(Action::EndlessMode, "endless mode".to_string(), 10.0, true);
        menu.add_item(Action::Quit, "quit".to_string(), 60.0, true);

        world.insert(menu);

        let mut dispatcher = DispatcherBuilder::new()
            .with(UpdateGlobalState, "global_system", &[])
            .with(UpdateMenu, "menu_system", &[])
            .build();
        dispatcher.setup(world);
        Self { dispatcher }
    }
}

impl<'a, 'b> Scene for MenuScene<'a, 'b> {
    fn update(&mut self, _ctx: &mut Context, world: &mut World) -> Result<Transition, String> {
        self.dispatcher.dispatch(world);

        Ok(Transition::None)
    }

    fn draw(&mut self, ctx: &mut Context, world: &mut World) -> GameResult {
        let mut render = MenuRender::new(ctx);
        render.run_now(world);
        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        input: KeyInput,
        _repeat: bool,
        world: &mut World,
    ) -> Result<Transition, String> {
        match input.keycode {
            Some(KeyCode::Return) => {
                let action = world.fetch::<Menu>().get_currect_action();
                match action {
                    Action::EndlessMode => Ok(Transition::MultiReplace(
                        vec![
                            Box::new(GameScene::new(ctx, world)),
                            Box::new(CurtainScene::new(world, true)),
                        ],
                        1,
                    )),
                    Action::Quit => {
                        ctx.request_quit();
                        Ok(Transition::None)
                    }
                    _ => Ok(Transition::None),
                }
            }
            _ => Ok(Transition::None),
        }
    }

    fn name(&self) -> &str {
        "Menu"
    }
}

impl<'a, 'b> fmt::Debug for MenuScene<'a, 'b> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
