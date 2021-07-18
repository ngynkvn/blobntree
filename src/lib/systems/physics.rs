use crate::systems::components::Collision;
use crate::Size;
use specs::Entity;

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
        ReadStorage<'a, Collision>,
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
            collision: &'a Collision,
        }
        let mut ent_vec = vec![];
        for (entity, pos, size, collision) in (&entities, &mut pos, &size, &collision).join() {
            ent_vec.push(EntityS {
                rect: Rect::new(pos.0, pos.1, size.0 as u32, size.1 as u32),
                entity,
                collision,
            })
        }
        let len = ent_vec.len();
        for i in 0..len {
            let (a, b) = ent_vec.split_at_mut(i);
            let entity_b = &mut b[0];
            for entity_a in a {
                lif! [Some(_) = entity_a.collision.0 => { continue }];
                lif![
                    Some(intersection) = entity_a.rect.intersection(entity_b.rect) => {
                        if let Some(Position(_, y)) = pos.get_mut(entity_a.entity) {
                            *y -= intersection.height() as i32;
                            entity_a.rect.set_y(*y);
                        }
                        if let Some(Velocity(_vx, vy)) = vel_storage.get_mut(entity_a.entity) {
                            *vy = 0;
                        }
                    }
                ];
            }
        }
    }
}
