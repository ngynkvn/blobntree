mod lib;
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

use lib::*;
use input::{Command, handle_event};
use sprite::Sprite;


struct Position {
    point: Point,
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

    let mut sprite = Sprite::new(&texture, (32, 32));

    let mut position = Point::new(100, 100);

    canvas.set_draw_color(Color::RGB(255, 0, 0));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump()?;

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
