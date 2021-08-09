use crate::aseprite::AsepriteJSON;
use crate::aseprite::SpriteRect;

use crate::lib::sprite::Sprite;
use glium_glue::sdl2::SDL2Facade;

use glium::implement_vertex;
use glium::index::PrimitiveType;
use glium::texture::MipmapsOption::NoMipmap;
use glium::texture::RawImage2d;
use glium::texture::Texture2dArray;
use glium::Blend;
use glium::IndexBuffer;

use glium::VertexBuffer;

use glium::DrawParameters;
use glium::Program;
use glium::Surface;

use glium::uniform;

use image::GenericImageView;

use specs::prelude::*;

use crate::InputHandler;
use crate::Size;
use crate::SpriteConfig;
use crate::Velocity;

use std::time::Duration;
use std::time::Instant;

use num_traits::One;

use specs::{Join, ReadStorage, System};

use crate::lib::sprite::SpriteManager;
use crate::lib::systems::components::{Position, SpriteHandle};

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
            Some("sprites/tile.json"),
        ).unwrap());
        set
    };
}

#[derive(Clone, Copy)]
pub struct Vertex {
    pos: [f32; 2],
}
implement_vertex!(Vertex, pos);
// All relevant OpenGL objects needed for rendering.
pub struct RenderSet<'a> {
    pub program: Program,
    pub projection: glm::Mat4x4,
    pub vertex_buffer: VertexBuffer<Vertex>,
    pub index_buffer: IndexBuffer<u16>,
    pub draw_params: DrawParameters<'a>,
}

pub struct Renderer<'a> {
    pub sprite_manager: &'a mut SpriteManager,
    pub window: SDL2Facade,
    pub render_set: Option<RenderSet<'a>>,
    pub now: Instant,
}

fn load_sprite(display: &SDL2Facade, info: &TextureInfo) -> Sprite {
    let path = info.path;
    let image = image::open(path)
        .unwrap_or_else(|_| panic!("Cannot find {}", path))
        .to_rgba8();
    let texture: Texture2dArray = {
        if let Some(json) = &info.json {
            let sub_images = json
                .frames
                .iter()
                .map(|frame| {
                    let SpriteRect { x, y, w, h } = frame.frame;
                    let sub = image.view(x as u32, y as u32, w, h).to_image();
                    let dims = sub.dimensions();
                    let raw = sub.into_raw();
                    RawImage2d::from_raw_rgba(raw, dims)
                })
                .collect();
            Texture2dArray::with_mipmaps(display, sub_images, NoMipmap).unwrap()
        } else {
            let dims = image.dimensions();
            let raw = image.into_raw();
            let raw = RawImage2d::from_raw_rgba(raw, dims);
            Texture2dArray::with_mipmaps(display, vec![raw], NoMipmap).unwrap()
        }
    };
    Sprite {
        texture,
        info: info.clone(),
    }
}

impl<'a> Renderer<'a> {
    pub fn init_render_data(&mut self, _world: &mut World) {
        // gl::load_with(|name| self.video_subsystem.gl_get_proc_address(name) as *const _);
        // self.canvas.window().gl_set_context_to_current().unwrap();
        let program = Program::from_source(
            &self.window,
            include_str!("graphics/world.vert"),
            include_str!("graphics/world.frag"),
            None,
        )
        .unwrap();

        let sprites = {
            let textures = &TEXTURES;
            let mut data = vec![];
            for texture in textures.iter() {
                let raw = load_sprite(&self.window, texture);
                data.push(raw);
            }
            data
        };
        self.sprite_manager.add_sprites(sprites);

        let quad = [
            Vertex {
                pos: [0.0, 1.0], // top left
            },
            Vertex {
                pos: [1.0, 0.0], // bottom right
            },
            Vertex {
                pos: [0.0, 0.0], // bottom left
            },
            Vertex {
                pos: [1.0, 1.0], // top right
            },
        ];
        let vertex_buffer = VertexBuffer::new(&self.window, &quad).unwrap();
        // building the index buffer
        let index_buffer = IndexBuffer::new(
            &self.window,
            PrimitiveType::TrianglesList,
            &[0u16, 1, 2, 0, 3, 1],
        )
        .unwrap();
        self.render_set = Some(RenderSet {
            program,
            projection: glm::ortho(0.0, 800.0, 600.0, 0.0, -1.0, 1.0),
            vertex_buffer,
            index_buffer,
            draw_params: DrawParameters {
                blend: Blend::alpha_blending(),
                ..Default::default()
            },
        });

        println!("INIT DONE");
    }

    fn draw_sprite(
        frame: &mut glium::Frame,
        texture: &Sprite,
        render_set: &RenderSet,
        ((x, y), (w, h), frame_index): ((i32, i32), (f32, f32), usize),
    ) {
        use nalgebra_glm::{scale, translate, vec3};
        let mut model = nalgebra_glm::Mat4x4::one();
        let RenderSet {
            program,
            projection,
            vertex_buffer,
            index_buffer,
            draw_params,
        } = render_set;
        model = translate(&model, &vec3(x as f32, y as f32, 0.0));
        model = scale(&model, &vec3(w, h, 1.0));
        let uniforms = uniform! {
            model: model.data.0,
            projection: projection.data.0,
            image: texture.sampler(),
            index: frame_index as i32
        };
        frame
            .draw(vertex_buffer, index_buffer, program, &uniforms, draw_params)
            .unwrap()
    }
    fn _debug_info(&mut self) {}
}

type EntityData<'s> = (
    ReadStorage<'s, Position>,
    ReadStorage<'s, Velocity>,
    ReadStorage<'s, Size>,
    ReadStorage<'s, SpriteHandle>,
    ReadStorage<'s, InputHandler>,
);
impl<'a, 's> System<'s> for Renderer<'a> {
    type SystemData = (
        EntityData<'s>,
        Write<'s, egui::CtxRef>,
        Read<'s, egui::RawInput>,
    );
    fn run(
        &mut self,
        (
            (position, velocity, size, sprite_handle, input_handler),
            _egui_context,
            _egui_raw_input,
        ): Self::SystemData,
    ) {
        let elapsed = self.now.elapsed();
        // given Position and Velocity, calculate the "delta-frame"
        let interpolate = |(x, y): (i32, i32), (vx, vy): (i32, i32)| {
            (
                x + (vx as f64 * elapsed.as_secs_f64()) as i32,
                y + (vy as f64 * elapsed.as_secs_f64()) as i32,
            )
        };

        let mut target = self.window.draw();
        // Clear Screen
        target.clear_color(0.1, 0.1, 0.2, 1.0);

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
                self.sprite_manager.signal(handle, signal.0.as_ref());
            }

            // TODO -- fix timing?
            let (sprite, frame_index) = self
                .sprite_manager
                .next_frame(handle, Duration::from_secs_f64(1.0 / 60.0));
            if let Some(Velocity(vx, vy)) = velocity {
                let (dx, dy) = interpolate((x, y), (*vx, *vy));
                x = dx;
                y = dy;
            }
            let info = ((x, y), (size.0 as f32, size.1 as f32), frame_index);
            Self::draw_sprite(&mut target, sprite, self.render_set.as_ref().unwrap(), info);
        }
        target.finish().unwrap();
        // let ctx: &mut egui::CtxRef = &mut egui_context;
        // let raw: &egui::RawInput = &egui_raw_input;
        // ctx.begin_frame(raw.clone());
        // egui::Window::new("My Window").show(&ctx, |ui| {
        //     ui.label("hello, world");
        // });
        // let (_output, shapes) = ctx.end_frame();
        // let clipped_meshes = ctx.tessellate(shapes);
        // unsafe {
        //     let texture: &Texture = &egui_context.texture();
        // }
        // self.canvas.present();
        self.now = Instant::now();
    }

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
        self.init_render_data(world);
    }
}
