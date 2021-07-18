use crate::ecs::join;
use crate::ecs::Entities;
use crate::ecs::Join;
use crate::sprite::SpriteIndex;
use crate::Color;
use crate::Component;
use crate::Instant;
use crate::Point;
use crate::Rect;
use crate::SpriteManager;
use crate::System;
use crate::WindowCanvas;
use core::time::Duration;
use sdl2::EventPump;

pub struct Velocity(pub i32, pub i32);
pub struct Position(pub i32, pub i32);
pub struct SpriteState(pub SpriteIndex);
pub struct InputHandler;

impl Component for Velocity {}

impl Component for Position {}

impl Component for SpriteState {}

impl Component for InputHandler {}

#[derive(Default)]
pub struct Physics {}

impl<'a> System<'a> for Physics {
    type SystemData = (Position, Velocity);
    fn update(&mut self, entities: Entities<'a, Self::SystemData>) {
        for &entity in entities.entities {
            if let Some((position, velocity)) =
                join::<Self::SystemData>(entities.components, entity)
            {
                let Position(mut x, mut y) = position;
                let Velocity(vx, vy) = velocity;
                x += *vx;
                y += *vy;
                if y > 800 {
                    y = 0;
                }
            }
        }
    }
}

pub struct Renderer<'a, 's> {
    pub sprite_manager: &'s mut SpriteManager<'a>,
    pub canvas: &'a mut WindowCanvas,
    pub now: Instant,
}

impl<'a, 's> System<'_> for Renderer<'a, 's> {
    type SystemData = (Position, SpriteState);

    fn update<'b>(&mut self, entities: Entities<'b, Self::SystemData>) {
        let mut i = 0;
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();
        for &entity in entities.entities {
            if let Some((position, state)) = join::<Self::SystemData>(entities.components, entity)
            {
                i += 1;
                let (texture, rect) = self.sprite_manager.next_frame(state.0, self.now.elapsed());
                let position = Point::new(position.0, position.1);

                self.canvas
                    .copy(
                        texture,
                        rect,
                        Rect::from_center(position, rect.width() * 2, rect.height() * 2),
                    )
                    .unwrap();
            }
            self.canvas.present();
            self.now = Instant::now();
        }
    }
}

pub struct InputSystem<'a> {
    pub event_pump: &'a mut EventPump,
}

// impl<'a> System for InputSystem<'a> {
//     type SystemData = (Position, Velocity);
//     fn update<'b>(&mut self, entities: impl Iterator<Item = Self::SystemData>) {
//         // for entity in entities {}
//     }
// }
