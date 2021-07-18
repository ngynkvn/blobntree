use crate::aseprite::AsepriteJSON;
use crate::aseprite::SpriteRect;

use crate::opengl::shader::Shader;
use crate::opengl::shader::SHADERS;
use crate::opengl::texture2d::Texture2D;
use crate::sprite::SpriteIndex;
use crate::HashMap;
use crate::InputHandler;
use crate::Size;
use crate::SpriteConfig;
use crate::Velocity;
use sdl2::VideoSubsystem;

use std::time::Instant;

use num_traits::One;
use sdl2::rect::Rect;
use sdl2::render::{TextureQuery, WindowCanvas};
use specs::{Join, ReadStorage, System};

use crate::game::Game;
use crate::lib::font::FontManager;
use crate::lib::sprite::SpriteManager;
use crate::lib::systems::components::{Position, SpriteHandle};
use crate::lif;
use std::fs::File;
use std::io::BufReader;

#[derive(Debug, Clone)]
pub struct TextureInfo {
    pub name: &'static str,
    pub path: &'static str,
    pub json: Option<AsepriteJSON>,
}

type Str = &'static str;
use color_eyre::Result;
impl TextureInfo {
    fn new(name: Str, path: Str, json_path: Option<Str>) -> Result<Self> {
        let mut info = TextureInfo {
            name,
            path,
            json: None,
        };
        if let Some(path) = json_path {
            let json_file = File::open(path)?;
            info.json = serde_json::from_reader(BufReader::new(json_file))?;
        }

        Ok(info)
    }
}

impl From<SpriteConfig> for TextureInfo {
    fn from(config: SpriteConfig) -> Self {
        let SpriteConfig { name, path, json } = config;
        let mut info = TextureInfo {
            name,
            path,
            json: None,
        };
        if let Ok(json_file) = File::open(json) {
            info.json = serde_json::from_reader(BufReader::new(json_file)).unwrap();
        }

        info
    }
}

lazy_static! {
    pub static ref TEXTURES: Vec<TextureInfo> = {
        let mut set = Vec::new();
        set.push(TextureInfo::new(
            "chicken",                           // Name
            "sprites/chicken_smear.png",         // Spritesheet path
            Some("sprites/chicken_smear.json"),  // Json path (Only for animated sprite sheets)
        ).unwrap());
        set.push(TextureInfo::new(
            "mushroom",
            "sprites/mushroom.png",
            Some("sprites/mushroom.json"),
        ).unwrap());
        set.push(TextureInfo::new(
            "tile",
            "sprites/tile.png",
            None,
        ).unwrap());
        set
    };
}

pub struct Renderer<'a, 's> {
    pub sprite_manager: &'a mut SpriteManager,
    pub font_manager: &'s mut FontManager<'a>,
    pub video_subsystem: &'s VideoSubsystem,
    pub canvas: &'a mut WindowCanvas,
    pub opengl_textures: HashMap<SpriteIndex, Texture2D>,
    pub quad_vao: u32,
    pub now: Instant,
}

static mut VBO: u32 = 0;
static mut VAO: u32 = 0;

impl<'a, 's> Renderer<'a, 's> {
    pub fn init_render_data(&mut self) {
        gl::load_with(|name| self.video_subsystem.gl_get_proc_address(name) as *const _);
        self.canvas.window().gl_set_context_to_current().unwrap();
        let shader = SHADERS.get("world").unwrap();
        let projection = nalgebra_glm::ortho(0.0, 800.0, 600.0, 0.0, -1.0, 1.0);
        shader.use_shader();
        shader.set("image", &(0i32));
        println!("Image set");
        shader.set("projection", &projection);
        println!("Ortho set");
        // self.sprite_manager.init_texture_data();
        unsafe {
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::BLEND);
            /*
            ┌──────────────┐
            │(0,0)    (1,0)│
            │              │
            │              │
            │              │
            │              │
            │              │
            │(0,1)    (1,1)│
            └──────────────┘
            */
            let vertices: [f32; 24] = [
                // pos      // tex
                0.0, 1.0, 0.0, 1.0, //
                1.0, 0.0, 1.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, //
                0.0, 1.0, 0.0, 1.0, //
                1.0, 1.0, 1.0, 1.0, //
                1.0, 0.0, 1.0, 0.0, //
            ];
            gl::GenVertexArrays(1, &mut self.quad_vao);
            gl::GenBuffers(1, &mut VBO);
            dbg!(self.quad_vao);
            dbg!(VBO);

            gl::BindBuffer(gl::ARRAY_BUFFER, VBO);
            // Send the vertices to the array buffer VBO.
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (std::mem::size_of::<f32>() * 24) as _,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            // Bind the Vertex array
            gl::BindVertexArray(self.quad_vao);
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                4,
                gl::FLOAT,
                gl::FALSE,
                (4 * std::mem::size_of::<f32>()) as _,
                std::ptr::null(),
            );
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);

            // let test_texture = Texture2D::from(TEXTURES[0].clone());
            // dbg!(&test_texture);
            // self.opengl_textures.insert(0, test_texture);
        }
        println!("INIT DONE");
    }

    fn draw_sprite(
        quad_vao: u32,
        texture: &Texture2D,
        shader: &Shader,
        (x, y): (i32, i32),
        size: (f32, f32),
        frame: SpriteRect,
    ) {
        use nalgebra_glm::{scale, translate, vec2, vec3};
        unsafe {
            shader.use_shader();
            let mut model = nalgebra_glm::Mat4x4::one();
            model = translate(&model, &vec3(x as f32, y as f32, 0.0));
            model = scale(&model, &vec3(size.0, size.1, 1.0));
            shader.set("model", &model);
            // shader.set("tex_offset", &vec2(tx, ty));
            let SpriteRect { x, y, w, h } = frame;
            // Sprite top left corner
            shader.set("sprite_pos", &vec2(x, y));
            // Sprite width and height
            shader.set("sprite_dim", &vec2(w as i32, h as i32));

            gl::ActiveTexture(gl::TEXTURE0);
            assert_ne!(texture.id, 0);
            texture.bind();
            gl::BindVertexArray(quad_vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
            let error = gl::GetError();
            if error != 0 {
                dbg!(error);
            }
            gl::BindVertexArray(0);
        }
    }
    fn debug_info(&mut self, game: &Game) {
        let total_elapsed = (Instant::now() - game.start_system_time).as_secs_f32();
        let fps_game = game.ticks as f32 / total_elapsed;
        let fps_render = game.render_ticks as f32 / total_elapsed;

        let render = self.font_manager.render(
            "joystix monospace.ttf",
            &format!(
                "StateFPS: [{:02.1}] RenderFPS: [{:02.1}] T: {:02.2}",
                fps_game, fps_render, total_elapsed
            ),
        );
        lif![Some(debug) = render => {
            let TextureQuery { width, height, .. } = debug.query();
            self.canvas
                .copy(&debug, None, Rect::from((0, 0, width, height))).unwrap();
        }];
    }
}

static mut i: usize = 0;
impl<'a, 's> System<'s> for Renderer<'a, 's> {
    type SystemData = (
        ReadStorage<'s, Position>,
        ReadStorage<'s, Velocity>,
        ReadStorage<'s, Size>,
        ReadStorage<'s, SpriteHandle>,
        ReadStorage<'s, InputHandler>,
    );
    fn run(&mut self, (position, velocity, size, sprite_handle, input_handler): Self::SystemData) {
        let elapsed = self.now.elapsed();
        // given Position and Velocity, calculate the "delta-frame"
        let interpolate = |(x, y): (i32, i32), (vx, vy): (i32, i32)| {
            (
                x + (vx as f64 * elapsed.as_secs_f64()) as i32,
                y + (vy as f64 * elapsed.as_secs_f64()) as i32,
            )
        };

        gl::load_with(|name| self.video_subsystem.gl_get_proc_address(name) as *const _);
        self.canvas.window().gl_set_context_to_current().unwrap();

        // Clear Screen
        unsafe {
            gl::ClearColor(0.1, 0.1, 0.2, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        let shader = SHADERS.get("world").unwrap();
        let sprite_manager = &mut self.sprite_manager;
        for (pos, velocity, input, size, handle) in (
            &position,
            velocity.maybe(),
            input_handler.maybe(),
            &size,
            &sprite_handle,
        )
            .join()
        {
            let (mut x, mut y) = (pos.0, pos.1);
            if let Some(signal) = input {
                sprite_manager.signal(handle, signal.0.as_ref());
            }
            let (texture, next_frame) = sprite_manager.next_frame(handle, elapsed);
            if let Some(Velocity(vx, vy)) = velocity {
                let (dx, dy) = interpolate((x, y), (*vx, *vy));
                x = dx;
                y = dy;
            }

            Self::draw_sprite(
                self.quad_vao,
                texture,
                shader,
                (x, y),
                (size.0 as f32, size.1 as f32),
                next_frame,
            );
        }
        // self.debug_info(&game);
        self.canvas.present();
        self.now = Instant::now();
    }
}
