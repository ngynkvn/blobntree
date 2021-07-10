use crate::Point;
use crate::Keycode;
use crate::Event;

#[derive(Debug)]
pub enum Command {
    Quit,
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

pub fn handle_event(event: Event, position: &mut Point) -> Option<Command> {
    match event {
        Event::Quit { .. }
        | Event::KeyDown {
            keycode: Some(Keycode::Escape),
            ..
        } => return Some(Command::Quit),
        event => {
            if is_keydown(&event, Keycode::Left) {
                *position = position.offset(-5, 0);
            }
            if is_keydown(&event, Keycode::Right) {
                *position = position.offset(5, 0);
            }
            if is_keydown(&event, Keycode::Up) {
                *position = position.offset(0, -30);
            }
            if is_keydown(&event, Keycode::Down) {
                *position = position.offset(0, 5);
            }
            None
        }
    }
}
