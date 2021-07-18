use crate::lif;
use crate::HashMap;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::EventPump;
use specs::{Join, System, WriteStorage};

use crate::lib::systems::components::{InputHandler, Velocity};

#[derive(Debug, Clone, Copy)]
enum KeyState {
    UP,
    DOWN,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Control {
    LEFT,
    RIGHT,
    UP,
    DOWN,
}

/**
 * Observer over event_pump stream.
 * The input system should live as long as
 * the event pump reference is alive.
 *
 * Ideally, we can rework this using enums
 * as indexes into an array / vec.
 */
pub struct InputSystem<'i> {
    keys: Vec<KeyState>,
    key_ref: HashMap<Keycode, usize>,
    key_map: HashMap<Control, usize>,
    event_pump: &'i mut EventPump,
    pub running: bool,
}

type InputConfig = HashMap<Keycode, Control>;

lazy_static! {
    pub static ref DEFAULT_CONFIG: InputConfig = {
        let mut ic = HashMap::new();
        ic.insert(Keycode::Up, Control::UP);
        ic.insert(Keycode::Down, Control::DOWN);
        ic.insert(Keycode::Left, Control::LEFT);
        ic.insert(Keycode::Right, Control::RIGHT);
        ic
    };
}

macro_rules! key {
    ($event: ident on $code: ident) => {
        Event::$event {
            keycode: Some(Keycode::$code),
            ..
        }
    };
}

impl<'i> InputSystem<'i> {
    pub fn new(event_pump: &'i mut EventPump) -> Self {
        let mut is = Self {
            keys: vec![],
            key_ref: HashMap::new(),
            key_map: HashMap::new(),
            event_pump,
            running: true,
        };
        is.apply_config(DEFAULT_CONFIG.clone());
        is
    }
    fn apply_config(&mut self, config: InputConfig) {
        for (code, control) in config {
            self.keys.push(KeyState::UP);
            self.key_ref.insert(code, self.keys.len() - 1);
            self.key_map.insert(control, self.keys.len() - 1);
        }
    }

    fn read_input(&mut self) {
        for event in self.event_pump.poll_iter() {
            match event {
                key!(KeyDown on Escape) | Event::Quit { .. } => self.running = false,
                Event::KeyDown {
                    keycode: Some(code),
                    ..
                } => lif! [
                    Some(&i) = self.key_ref.get(&code) => {
                        self.keys[i] = KeyState::DOWN;
                    }
                ],
                Event::KeyUp {
                    keycode: Some(code),
                    ..
                } => lif! [
                    Some(&i) = self.key_ref.get(&code) => {
                        self.keys[i] = KeyState::UP;
                    }
                ],
                _ => {}
            }
        }
        self.event_pump.pump_events();
    }

    fn get_state(&mut self, control: Control) -> Option<KeyState> {
        self.key_map.get(&control).map(|&i| self.keys[i])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputState {
    Idle,
    Running,
}

impl<'a, 'i> System<'a> for InputSystem<'i> {
    type SystemData = (WriteStorage<'a, InputHandler>, WriteStorage<'a, Velocity>);
    fn run(&mut self, (mut input, mut velocity): Self::SystemData) {
        let mut x_target_speed = 0;
        let mut y_target_speed = 10;
        let mut state: Option<InputState> = Some(InputState::Idle);
        self.read_input();
        match self.get_state(Control::LEFT) {
            Some(KeyState::DOWN) => {
                x_target_speed = -5;
                state.replace(InputState::Running);
            }
            Some(KeyState::UP) => {}
            _ => {}
        };
        match self.get_state(Control::RIGHT) {
            Some(KeyState::DOWN) => {
                x_target_speed = 5;
                state.replace(InputState::Running);
            }
            Some(KeyState::UP) => {}
            _ => {}
        };
        match self.get_state(Control::UP) {
            Some(KeyState::DOWN) => {
                y_target_speed = -10;
            }
            _ => {}
        };

        let a = 0.5;
        for (inp, vel) in (&mut input, &mut velocity).join() {
            let x: f64 = a * x_target_speed as f64;
            let y: f64 = (1.0 - a) * vel.0 as f64;
            vel.0 = (x + y) as i32;
            vel.1 = y_target_speed as i32;
            inp.0 = state;
        }
    }
}
