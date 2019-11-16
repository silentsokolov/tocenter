use std::fmt;

use ggez::event;
use ggez::input::keyboard::{KeyCode, KeyMods};
use ggez::{Context, GameResult};
use specs::prelude::*;

use crate::ecs::resources::{Action, Menu};
use crate::ecs::systems::{MenuRender, UpdateGlobalState, UpdateMenu};
use crate::scenes::stack::{Scene, Transition};

pub struct PauseScene<'a, 'b> {
    dispatcher: Dispatcher<'a, 'b>,
}

impl<'a, 'b> PauseScene<'a, 'b> {
    pub fn new(_ctx: &mut Context, world: &mut World) -> Self {
        let mut menu = Menu::new("pause".to_string());
        menu.add_item(Action::Continue, "continue".to_string(), 10.0, true);
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

impl<'a, 'b> Scene for PauseScene<'a, 'b> {
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
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
        world: &mut World,
    ) -> Result<Transition, String> {
        if keycode == KeyCode::Return {
            let action = world.fetch::<Menu>().get_currect_action();
            match action {
                Action::Continue => Ok(Transition::Pop),
                Action::Quit => {
                    event::quit(ctx);
                    Ok(Transition::None)
                }
                _ => Ok(Transition::None),
            }
        } else if keycode == KeyCode::Escape {
            Ok(Transition::Pop)
        } else {
            Ok(Transition::None)
        }
    }

    fn name(&self) -> &str {
        "Pause"
    }
}

impl<'a, 'b> fmt::Debug for PauseScene<'a, 'b> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
