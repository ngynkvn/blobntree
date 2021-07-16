mod lib;
extern crate sdl2;

use crate::ecs::Component;
use crate::ecs::Entity;
use crate::ecs::System;
use crate::ecs::World;
use crate::font::FontConfig;
use crate::state::PlayerState;
use sdl2::image::{self, InitFlag, LoadTexture};
use std::collections::HashSet;
use std::iter::FromIterator;

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
use std::any::TypeId;

use crate::lib::sprite::{SpriteConfig, SpriteManager};

struct Velocity(i32, i32);
struct Position(i32, i32);
struct SpriteName(&'static str);

impl Component for Velocity {}

impl Component for Position {}

impl Component for SpriteName {}

#[derive(Default)]
struct Physics {}

impl System for Physics {
    fn update<'a>(&mut self, entities: impl Iterator<Item = &'a mut Entity>) {
        for entity in entities {
            let Position(x, y) = entity.get::<Position>();
            let Velocity(vx, vy) = entity.get::<Velocity>();
            let (x, mut y) = (x + vx, y + vy);
            if y > 800 {
                y = 0;
            }
            entity.set(Position(x, y));
        }
    }
}

struct Renderer<'a, 's> {
    sprite_manager: &'s mut SpriteManager<'a>,
    canvas: &'a mut WindowCanvas,
}

impl<'s, 'a> System for Renderer<'a, 's> {
    fn update<'b>(&mut self, entities: impl Iterator<Item = &'b mut Entity>) {
        let mut i = 0;
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();
        for entity in entities {
            i += 1;
            let name = entity.get::<SpriteName>();
            let position = entity.get::<Position>();
            let sprite = self.sprite_manager.get(name.0).unwrap();

            let (texture, rect) = sprite.next_frame(Duration::from_secs_f64((1.0 / 60.0) / 10.0));
            let position = Point::new(position.0, position.1);

            self.canvas
                .copy(
                    texture,
                    rect,
                    Rect::from_center(position, rect.width() * 3, rect.height() * 3),
                )
                .unwrap();
        }
        self.canvas.present();
    }
}

fn type_id<T: 'static>() -> TypeId {
    TypeId::of::<T>()
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

    let mut world = World::new();

    world.register::<Position>();
    world.register::<Velocity>();
    world.register::<SpriteName>();

    for i in 0..10000 {
        world
            .create_entity()
            .with(Velocity(0, 1))
            .with(Position(i * 30, i * 30))
            .with(SpriteName("chicken"))
            .build();
    }

    let mut physics: Physics = Default::default();

    let mut renderer = Renderer {
        sprite_manager: &mut sprite_manager,
        canvas: &mut canvas,
    };
    /**
    // world::register<InputHandler>();

        // .with(InputHandler())
        .build()
    loop {
        world.run_system(InputHandler);
        world.run_system(UpdateGame);
        world.run_system(RenderGame);
    }

     */
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
        const TICKS_PER_SECOND: u64 = 30;
        const MAX_FRAMESKIP: u64 = 5;
        let skip_ticks: Duration = Duration::from_millis(1000 / TICKS_PER_SECOND);
        let mut loops = 0;
        while Instant::now() > next_tick && loops < MAX_FRAMESKIP {
            world.run_system(
                &mut physics,
                &[type_id::<Position>(), type_id::<Velocity>()],
            );
            //tick counter
            next_tick += skip_ticks;
            loops += 1;
            game.ticks += 1;
        }
        world.run_system(
            &mut renderer,
            &[type_id::<Position>(), type_id::<SpriteName>()],
        );
        println!("{:?}", Instant::now() - now);
        game.render_ticks += 1;

        let total_elapsed = (Instant::now() - start_system_time).as_secs_f32();
        let fps_game = game.ticks as f32 / total_elapsed;
        let fps_render = game.render_ticks as f32 / total_elapsed;
        // println!(
        //     "{}",
        //     format!(
        //         "StateFPS: [{:02.1}] RenderFPS: [{:02.1}] T: {:02.2}",
        //         fps_game, fps_render, total_elapsed
        //     )
        // );

        // if let Some(debug) = font_manager.render(
        //     "joystix monospace.ttf",
        //     &format!(
        //         "StateFPS: [{:02.1}] RenderFPS: [{:02.1}] T: {:02.2}",
        //         fps_game, fps_render, total_elapsed
        //     ),
        // ) {
        //     render_debug(&mut game, &mut canvas, &debug);
        // }

        // canvas.present();

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
