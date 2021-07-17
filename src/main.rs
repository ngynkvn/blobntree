mod lib;
extern crate sdl2;

use crate::ecs::Component;
use crate::ecs::Entity;
use crate::ecs::System;
use crate::ecs::World;
use crate::font::FontConfig;
use crate::state::PlayerState;
use crate::systems::InputHandler;
use crate::systems::InputSystem;
use crate::systems::Physics;
use crate::systems::Position;
use crate::systems::SpriteState;
use crate::systems::Velocity;
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
use systems::Renderer;

use crate::lib::sprite::{SpriteConfig, SpriteManager};

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

    sprite_manager.add(SpriteConfig {
        name: "mushroom",
        path: "sprites/mushroom.png",
        json: "sprites/mushroom.json",
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
    let mut player_input = InputSystem {
        event_pump: &mut event_pump,
    };

    let mut physics: Physics = Default::default();

    let mut renderer = Renderer {
        sprite_manager: &mut sprite_manager,
        canvas: &mut canvas,
    };

    world.register::<Position>();
    world.register::<Velocity>();
    world.register::<InputHandler>();
    world.register::<SpriteState>();


    world
        .create_entity()
        .with(Velocity(0, 1))
        .with(Position(600, 600))
        .with(SpriteState(renderer.sprite_manager.init("mushroom")))
        .with(InputHandler)
        .build();

    for x in 0..20 {
        for y in 0..10 {
            world
                .create_entity()
                .with(Velocity(0, 1))
                .with(Position(x * 50, y * 50))
                .with(SpriteState(renderer.sprite_manager.init("chicken")))
                .build();
        }
    }

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
        // world.run_system(
        //     &mut player_input,
        //     &[type_id::<Velocity>(), type_id::<InputHandler>()],
        // );
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
            &[type_id::<Position>(), type_id::<SpriteState>()],
        );
        println!("{:?}", Instant::now() - now);
        game.render_ticks += 1;

        let total_elapsed = (Instant::now() - start_system_time).as_secs_f32();
        let fps_game = game.ticks as f32 / total_elapsed;
        let fps_render = game.render_ticks as f32 / total_elapsed;
        println!(
            "{}",
            format!(
                "StateFPS: [{:02.1}] RenderFPS: [{:02.1}] T: {:02.2}",
                fps_game, fps_render, total_elapsed
            )
        );

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
