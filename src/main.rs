extern crate sdl2;

use sdl2::event::Event;
use sdl2::image::{self, InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Texture, TextureCreator, WindowCanvas};
use sdl2::EventPump;
use std::time::Duration;
use std::time::Instant;

#[derive(Debug)]
enum Command {
    Quit,
    Event(Event),
}

fn receive_event(event_pump: &mut EventPump) -> Option<Command> {
    return None;
}

struct Sprite<'a> {
    texture: &'a Texture<'a>,
    frame: u32,
    frame_len: u32,
    size: (u32, u32),
    nrow: u32,
    ncol: u32,
}

struct Position {
    point: Point,
}

impl<'a> Sprite<'a> {
    fn new(texture: &'a Texture, size: (u32, u32)) -> Self {
        let (sprite_h, sprite_w) = size;
        let h = texture.query().height;
        let w = texture.query().width;
        let nrow = h / sprite_h;
        let ncol = w / sprite_w;
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
        let (size_x, size_y) = self.size;
        let row = if self.nrow == 1 {
            0
        } else {
            ((i / self.nrow) as u32 * size_x) as i32
        };
        let col = ((i % self.ncol) as u32 * size_y) as i32;
        let rect = Rect::new(col, row, size_x, size_y);
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

    canvas.copy(texture, sprite_rect, Rect::from_center(position, 128, 128))?;

    canvas.present();

    Ok(())
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

fn gravity(position: Point) -> Point {
    let mut new_y = position.y + 10;
    if new_y > 500 {
        new_y = 500;
    }
    Point::new(position.x, new_y)
}

fn handle_event(event: Event, position: &mut Point) -> Option<Command> {
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

    let mut sprite = Sprite::new(&texture, (32, 32));

    let mut position = Point::new(100, 100);

    canvas.set_draw_color(Color::RGB(255, 0, 0));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump()?;
    let mut i = 0;

    //https://gafferongames.com/post/fix_your_timestep/
    //https://dewitters.com/dewitters-gameloop/
    let mut ticks = 0;
    let mut render_ticks = 0;
    // Initial game time.
    let start_system_time = Instant::now();
    let mut next_tick = start_system_time;

    const TICKS_PER_SECOND: u64 = 25;
    const MAX_FRAMESKIP: u64 = 5;
    let skip_ticks: Duration = Duration::from_millis(1000 / TICKS_PER_SECOND);
    loop {
        let now = Instant::now();
        let mut loops = 0;
        while Instant::now() > next_tick && loops < MAX_FRAMESKIP {
            position = gravity(position);
            next_tick += skip_ticks;
            loops += 1;
            ticks += 1;
        }

        let mut cmd = None;

        event_pump
            .poll_iter()
            .map(|event| {
                let e = handle_event(event, &mut position);
                cmd = e.or(None);
            })
            .last();
        // println!("{:?}", event);
        if let Some(Command::Quit) = cmd {
            break;
        }

        let (texture, rect) = sprite.next_frame();
        render(
            &mut canvas,
            Color::RGB(255, 64, 128),
            &texture,
            rect,
            position,
        )?;
        render_ticks += 1;

        i += 1;
        i %= 255;
        // ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 15));
        let elapsed = Instant::now() - start_system_time;
        let t = render_ticks as f32 / elapsed.as_secs_f32();
        println!("[{}] [{}] {}", ticks as f32 / elapsed.as_secs_f32(), t, elapsed.as_secs_f32());
        // Rendering should be capped at 60fps.
        if Instant::now() - now < Duration::from_millis(1000 / 60) {
            std::thread::sleep(Duration::from_millis(1000 / 60) - (Instant::now() - now));
        }
        // The rest of the game loop goes here...
    }

    Ok(())
}
