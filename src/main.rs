extern crate sdl2;

use sdl2::event::Event;
use sdl2::image::{self, InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Texture, TextureCreator, WindowCanvas};
use sdl2::EventPump;
use std::time::Duration;

#[derive(Debug)]
enum Command {
    Quit,
    Event(Event),
}

fn receive_event(event_pump: &mut EventPump) -> Option<Command> {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => return Some(Command::Quit),
            event => return Some(Command::Event(event)),
        }
    }
    return None;
}

struct Sprite<'a> {
    texture: &'a Texture<'a>,
    frame: usize,
    frame_len: usize,
    size: u32,
    nrow: usize,
    ncol: usize,
}

struct Position {
    point: Point,
}

impl<'a> Sprite<'a> {
    fn new(texture: &'a Texture, size: u32, nrow: usize, ncol: usize) -> Self {
        Self {
            texture,
            frame: 0,
            frame_len: nrow * ncol,
            size,
            nrow,
            ncol,
        }
    }
    fn next_frame(&mut self) -> (&Texture, Rect) {
        let i = self.frame;
        let size = self.size;
        let row = ((i / self.nrow) as u32 * size) as i32;
        let col = ((i % self.ncol) as u32 * size) as i32;
        let rect = Rect::new(col, row, size, size);
        println!("{} {} {:?}", i / self.nrow, i % self.ncol, rect);
        self.frame += 1;
        self.frame %= self.frame_len;
        (self.texture, rect)
    }
}

fn render(
    canvas: &mut WindowCanvas,
    color: Color,
    texture: &Texture,
    sprite_rect: Rect,
    position: Point,
) -> Result<(), String> {
    canvas.set_draw_color(color);
    canvas.clear();

    canvas.copy(texture, sprite_rect, Rect::from_center(position, 32, 64))?;

    canvas.present();

    Ok(())
}

fn is_keydown(command: &Option<Command>, keycode: Keycode) -> bool {
    use sdl2::event::Event::KeyDown;
    command
        .as_ref()
        .map(|cmd| match cmd {
            Command::Event(KeyDown {
                keycode: Some(key), ..
            }) => key == &keycode,
            _ => false,
        })
        .unwrap_or(false)
}

fn gravity(position: Point) -> Point {
    let mut new_y = position.y + 10;
    if new_y > 500 {
        new_y = 500;
    }
    Point::new(position.x, new_y)
}

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let _image_context = image::init(InitFlag::PNG | InitFlag::JPG)?;

    let window = video_subsystem
        .window("rust-sdl2 demo: Video", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    let texture_creator = canvas.texture_creator();
    let texture = texture_creator.load_texture("piskel.png")?;

    let mut sprite = Sprite::new(&texture, 32, 2, 1);

    let mut position = Point::new(100, 100);

    canvas.set_draw_color(Color::RGB(255, 0, 0));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump()?;
    let mut i = 0;

    loop {
        let event = receive_event(&mut event_pump);
        // println!("{:?}", event);
        if let Some(Command::Quit) = event {
            break;
        }

        if is_keydown(&event, Keycode::Left) {
            position = Point::new(position.x - 5, position.y);
        }
        if is_keydown(&event, Keycode::Right) {
            position = Point::new(position.x + 5, position.y);
        }
        if is_keydown(&event, Keycode::Down) {
            position = Point::new(position.x, position.y + 5);
        }
        if is_keydown(&event, Keycode::Up) {
            position = Point::new(position.x, position.y - 30);
        }

        position = gravity(position);

        let (texture, rect) = sprite.next_frame();
        render(
            &mut canvas,
            Color::RGB(i, 64, 128),
            &texture,
            rect,
            position,
        )?;

        i += 1;
        i %= 255;
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 15));
        // The rest of the game loop goes here...
    }

    Ok(())
}
