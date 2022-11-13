use std::fmt;

use ggez::{Context, GameResult};
use specs::prelude::*;
use specs::{Dispatcher, DispatcherBuilder, World};

use crate::ecs::resources::Curtain;
use crate::ecs::systems::{CurtainRender, UpdateCurtain};
use crate::scenes::game::GameScene;
use crate::scenes::stack::{Scene, Transition};
use ggez::input::keyboard::KeyInput;

pub struct CurtainScene<'a, 'b> {
    dispatcher: Dispatcher<'a, 'b>,
}

impl<'a, 'b> CurtainScene<'a, 'b> {
    pub fn new(world: &mut World, on_player: bool) -> Self {
        let res = if on_player {
            Curtain::new_center_player()
        } else {
            Curtain::new_center()
        };
        world.insert(res);

        let mut dispatcher = DispatcherBuilder::new()
            .with(UpdateCurtain, "cus_system", &[])
            .build();

        dispatcher.setup(world);
        Self { dispatcher }
    }
}

impl<'a, 'b> Scene for CurtainScene<'a, 'b> {
    fn draw_previous(&self) -> bool {
        true
    }

    fn update(&mut self, ctx: &mut Context, world: &mut World) -> Result<Transition, String> {
        self.dispatcher.dispatch(world);
        if world.fetch::<Curtain>().radius > 500.0 {
            return Ok(Transition::Pop);
        } else if world.fetch::<Curtain>().radius < 0.0 {
            return Ok(Transition::MultiReplace(
                vec![
                    Box::new(GameScene::new(ctx, world)),
                    Box::new(CurtainScene::new(world, true)),
                ],
                2,
            ));
        }

        Ok(Transition::None)
    }

    fn draw(&mut self, ctx: &mut Context, world: &mut World) -> GameResult {
        let mut rs = CurtainRender::new(ctx);
        rs.run_now(world);
        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        _input: KeyInput,
        _repeat: bool,
        _world: &mut World,
    ) -> Result<Transition, String> {
        Ok(Transition::None)
    }

    fn name(&self) -> &str {
        "Curtain"
    }
}

impl<'a, 'b> fmt::Debug for CurtainScene<'a, 'b> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
