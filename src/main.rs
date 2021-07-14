mod lib;
extern crate sdl2;

use crate::font::FontConfig;
use crate::state::PlayerState;
use sdl2::image::{self, InitFlag, LoadTexture};

use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Texture, WindowCanvas};
use sdl2::render::{TextureCreator, TextureQuery};
use sdl2::video::WindowContext;

use std::collections::HashMap;
use std::time::Duration;
use std::time::Instant;

use font::FontManager;
use input::handle_input;
use lib::*;
use misc::to_string;
use sprite::Sprite;

use crate::lib::sprite::{SpriteConfig, SpriteManager};

struct Position {
    point: Point,
}

pub struct Game {
    player: PlayerState,
    ticks: usize,
    render_ticks: usize,
    start_system_time: Instant,
    running: bool,
}

impl Game {
    fn update(&mut self) {
        self.player.update();
    }
}

fn render_game(
    game: &mut Game,
    canvas: &mut WindowCanvas,
    sprite: &mut SpriteManager,
    elapsed: Duration,
) {
    let sprite = sprite.get("chicken").unwrap();
    let (texture, rect) = sprite.next_frame(elapsed);
    let position = Point::new(game.player.position.0, game.player.position.1);
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    canvas
        .copy(
            texture,
            rect,
            Rect::from_center(position, rect.width() * 3, rect.height() * 3),
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
        .map_err(to_string)?;

    let mut canvas = window.into_canvas().build().map_err(to_string)?;

    let texture_creator = canvas.texture_creator();

    let mut sprite_manager = SpriteManager::new(&texture_creator);
    sprite_manager.add(SpriteConfig {
        name: "chicken",
        path: "sprites/chicken_smear.png",
        json: "sprites/chicken_smear.json",
    });

    canvas.set_draw_color(Color::RGB(255, 0, 0));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump()?;

    // Load a font TODO
    let mut font_manager = FontManager::new(&texture_creator, &ttf_context);
    font_manager.add(FontConfig {
        path: "joystix monospace.ttf",
        size: 16,
        style: sdl2::ttf::FontStyle::BOLD,
    });

    // Initial game time.
    let start_system_time = Instant::now();
    let mut next_tick = start_system_time;

    let mut game = Game {
        player: PlayerState::new(),
        ticks: 0,
        render_ticks: 0,
        start_system_time,
        running: true,
    };

    // render a surface, and convert it to a texture bound to the canvas

    let mut now = Instant::now();
    let frame_time = Duration::from_secs_f64(1.0 / 60.0);
    loop {
        handle_input(&mut game, &mut event_pump);
        if !game.running {
            break;
        }

        //https://gafferongames.com/post/fix_your_timestep/
        //https://dewitters.com/dewitters-gameloop/
        update_game(&mut game, &mut next_tick);
        render_game(&mut game, &mut canvas, &mut sprite_manager, frame_time);

        let total_elapsed = (Instant::now() - start_system_time).as_secs_f32();
        let fps_game = game.ticks as f32 / total_elapsed;
        let fps_render = game.render_ticks as f32 / total_elapsed;

        if let Some(debug) = font_manager.render(
            "joystix monospace.ttf",
            &format!(
                "StateFPS: [{:02.1}] RenderFPS: [{:02.1}] T: {:02.2}",
                fps_game, fps_render, total_elapsed
            ),
        ) {
            render_debug(&mut game, &mut canvas, &debug);
        }

        canvas.present();

        std::thread::sleep(
            Duration::from_secs_f64(1.0 / 62.0)
                .checked_sub(Instant::now() - now)
                .unwrap_or(Duration::ZERO),
        );
        now = Instant::now();
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
    while Instant::now() > *next_tick && loops < MAX_FRAMESKIP {
        game.update();
        //tick counter
        *next_tick += skip_ticks;
        loops += 1;
        game.ticks += 1;
    }
}

fn render_debug(
    _game: &mut Game,
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    texture: &Texture,
) {
    let TextureQuery { width, height, .. } = texture.query();
    canvas
        .copy(texture, None, Some(Rect::new(0, 0, width, height)))
        .unwrap();
}
