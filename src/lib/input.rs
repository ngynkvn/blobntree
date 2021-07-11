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

fn is_keydown(event: &Event, keycode: Keycode) -> bool {
    use sdl2::event::Event::KeyDown;
    if let KeyDown {
        keycode: Some(key), ..
    } = event
    {
        &keycode == key
    } else {
        false
    }
}

pub fn handle_input(game: &mut Game, event_pump: &mut EventPump) {
    for command in event_pump.poll_iter().filter_map(parse_event) {
        println!("{:?}", command);
        match command {
            Command::PlayerInput(Movement::UP) => {
                game.player.velocity.1 -= 40;
            }
            Command::PlayerInput(Movement::DOWN) => {}
            Command::PlayerInput(Movement::LEFT) => {
                if game.player.velocity.0 > 0 {
                    game.player.velocity.0 = -5;
                }
                game.player.velocity.0 -= 5;
            }
            Command::PlayerInput(Movement::RIGHT) => {
                if game.player.velocity.0 < 0 {
                    game.player.velocity.0 = 5;
                }
                game.player.velocity.0 += 5;
            }
            Command::Quit => {
                game.running = false;
            }
            _ => {}
        }
    }
}

pub fn parse_event(event: Event) -> Option<Command> {
    match event {
        Event::Quit { .. }
        | Event::KeyDown {
            keycode: Some(Keycode::Escape),
            ..
        } => Some(Command::Quit),
        Event::KeyDown {
            keycode: Some(Keycode::Left),
            ..
        } => Some(Command::PlayerInput(Movement::LEFT)),
        Event::KeyDown {
            keycode: Some(Keycode::Right),
            ..
        } => Some(Command::PlayerInput(Movement::RIGHT)),
        Event::KeyDown {
            keycode: Some(Keycode::Up),
            ..
        } => Some(Command::PlayerInput(Movement::UP)),
        Event::KeyDown {
            keycode: Some(Keycode::Down),
            ..
        } => Some(Command::PlayerInput(Movement::DOWN)),
        _event => None,
    }
}
