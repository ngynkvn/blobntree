use crate::sprite::SpriteIndex;
use crate::Instant;
use crate::Color;
use crate::Component;
use crate::Entity;
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

impl System for Physics {
    fn update<'a>(&mut self, entities: impl Iterator<Item = &'a mut Entity>) {
        for entity in entities {
            let Position(x, y) = entity.get::<Position>();
            let Velocity(vx, vy) = entity.get::<Velocity>();
            let (x, mut y) = (x + vx, y + vy);
            if y > 800 {
                y = 0;
            }
            entity.set(Position(x, y));
        }
    }
}

pub struct Renderer<'a, 's> {
    pub sprite_manager: &'s mut SpriteManager<'a>,
    pub canvas: &'a mut WindowCanvas,
}

impl<'s, 'a> System for Renderer<'a, 's> {
    fn update<'b>(&mut self, entities: impl Iterator<Item = &'b mut Entity>) {
        let mut i = 0;
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();
        for entity in entities {
            i += 1;
            let state = entity.get::<SpriteState>();
            let position = entity.get::<Position>();
            let (texture, rect) = self.sprite_manager.next_frame(state.0, Duration::from_secs_f64(1.0 / 60.0));
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
    }
}

pub struct InputSystem<'a> {
    pub event_pump: &'a mut EventPump,
}

impl<'a> System for InputSystem<'a> {
    fn update<'b>(&mut self, entities: impl Iterator<Item = &'b mut Entity>) {
        for entity in entities {}
    }
}
