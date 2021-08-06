use egui::Pos2;
use egui::Rect;
use std::collections::HashMap;
use std::time::Duration;
use std::time::Instant;
use systems::components::CollisionType::Static;
use systems::components::Size;
use winit::window::WindowBuilder;

use specs::prelude::*;

use game::Game;

use lib::logging::DisplayError;
use lib::*;
use systems::components::InputHandler;
use systems::components::Position;
use systems::components::Velocity;
use systems::input::InputSystem;
use systems::physics::Physics;
use systems::renderer::Renderer;

use opengl::DisplayBuild;
use sprite::{SpriteConfig, SpriteManager};
use systems::components::Collision;

mod game;
mod lib;

#[macro_use]
extern crate lazy_static;
extern crate image;
extern crate sdl2;
#[macro_use]
extern crate prettytable;
extern crate nalgebra_glm as glm;

use color_eyre::Result;

pub fn main() -> Result<()> {
    color_eyre::install()?;
    // winit_main()
    sdl_main()
}

pub fn winit_main() -> Result<()> {
    // Always include backtrace on panic.
    std::env::set_var("RUST_BACKTRACE", "1");
    let event_loop = winit::event_loop::EventLoop::new();
    let _window = WindowBuilder::new()
        .with_title("winit")
        .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
        .build(&event_loop)?;
    event_loop.run(move |_event, _, _control_flow| {});
    Ok(())
}

pub fn sdl_main() -> Result<()> {
    // Always include backtrace on panic.
    std::env::set_var("RUST_BACKTRACE", "1");
    let sdl_context = sdl2::init().map_err(DisplayError::from)?;
    let video_subsystem = sdl_context.video().map_err(DisplayError::from)?;

    let _ttf_context = sdl2::ttf::init()?;

    let gl_attr = video_subsystem.gl_attr();

    let egui = egui::CtxRef::default();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    // gl_attr.set_context_flags().forward_compatible().set();
    gl_attr.set_context_version(3, 3);

    let window = video_subsystem
        .window("rust-sdl2 demo: Video", 800, 600)
        .build_glium()
        .unwrap();

    // let raw = egui::RawInput {
    //     screen_rect: Some(Rect::from_two_pos(
    //         Default::default(),
    //         Pos2::new(800f32, 600f32),
    //     )),
    //     pixels_per_point: Some(
    //         video_subsystem
    //             .display_dpi(window.display_index().unwrap())
    //             .unwrap()
    //             .1,
    //     ),
    //     ..Default::default()
    // };

    let mut event_pump = sdl_context.event_pump().map_err(DisplayError::from)?;

    let mut sprite_manager = SpriteManager::new();

    // Initial game time.
    let start_system_time = Instant::now();
    let mut next_tick = start_system_time;

    let game = Game {
        ticks: 0,
        render_ticks: 0,
        start_system_time,
        running: true,
        time: start_system_time,
    };

    let mut world = World::new();

    let mut player_input = InputSystem::new(&mut event_pump);
    RunNow::setup(&mut player_input, &mut world);

    let mut physics: Physics = Default::default();
    RunNow::setup(&mut physics, &mut world);

    let mut renderer = Renderer {
        sprite_manager: &mut sprite_manager,
        window,
        render_set: None,
        now: start_system_time,
    };
    RunNow::setup(&mut renderer, &mut world);
    // renderer.prep();

    world.insert(game);
    world.insert(egui);
    // world.insert(raw);

    world
        .create_entity()
        .with(Velocity(0, 0))
        .with(Position(50, 50))
        .with(Size(18 * 3, 18 * 3))
        .with(renderer.sprite_manager.init("chicken"))
        .with(InputHandler(None))
        .with(Collision(None))
        .build();

    for x in 1..10 {
        for y in 1..10 {
            world
                .create_entity()
                .with(Velocity(0, 1))
                .with(Position(x * 18, y * 18))
                .with(Size(18, 18))
                .with(Collision(None))
                .with(renderer.sprite_manager.init("chicken"))
                .build();
        }
    }

    for x in 0..20 {
        world
            .create_entity()
            .with(Position(x * 32, 400))
            .with(Size(32, 32))
            .with(renderer.sprite_manager.init("tile"))
            .with(Collision(Some(Static)))
            .build();
    }
    // render a surface, and convert it to a texture bound to the canvas
    let mut now = Instant::now();
    let _frame_time = Duration::from_secs_f64(1.0 / 60.0);
    loop {
        player_input.run_now(&world);
        if !player_input.running {
            break;
        }

        //https://gafferongames.com/post/fix_your_timestep/
        //https://dewitters.com/dewitters-gameloop/
        const TICKS_PER_SECOND: u64 = 30;
        const MAX_FRAMESKIP: u64 = 5;
        let skip_ticks: Duration = Duration::from_millis(1000 / TICKS_PER_SECOND);
        let mut loops = 0;
        while Instant::now() > next_tick && loops < MAX_FRAMESKIP {
            physics.run_now(&world);
            //tick counter
            next_tick += skip_ticks;
            loops += 1;
            let mut game = world.write_resource::<Game>();
            game.ticks += 1;
        }
        renderer.run_now(&world);

        let mut game = world.write_resource::<Game>();
        game.render_ticks += 1;

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
