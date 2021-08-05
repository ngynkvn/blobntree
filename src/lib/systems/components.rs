use crate::systems::input::InputState;
use specs::{Component, DenseVecStorage};

use crate::lib::sprite::SpriteIndex;

#[derive(Component)]
pub struct Velocity(pub i32, pub i32);

#[derive(Component)]
pub struct Position(pub i32, pub i32);

#[derive(Component)]
pub struct Size(pub i32, pub i32);

#[derive(Component)]
pub struct SpriteHandle {
    pub index: SpriteIndex,
    pub width: usize,
    pub height: usize,
    pub scale: usize,
}

#[derive(Component)]
pub struct StaticSprite(pub &'static str, pub usize);

#[derive(Component)]
pub struct InputHandler(pub Option<InputState>);

pub enum CollisionType {
    Static,
}

#[derive(Component)]
pub struct Collision(pub Option<CollisionType>);
