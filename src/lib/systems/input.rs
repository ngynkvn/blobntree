use crate::lif;
use crate::HashMap;
use egui::pos2;
use egui::vec2;

use egui::Pos2;
use egui::RawInput;

use sdl2::event::Event;

use sdl2::keyboard::Keycode;

use sdl2::mouse::MouseButton;
use sdl2::EventPump;
use specs::prelude::*;
use specs::{Join, System, WriteStorage};

use crate::lib::systems::components::{InputHandler, Velocity};

#[derive(Debug, Clone, Copy)]
enum KeyState {
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Control {
    Left,
    Right,
    Up,
    Down,
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
    pointer_pos: Pos2,
    pub running: bool,
}

type InputConfig = HashMap<Keycode, Control>;

lazy_static! {
    pub static ref DEFAULT_CONFIG: InputConfig = {
        let mut ic = HashMap::new();
        ic.insert(Keycode::Up, Control::Up);
        ic.insert(Keycode::Down, Control::Down);
        ic.insert(Keycode::Left, Control::Left);
        ic.insert(Keycode::Right, Control::Right);
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
        Self {
            keys: vec![],
            key_ref: HashMap::new(),
            key_map: HashMap::new(),
            event_pump,
            running: true,
            pointer_pos: Pos2::default(),
        }
    }
    fn apply_config(&mut self, config: InputConfig) {
        for (code, control) in config {
            self.keys.push(KeyState::Up);
            self.key_ref.insert(code, self.keys.len() - 1);
            self.key_map.insert(control, self.keys.len() - 1);
        }
    }

    fn key_map(&mut self, event: &Event) {
        match event {
            key!(KeyDown on Escape) | Event::Quit { .. } => self.running = false,
            Event::KeyDown {
                keycode: Some(code),
                ..
            } => lif! [
                Some(&i) = self.key_ref.get(code) => {
                    self.keys[i] = KeyState::Down;
                }
            ],
            Event::KeyUp {
                keycode: Some(code),
                ..
            } => lif! [
                Some(&i) = self.key_ref.get(code) => {
                    self.keys[i] = KeyState::Up;
                }
            ],
            _ => {}
        }
    }
    fn egui_raw(&mut self, event: &Event, raw: &mut RawInput) {
        use sdl2::event::Event::*;
        use sdl2::event::*;
        let event = event.clone();

        //https://github.com/ArjunNair/egui_sdl2_gl/blob/main/src/lib.rs
        match event {
            //Only the window resize event is handled
            Window {
                win_event: WindowEvent::Resized(_width, _height),
                ..
            } => {
                // raw.screen_rect = Some(Rect::from_min_size(
                //     Pos2::new(0f32, 0f32),
                //     egui::vec2(width as f32, height as f32) / raw.pixels_per_point.unwrap(),
                // ))
            }

            //MouseButonLeft pressed is the only one needed by egui
            MouseButtonDown { mouse_btn, .. } => raw.events.push(egui::Event::PointerButton {
                pos: self.pointer_pos,
                button: match mouse_btn {
                    MouseButton::Left => egui::PointerButton::Primary,
                    MouseButton::Right => egui::PointerButton::Secondary,
                    MouseButton::Middle => egui::PointerButton::Middle,
                    _ => unreachable!(),
                },
                pressed: true,
                modifiers: raw.modifiers,
            }),

            //MouseButonLeft pressed is the only one needed by egui
            MouseButtonUp { mouse_btn, .. } => raw.events.push(egui::Event::PointerButton {
                pos: self.pointer_pos,
                button: match mouse_btn {
                    MouseButton::Left => egui::PointerButton::Primary,
                    MouseButton::Right => egui::PointerButton::Secondary,
                    MouseButton::Middle => egui::PointerButton::Middle,
                    _ => unreachable!(),
                },
                pressed: false,
                modifiers: raw.modifiers,
            }),

            MouseMotion { x, y, .. } => {
                self.pointer_pos = pos2(x as f32, y as f32);
                raw.events.push(egui::Event::PointerMoved(self.pointer_pos))
            }
            TextInput { text, .. } => {
                raw.events.push(egui::Event::Text(text));
            }

            MouseWheel { x, y, .. } => {
                raw.scroll_delta = vec2(x as f32, y as f32);
            }

            _ => {
                //dbg!(event);
            }
        }
    }

    fn read_input(&mut self, raw: &mut RawInput) {
        while let Some(event) = self.event_pump.poll_event() {
            self.key_map(&event);
            self.egui_raw(&event, raw);
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
    type SystemData = (
        WriteStorage<'a, InputHandler>,
        WriteStorage<'a, Velocity>,
        Write<'a, RawInput>,
    );
    fn run(&mut self, (mut input, mut velocity, mut raw): Self::SystemData) {
        let mut x_target_speed = 0;
        let mut y_target_speed = 10;
        let mut state: Option<InputState> = Some(InputState::Idle);
        let mut raw_input = raw.clone();
        self.read_input(&mut raw_input);
        *raw = raw_input;
        match self.get_state(Control::Left) {
            Some(KeyState::Down) => {
                x_target_speed = -5;
                state.replace(InputState::Running);
            }
            Some(KeyState::Up) => {}
            _ => {}
        };
        match self.get_state(Control::Right) {
            Some(KeyState::Down) => {
                x_target_speed = 5;
                state.replace(InputState::Running);
            }
            Some(KeyState::Up) => {}
            _ => {}
        };
        match self.get_state(Control::Up) {
            Some(KeyState::Down) => {
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
    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
        self.apply_config(DEFAULT_CONFIG.clone());
    }
}
