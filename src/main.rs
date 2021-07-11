mod lib;
extern crate sdl2;

use sdl2::event::Event;
use sdl2::image::{self, InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::TextureQuery;
use sdl2::render::{Texture, TextureCreator, WindowCanvas};
use sdl2::EventPump;
use std::error::Error;
use std::time::Duration;
use std::time::Instant;

use input::{handle_input, Command};
use lib::*;
use misc::to_string;
use sprite::Sprite;

use crate::lib::input::{parse_event, Movement};

struct Position {
    point: Point,
}

#[derive(Debug)]
struct Player {
    velocity: (i32, i32),
    position: (i32, i32),
}

impl Player {
    fn new() -> Self {
        Self {
            velocity: (0, 0),
            position: (100, 100),
        }
    }

    fn update(&mut self) {}
}

pub struct Game {
    player: Player,
    ticks: usize,
    render_ticks: usize,
    start_system_time: Instant,
    running: bool,
}

fn render_game(game: &mut Game, canvas: &mut WindowCanvas, sprite: &mut Sprite) {
    let (texture, rect) = sprite.next_frame();
    let position = Point::new(game.player.position.0, game.player.position.1);
    canvas.set_draw_color(Color::RGB(128, 128, 128));
    canvas.clear();

    let TextureQuery { width, height, .. } = texture.query();

    canvas
        .copy(
            texture,
            rect,
            Rect::from_center(position, width * 2, height * 2),
        )
        .unwrap();
    game.render_ticks += 1;
}

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
    let _image_context = image::init(InitFlag::PNG | InitFlag::JPG)?;

    let window = video_subsystem
        .window("rust-sdl2 demo: Video", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(to_string)?;

    let texture_creator = canvas.texture_creator();
    let texture = texture_creator.load_texture("piskel.png")?;

    let mut sprite = Sprite::new(&texture, (32, 32));

    canvas.set_draw_color(Color::RGB(255, 0, 0));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump()?;

    // Load a font TODO
    let mut font = ttf_context.load_font("joystix monospace.ttf", 16)?;
    font.set_style(sdl2::ttf::FontStyle::BOLD);

    // Initial game time.
    let start_system_time = Instant::now();
    let mut next_tick = start_system_time;

    let mut game = Game {
        player: Player::new(),
        ticks: 0,
        render_ticks: 0,
        start_system_time,
        running: true,
    };

    // render a surface, and convert it to a texture bound to the canvas

    loop {
        let now = Instant::now();
        handle_input(&mut game, &mut event_pump);
        if !game.running {
            break;
        }

        //https://gafferongames.com/post/fix_your_timestep/
        //https://dewitters.com/dewitters-gameloop/
        update_game(&mut game, &mut next_tick);
        render_game(&mut game, &mut canvas, &mut sprite);

        let total_elapsed = (Instant::now() - start_system_time).as_secs_f32();
        let fps_game = game.ticks as f32 / total_elapsed;
        let fps_render = game.render_ticks as f32 / total_elapsed;

        let surface = font
            .render(&format!(
                "StateFPS: [{:02.1}] RenderFPS: [{:02.1}] T: {:02.2}",
                fps_game, fps_render, total_elapsed
            ))
            .blended(Color::RGBA(255, 255, 255, 255))
            .map_err(to_string)?;
        let debug = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(to_string)?;
        render_debug(&mut game, &mut canvas, &debug);

        canvas.present();

        std::thread::sleep(
            Duration::from_secs_f64(1.0 / 62.0)
                .checked_sub(Instant::now() - now)
                .unwrap_or(Duration::ZERO),
        );

        // if let Some(Command::Quit) = cmd {
        //     break;
        // }

        // The rest of the game loop goes here...
    }
    println!("Exiting");

    Ok(())
}

fn update_game(game: &mut Game, next_tick: &mut Instant) {
    // update_game(&mut game, &mut next_tick);
    const TICKS_PER_SECOND: u64 = 30;
    const MAX_FRAMESKIP: u64 = 5;
    let skip_ticks: Duration = Duration::from_millis(1000 / TICKS_PER_SECOND);
    let mut loops = 0;
    let (mut player_x, mut player_y) = game.player.position;
    let (mut pv_x, mut pv_y) = game.player.velocity;
    while Instant::now() > *next_tick && loops < MAX_FRAMESKIP {
        //player movement
        pv_x = pv_x.min(20);
        pv_x = (pv_x as f32 * 0.98) as i32;
        player_x += pv_x;
        player_y += pv_y;

        //gravity
        if player_y > 200 {
            player_y = 200;
            pv_y = 0;
        } else {
            pv_y += 10;
        }

        //tick counter
        *next_tick += skip_ticks;
        loops += 1;
        game.ticks += 1;
    }
    game.player.velocity = (pv_x, pv_y);
    game.player.position = (player_x, player_y);
}

fn render_debug(
    game: &mut Game,
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    texture: &Texture,
) -> () {
    let TextureQuery { width, height, .. } = texture.query();
    canvas
        .copy(&texture, None, Some(Rect::new(0, 0, width, height)))
        .unwrap();
}
