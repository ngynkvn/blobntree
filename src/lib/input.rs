use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use sdl2::EventPump;

use crate::Game;

#[derive(Debug)]
pub enum Movement {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

#[derive(Debug)]
pub enum Command {
    Quit,
    PlayerInput(Movement),
    Event(Event),
}

macro_rules! key {
    ($event: ident on $code: ident) => {
        Event::$event {
            keycode: Some(Keycode::$code),
            ..
        }
    };
}

pub fn handle_input(game: &mut Game, event_pump: &mut EventPump) {
    for command in event_pump.poll_iter() {
        match command {
            key!(KeyDown on Up) => {
                game.player.velocity.1 -= 40;
                // enter jumping state...
            }
            key!(KeyDown on Left) => {
                if game.player.velocity.0 > 0 {
                    game.player.velocity.0 = -5;
                }
                game.player.velocity.0 -= 5;
                // enter run left.
            }
            key!(KeyUp on Left) => {
                game.player.velocity.0 = -5;
            }
            key!(KeyDown on Right) => {
                if game.player.velocity.0 < 0 {
                    game.player.velocity.0 = 5;
                }
                game.player.velocity.0 += 5;
                // enter run left.
            }
            key!(KeyUp on Right) => {
                game.player.velocity.0 = 5;
            }
            key!(KeyDown on Escape) | Event::Quit { .. } => {
                game.running = false;
            }
            // Command::PlayerInput(Movement::LEFT) => {
            // }
            // Command::PlayerInput(Movement::RIGHT) => {
            //     if game.player.velocity.0 < 0 {
            //         game.player.velocity.0 = 5;
            //     }
            //     game.player.velocity.0 += 5;
            // }
            // Command::Quit => {
            //     game.running = false;
            // }
            _ => {}
        }
    }
    event_pump.pump_events();
}
