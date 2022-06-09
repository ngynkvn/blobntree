use std::iter::Inspect;

use crate::systems::components::Shape;
use crate::Size;
use specs::prelude::*;

use crate::systems::components::SpriteHandle;
use sdl2::rect::Rect;
use specs::Entities;

use specs::ReadStorage;
use specs::{Join, System, WriteStorage};

use crate::lib::systems::components::{Position, Velocity};
use crate::lif;

#[derive(Default)]
pub struct Physics {}

impl<'a> System<'a> for Physics {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Velocity>,
        ReadStorage<'a, Shape>,
        ReadStorage<'a, SpriteHandle>,
        ReadStorage<'a, Size>,
    );
    fn run(
        &mut self,
        (entities, mut pos, mut vel_storage, collision, _sprite_state, size): Self::SystemData,
    ) {
        for (pos, vel) in (&mut pos, &mut vel_storage).join() {
            let Position(x, y) = pos;
            let Velocity(vx, vy) = vel;
            *x += *vx;
            *y += *vy;
            *vy = *vy.min(&mut 20);
            *vy += 1;
        }

        pub struct EntityS<'a> {
            rect: Rect,
            entity: Entity,
            shape: &'a Shape,
            update: Option<(Position, Velocity)>,
        }
        let mut ent_vec = vec![];
        for (entity, pos, size, shape) in (&entities, &mut pos, &size, &collision).join() {
            ent_vec.push(EntityS {
                rect: Rect::new(pos.0, pos.1, size.0 as u32, size.1 as u32),
                entity,
                shape,
                update: None,
            })
        }
        let len = ent_vec.len();
        for i in 0..len {
            let (entity_states_a, entity_states_b) = ent_vec.split_at_mut(i);
            let b = &mut entity_states_b[0];
            for a in entity_states_a {
                if let Some(intersection) = a.rect.intersection(b.rect)  {
                    if let Some(Position(_, y)) = pos.get_mut(a.entity) {
                        *y -= intersection.height() as i32;
                        a.rect.set_y(*y);
                    }
                    if let Some(Velocity(_vx, vy)) = vel_storage.get_mut(a.entity) {
                        *vy = 0;
                    }
                };
            }
        }
    }
    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
    }
}

