// use crate::Entity;
// use crate::Component;
// use crate::System;

pub mod components;
pub mod input;
pub mod physics;
pub mod renderer;
// pub mod egui;

#[macro_export]
macro_rules! lif {
    ($ty: pat = $target:expr => $body: block) => {
        if let $ty = $target $body
    };
}
