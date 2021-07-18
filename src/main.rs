use std::collections::HashMap;
use std::time::Duration;
use std::time::Instant;
use systems::components::CollisionType::Static;
use systems::components::Size;
use systems::components::SpriteHandle;

use sdl2::pixels::Color;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use sdl2::video::WindowContext;
use specs::prelude::*;

use font::FontManager;
use game::Game;

use lib::logging::DisplayError;
use lib::*;
use systems::components::InputHandler;
use systems::components::Position;
use systems::components::Velocity;
use systems::input::InputSystem;
use systems::physics::Physics;
use systems::renderer::Renderer;

use font::FontConfig;
use sprite::{SpriteConfig, SpriteManager};
use systems::components::{Collision, StaticSprite};

mod game;
mod lib;

#[macro_use]
extern crate lazy_static;
extern crate image;
extern crate sdl2;
#[macro_use]
extern crate prettytable;

fn find_sdl_gl_driver() -> Option<u32> {
    for (index, item) in sdl2::render::drivers().enumerate() {
        if item.name == "opengl" {
            return Some(index as u32);
        }
    }
    None
}

use color_eyre::Result;

pub fn main() -> Result<()> {
    color_eyre::install()?;
    use sdl2::image;
    use sdl2::image::InitFlag;
    // Always include backtrace on panic.
    std::env::set_var("RUST_BACKTRACE", "1");
    let sdl_context = sdl2::init().map_err(DisplayError::from)?;
    let video_subsystem = sdl_context.video().map_err(DisplayError::from)?;

    let ttf_context = sdl2::ttf::init()?;

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    // gl_attr.set_context_flags().forward_compatible().set();
    gl_attr.set_context_version(3, 3);

    let window = video_subsystem
        .window("rust-sdl2 demo: Video", 800, 600)
        .position_centered()
        .opengl()
        .build()?;

    dbg!(gl_attr.context_profile());
    dbg!(gl_attr.context_version());

    let mut canvas = window
        .into_canvas()
        .index(find_sdl_gl_driver().unwrap())
        .build()?;

    let texture_creator = canvas.texture_creator();
    let mut event_pump = sdl_context.event_pump().map_err(DisplayError::from)?;

    let mut sprite_manager = SpriteManager::new();
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

    // TODO -- Some sprites don't have animation data. should be able to specify static sprites.
    sprite_manager.add(SpriteConfig {
        name: "tile",
        path: "sprites/tile.png",
        json: "sprites/tile.json",
    });

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

    let game = Game {
        ticks: 0,
        render_ticks: 0,
        start_system_time,
        running: true,
    };

    let mut world = World::new();

    let mut player_input = InputSystem::new(&mut event_pump);

    let mut physics: Physics = Default::default();

    let mut renderer = Renderer {
        sprite_manager: &mut sprite_manager,
        font_manager: &mut font_manager,
        video_subsystem: &video_subsystem,
        canvas: &mut canvas,
        quad_vao: 0,
        opengl_textures: HashMap::new(),
        now: Instant::now(),
    };
    // renderer.prep();
    renderer.init_render_data();
    unsafe {
        let gl_version = std::ffi::CStr::from_ptr(gl::GetString(gl::VERSION) as *const _);
        dbg!(gl_version);
    }

    world.insert(game);
    world.register::<Position>();
    world.register::<Velocity>();
    world.register::<Size>();
    world.register::<InputHandler>();
    world.register::<SpriteHandle>();
    world.register::<StaticSprite>();
    world.register::<Collision>();

    world
        .create_entity()
        .with(Velocity(0, 0))
        .with(Position(200, 200))
        .with(Size(50, 50))
        .with(renderer.sprite_manager.init("chicken"))
        .with(InputHandler(None))
        .with(Collision(None))
        .build();

    for x in 1..3 {
        for y in 1..3 {
            world
                .create_entity()
                .with(Velocity(0, 1))
                .with(Position(x * 50, y * 50))
                .with(Size(16, 16))
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
